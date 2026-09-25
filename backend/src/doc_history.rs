//! 변경 이력 조회와 임의 시점 복원.

use std::collections::{HashMap, HashSet};

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;

use crate::auth::CurrentUser;
use crate::diff;
use crate::doc_edit::{self, Sentences};
use crate::error::AppError;
use crate::state::AppState;
use crate::util::now_unix;

/// 편집이 하나도 얹히지 않은 자동 기준선의 항목 id. 행이 아니라 **자리**라서 예약어다.
pub const AUTO: &str = "auto";

pub mod kind {
    pub const AUTO: &str = "auto";
    pub const EDIT: &str = "edit";
    pub const RESTORE: &str = "restore";
}

/// 복원의 출처. 시점을 고른 것은 언제나 사람이고, 모델이 끼어들지 않는다.
const RESTORE_SOURCE: &str = "user_direct";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/analyses/{id}/features/{key}/history", get(list))
        // 항목도 주소를 가진다 — 되돌리기 전에 무엇이 달라지는지 보는 자리이고,
        // 새로고침이 같은 미리보기를 다시 그려야 한다.
        .route("/api/analyses/{id}/features/{key}/history/{entry}", get(entry))
        .route(
            "/api/analyses/{id}/features/{key}/history/{entry}/restore",
            post(restore),
        )
}

#[derive(sqlx::FromRow)]
pub(crate) struct EditRow {
    pub id: String,
    pub feature_key: String,
    pub scenario_index: i64,
    pub request: String,
    pub source: String,
    pub before_json: String,
    pub after_json: String,
    pub created_at: i64,
    pub decided_at: Option<i64>,
    pub carried_from: Option<String>,
    pub after_restore: Option<String>,
}

#[derive(sqlx::FromRow)]
pub(crate) struct RestoreRow {
    pub id: String,
    pub feature_key: String,
    pub target_kind: String,
    pub target_id: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EntryView {
    id: String,
    kind: String,
    source: String,
    at: i64,
    request: Option<String>,
    before: Option<Sentences>,
    after: Vec<Sentences>,
    carried_from: Option<String>,
    restored_to: Option<String>,
    current: bool,
    standing: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryView {
    feature_key: String,
    feature_name: Option<String>,
    entries: Vec<EntryView>,
    scenarios: Vec<Sentences>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewView {
    entry_id: String,
    scenarios: Vec<Sentences>,
    lines: Vec<diff::ScenarioLine>,
    is_current: bool,
}

enum Step<'a> {
    Edit(&'a EditRow),
    Restore(&'a RestoreRow),
}

/// 어느 세대에도 들지 못한 편집(가리키는 복원 행이 사라진 경우)은 마지막에 붙인다.
/// 이력에서 조용히 빠지느니 순서가 거친 편이 낫다.
fn steps<'a>(edits: &'a [EditRow], restores: &'a [RestoreRow]) -> Vec<Step<'a>> {
    let mut out: Vec<Step<'a>> = Vec::with_capacity(edits.len() + restores.len());
    let mut placed: HashSet<&str> = HashSet::new();
    for edit in edits.iter().filter(|e| e.after_restore.is_none()) {
        placed.insert(edit.id.as_str());
        out.push(Step::Edit(edit));
    }
    for restore in restores {
        out.push(Step::Restore(restore));
        for edit in edits
            .iter()
            .filter(|e| e.after_restore.as_deref() == Some(restore.id.as_str()))
        {
            placed.insert(edit.id.as_str());
            out.push(Step::Edit(edit));
        }
    }
    for edit in edits.iter().filter(|e| !placed.contains(e.id.as_str())) {
        out.push(Step::Edit(edit));
    }
    out
}

/// 사건을 순서대로 걸으며 「그 사건 직후에 서 있던 편집」을 남긴다.
fn walk(steps: &[Step<'_>]) -> (Vec<String>, HashMap<String, Vec<String>>) {
    let mut active: Vec<String> = Vec::new();
    let mut snapshots: HashMap<String, Vec<String>> = HashMap::new();
    snapshots.insert(AUTO.to_string(), Vec::new());
    for step in steps {
        match step {
            Step::Edit(edit) => {
                active.push(edit.id.clone());
                snapshots.insert(edit.id.clone(), active.clone());
            }
            Step::Restore(restore) => {
                active = match restore.target_id.as_deref() {
                    None => Vec::new(),
                    Some(target) => snapshots.get(target).cloned().unwrap_or_default(),
                };
                snapshots.insert(restore.id.clone(), active.clone());
            }
        }
    }
    (active, snapshots)
}

pub(crate) async fn approved_edits(
    state: &AppState,
    analysis_id: &str,
) -> Result<Vec<EditRow>, AppError> {
    Ok(sqlx::query_as::<_, EditRow>(
        "SELECT id, feature_key, scenario_index, request, source, before_json, after_json, \
                created_at, decided_at, carried_from, after_restore \
           FROM feature_doc_edits WHERE analysis_id = ? AND status = ? \
          ORDER BY created_at, rowid",
    )
    .bind(analysis_id)
    .bind(doc_edit::status::APPROVED)
    .fetch_all(&state.db)
    .await?)
}

pub(crate) async fn restores(
    state: &AppState,
    analysis_id: &str,
) -> Result<Vec<RestoreRow>, AppError> {
    Ok(sqlx::query_as::<_, RestoreRow>(
        "SELECT id, feature_key, target_kind, target_id, created_at \
           FROM feature_doc_restores WHERE analysis_id = ? ORDER BY feature_key, seq",
    )
    .bind(analysis_id)
    .fetch_all(&state.db)
    .await?)
}

/// 지금 문서에 서 있는 승인 편집 id.
pub async fn standing_edits(
    state: &AppState,
    analysis_id: &str,
) -> Result<HashSet<String>, AppError> {
    let restore_rows = restores(state, analysis_id).await?;
    let edit_rows = approved_edits(state, analysis_id).await?;
    if restore_rows.is_empty() {
        return Ok(edit_rows.into_iter().map(|row| row.id).collect());
    }
    let mut standing = HashSet::new();
    let keys: Vec<String> = {
        let mut seen: Vec<String> = Vec::new();
        for row in &edit_rows {
            if !seen.contains(&row.feature_key) {
                seen.push(row.feature_key.clone());
            }
        }
        seen
    };
    for key in keys {
        let edits: Vec<EditRow> = edit_rows
            .iter()
            .filter(|row| row.feature_key == key)
            .map(clone_edit)
            .collect();
        let feature_restores: Vec<RestoreRow> = restore_rows
            .iter()
            .filter(|row| row.feature_key == key)
            .map(clone_restore)
            .collect();
        let (active, _) = walk(&steps(&edits, &feature_restores));
        standing.extend(active);
    }
    Ok(standing)
}

fn clone_edit(row: &EditRow) -> EditRow {
    EditRow {
        id: row.id.clone(),
        feature_key: row.feature_key.clone(),
        scenario_index: row.scenario_index,
        request: row.request.clone(),
        source: row.source.clone(),
        before_json: row.before_json.clone(),
        after_json: row.after_json.clone(),
        created_at: row.created_at,
        decided_at: row.decided_at,
        carried_from: row.carried_from.clone(),
        after_restore: row.after_restore.clone(),
    }
}

fn clone_restore(row: &RestoreRow) -> RestoreRow {
    RestoreRow {
        id: row.id.clone(),
        feature_key: row.feature_key.clone(),
        target_kind: row.target_kind.clone(),
        target_id: row.target_id.clone(),
        created_at: row.created_at,
    }
}

/// 이 feature 의 편집이 지금 서 있는 복원 — 새 편집 행이 그 뒤에 선다.
pub async fn current_restore(
    state: &AppState,
    analysis_id: &str,
    feature_key: &str,
) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM feature_doc_restores WHERE analysis_id = ? AND feature_key = ? \
          ORDER BY seq DESC LIMIT 1",
    )
    .bind(analysis_id)
    .bind(feature_key)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// 주어진 편집 목록만 얹은 그 feature 의 시나리오 — 「그 시점의 상태」.
async fn scenarios_with(
    state: &AppState,
    analysis_id: &str,
    feature_key: &str,
    edits: &[EditRow],
    ids: &[String],
) -> Result<Vec<Sentences>, AppError> {
    let mut doc = doc_edit::base_document(state, analysis_id).await?;
    for id in ids {
        let Some(edit) = edits.iter().find(|row| &row.id == id) else {
            continue;
        };
        let Ok(after) = serde_json::from_str::<Vec<Sentences>>(&edit.after_json) else {
            continue;
        };
        doc_edit::splice(&mut doc, feature_key, edit.scenario_index as usize, &after);
    }
    doc_edit::scenarios_of(&doc, feature_key)
}

fn as_scenarios(sentences: &[Sentences]) -> Vec<diff::Scenario> {
    sentences
        .iter()
        .map(|s| diff::scenario_of(&s.given, &s.when, &s.then))
        .collect()
}

async fn load(
    state: &AppState,
    analysis_id: &str,
    feature_key: &str,
) -> Result<(Vec<EditRow>, Vec<RestoreRow>), AppError> {
    let edits = approved_edits(state, analysis_id).await?;
    let restore_rows = restores(state, analysis_id).await?;
    let edits: Vec<EditRow> = edits
        .iter()
        .filter(|row| row.feature_key == feature_key)
        .map(clone_edit)
        .collect();
    let restore_rows: Vec<RestoreRow> = restore_rows
        .iter()
        .filter(|row| row.feature_key == feature_key)
        .map(clone_restore)
        .collect();
    Ok((edits, restore_rows))
}

async fn baseline_at(state: &AppState, analysis_id: &str) -> Result<i64, AppError> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT created_at FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(analysis_id)
    .bind(crate::pipeline::ACCEPTANCE_DEPENDENCIES)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.map(|(at,)| at).unwrap_or(0))
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, key)): Path<(String, String)>,
) -> Result<Json<HistoryView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let doc = doc_edit::document_of(&state, &id).await?;
    let scenarios = doc_edit::scenarios_of(&doc, &key)?;

    let (edits, restore_rows) = load(&state, &id, &key).await?;
    let ordered = steps(&edits, &restore_rows);
    let (active, _) = walk(&ordered);
    let standing: HashSet<&str> = active.iter().map(String::as_str).collect();
    let last: Option<&str> = ordered.last().map(|step| match step {
        Step::Edit(edit) => edit.id.as_str(),
        Step::Restore(restore) => restore.id.as_str(),
    });

    let mut entries = vec![EntryView {
        id: AUTO.to_string(),
        kind: kind::AUTO.to_string(),
        source: doc_edit::SOURCE_AUTO.to_string(),
        at: baseline_at(&state, &id).await?,
        request: None,
        before: None,
        after: Vec::new(),
        carried_from: None,
        restored_to: None,
        current: last.is_none(),
        standing: true,
    }];
    for step in &ordered {
        entries.push(match step {
            Step::Edit(edit) => EntryView {
                id: edit.id.clone(),
                kind: kind::EDIT.to_string(),
                source: edit.source.clone(),
                at: edit.decided_at.unwrap_or(edit.created_at),
                request: Some(edit.request.clone()),
                before: Some(doc_edit::sentences_from_json(&edit.before_json)?),
                after: doc_edit::scenarios_from_json(&edit.after_json)?,
                carried_from: edit.carried_from.clone(),
                restored_to: None,
                current: last == Some(edit.id.as_str()),
                standing: standing.contains(edit.id.as_str()),
            },
            Step::Restore(restore) => EntryView {
                id: restore.id.clone(),
                kind: kind::RESTORE.to_string(),
                source: RESTORE_SOURCE.to_string(),
                at: restore.created_at,
                request: None,
                before: None,
                after: Vec::new(),
                carried_from: None,
                restored_to: Some(
                    restore
                        .target_id
                        .clone()
                        .unwrap_or_else(|| AUTO.to_string()),
                ),
                current: last == Some(restore.id.as_str()),
                standing: true,
            },
        });
    }

    Ok(Json(HistoryView {
        feature_key: key.clone(),
        feature_name: diff::feature_name(&doc, &key),
        entries,
        scenarios,
    }))
}

async fn snapshot_of(
    state: &AppState,
    analysis_id: &str,
    feature_key: &str,
    entry: &str,
) -> Result<(Vec<EditRow>, Vec<String>, bool), AppError> {
    let (edits, restore_rows) = load(state, analysis_id, feature_key).await?;
    let ordered = steps(&edits, &restore_rows);
    let (active, snapshots) = walk(&ordered);
    let wanted = snapshots.get(entry).cloned().ok_or(AppError::NotFound)?;
    let is_current = wanted == active;
    Ok((edits, wanted, is_current))
}

async fn entry(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, key, entry)): Path<(String, String, String)>,
) -> Result<Json<PreviewView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let (edits, wanted, is_current) = snapshot_of(&state, &id, &key, &entry).await?;
    let scenarios = scenarios_with(&state, &id, &key, &edits, &wanted).await?;
    let doc = doc_edit::document_of(&state, &id).await?;
    let now = doc_edit::scenarios_of(&doc, &key)?;
    let lines = diff::feature_diff(
        &key,
        "",
        None,
        &as_scenarios(&now),
        &as_scenarios(&scenarios),
        None,
        None,
    )
    .map(|d| d.scenario_lines)
    .unwrap_or_default();
    Ok(Json(PreviewView {
        entry_id: entry,
        scenarios,
        lines,
        is_current,
    }))
}

async fn restore(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, key, entry)): Path<(String, String, String)>,
) -> Result<Json<HistoryView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let (_, _, is_current) = snapshot_of(&state, &id, &key, &entry).await?;
    if is_current {
        return Err(AppError::Conflict("이미 그 시점의 상태입니다".into()));
    }

    let target_kind = if entry == AUTO {
        kind::AUTO
    } else if sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM feature_doc_restores WHERE id = ? AND analysis_id = ?",
    )
    .bind(&entry)
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .is_some()
    {
        kind::RESTORE
    } else {
        kind::EDIT
    };

    let seq: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(seq), 0) + 1 FROM feature_doc_restores \
          WHERE analysis_id = ? AND feature_key = ?",
    )
    .bind(&id)
    .bind(&key)
    .fetch_one(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO feature_doc_restores \
           (id, analysis_id, feature_key, seq, target_kind, target_id, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&id)
    .bind(&key)
    .bind(seq)
    .bind(target_kind)
    .bind(if entry == AUTO { None } else { Some(entry.as_str()) })
    .bind(now_unix())
    .execute(&state.db)
    .await?;

    list(State(state), CurrentUser(user), Path((id, key))).await
}

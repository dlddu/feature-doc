//! AC3.5: 코드 자동 분석과 사용자 편집의 충돌 처리.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::auth::CurrentUser;
use crate::diff;
use crate::doc_edit::{self, Sentences};
use crate::error::AppError;
use crate::feature_add::SOURCE_USER_DIRECT;
use crate::llm::{self, Ask};
use crate::pipeline;
use crate::state::AppState;
use crate::util::now_unix;

pub mod status {
    pub const OPEN: &str = "open";
    pub const AUTO: &str = "auto";
    pub const MINE: &str = "mine";
    pub const MERGED: &str = "merged";
}

pub const MERGE_REQUEST: &str = "내가 고친 문장과 이번 자동 결과 합치기";
pub const MERGE_REQUEST_KEEP: &str =
    "내가 고친 문장과 이번 자동 결과 합치기 — 내가 덧붙인 문단은 그대로";

/// 충돌 화면의 거부에는 사유 칸이 없다 — 회피의 단위는 사유가 아니라 문면이므로
/// (`doc_edit::rejections`), 고정 사유로도 다음 합치기는 같은 문장을 피한다.
const REJECT_REASON: &str = "합친 문장을 버렸다";

const SYSTEM_MERGE: &str = "\
You reconcile two versions of one acceptance scenario of an end-user feature: the
version the person reading it edited, and the version the automatic analysis now
produces after the code changed.
Answer with the scenarios that should stand in its place — one, or up to three.
Keep what the reader added, and layer on only the new rule the automatic version
introduces. Keep the given / when / then shape.
Write in the words of a person using the product — never developer vocabulary,
never an HTTP route or a function name.
Never invent behaviour the feature does not have.
Some merged sentences were already rejected by this reader; they are listed. Do
not propose them again — take a different one.";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/analyses/{id}/conflicts", get(list))
        .route("/api/analyses/{id}/conflicts/{conflict}", get(one))
        .route(
            "/api/analyses/{id}/conflicts/{conflict}/decision",
            post(decide),
        )
        .route("/api/analyses/{id}/conflicts/{conflict}/merge", post(merge))
        .route(
            "/api/analyses/{id}/conflicts/{conflict}/merge-decision",
            post(decide_merge),
        )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictView {
    id: String,
    feature_key: String,
    feature_name: String,
    scenario_index: usize,
    status: String,
    source: String,
    request: String,
    mine: Vec<Sentences>,
    /// 그 편집이 고쳐 쓴 당시의 자동 문장.
    before: Sentences,
    auto: Sentences,
    mine_decided_at: i64,
    previous_analysis_id: String,
    merge_proposal: Option<MergeProposalView>,
    decided_at: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeProposalView {
    id: String,
    status: String,
    after: Vec<Sentences>,
    removed: Vec<String>,
    added: Vec<String>,
    changed_lines: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListView {
    open: usize,
    conflicts: Vec<ConflictView>,
}

#[derive(Deserialize)]
struct DecisionReq {
    decision: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergeReq {
    #[serde(default)]
    keep_mine: bool,
}

#[derive(sqlx::FromRow)]
struct ConflictRow {
    id: String,
    previous_analysis_id: String,
    feature_key: String,
    scenario_index: i64,
    request: String,
    source: String,
    mine_json: String,
    before_json: String,
    auto_json: String,
    mine_decided_at: i64,
    status: String,
    merge_edit_id: Option<String>,
    decided_at: Option<i64>,
}

struct Inherited {
    edit_id: String,
    feature_key: String,
    scenario_index: usize,
    request: String,
    source: String,
    before: Sentences,
    after: Vec<Sentences>,
    decided_at: i64,
}

fn same(a: &Sentences, b: &Sentences) -> bool {
    let norm = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    norm(&a.given) == norm(&b.given) && norm(&a.when) == norm(&b.when) && norm(&a.then) == norm(&b.then)
}

/// 재분석 문서가 저장된 직후, 직전 분석의 편집을 이 문서 위에 재생한다.
pub async fn inherit(state: &AppState, analysis_id: &str, doc: &Value) -> Result<(), AppError> {
    sqlx::query("DELETE FROM feature_doc_edits WHERE analysis_id = ? AND carried_from IS NOT NULL")
        .bind(analysis_id)
        .execute(&state.db)
        .await?;
    sqlx::query("DELETE FROM feature_doc_conflicts WHERE analysis_id = ? AND status = ?")
        .bind(analysis_id)
        .bind(status::OPEN)
        .execute(&state.db)
        .await?;

    let Some(previous) = previous_documented(state, analysis_id).await? else {
        return Ok(());
    };

    let mut effective: HashMap<String, Vec<Sentences>> = HashMap::new();
    for key in diff::feature_keys(doc) {
        let list = doc_edit::scenarios_of(doc, &key).unwrap_or_default();
        effective.insert(key, list);
    }

    let now = now_unix();
    for item in inherited_from(state, &previous).await? {
        let Some(list) = effective.get_mut(&item.feature_key) else {
            continue;
        };
        let at = item.scenario_index;
        let found = if at < list.len() && same(&list[at], &item.before) {
            Some(at)
        } else {
            list.iter().position(|s| same(s, &item.before))
        };
        match found {
            Some(slot) => {
                list.splice(slot..slot + 1, item.after.iter().cloned());
                insert_edit(
                    state,
                    analysis_id,
                    &item.feature_key,
                    slot,
                    &item.request,
                    &item.before,
                    &item.after,
                    doc_edit::status::APPROVED,
                    &item.source,
                    Some(&item.edit_id),
                    now,
                    Some(item.decided_at),
                )
                .await?;
            }
            None if at < list.len() => {
                sqlx::query(
                    "INSERT OR IGNORE INTO feature_doc_conflicts \
                       (id, analysis_id, previous_analysis_id, edit_id, feature_key, \
                        scenario_index, request, source, mine_json, before_json, auto_json, \
                        mine_decided_at, status, created_at) \
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(uuid::Uuid::new_v4().to_string())
                .bind(analysis_id)
                .bind(&previous)
                .bind(&item.edit_id)
                .bind(&item.feature_key)
                .bind(at as i64)
                .bind(&item.request)
                .bind(&item.source)
                .bind(to_json(&item.after)?)
                .bind(to_json(&item.before)?)
                .bind(to_json(&list[at])?)
                .bind(item.decided_at)
                .bind(status::OPEN)
                .bind(now)
                .execute(&state.db)
                .await?;
            }
            None => {}
        }
    }
    Ok(())
}

/// 같은 타깃의 직전 분석 중 인수 문서를 가진 것 — 편집은 문서 위에만 있으므로 문서 없는 분석은 이어받을 것이 없다.
async fn previous_documented(state: &AppState, analysis_id: &str) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT prev.id \
           FROM analyses cur \
           JOIN analyses prev \
             ON prev.user_id = cur.user_id AND prev.repo_owner = cur.repo_owner \
            AND prev.repo_name = cur.repo_name AND prev.branch = cur.branch \
            AND (prev.created_at < cur.created_at \
                 OR (prev.created_at = cur.created_at AND prev.rowid < cur.rowid)) \
           JOIN analysis_documents d ON d.analysis_id = prev.id AND d.kind = ? \
          WHERE cur.id = ? \
          ORDER BY prev.created_at DESC, prev.rowid DESC LIMIT 1",
    )
    .bind(pipeline::ACCEPTANCE_DEPENDENCIES)
    .bind(analysis_id)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// 앞선 분석에서 재생할 것 — 열린 충돌을 먼저(그 분석에서도 얹히지 않았던 자리),
/// 승인 편집을 그 다음에 적용 순서대로.
async fn inherited_from(state: &AppState, previous: &str) -> Result<Vec<Inherited>, AppError> {
    let mut items = Vec::new();

    let conflicts: Vec<(String, String, i64, String, String, String, String, i64)> = sqlx::query_as(
        "SELECT edit_id, feature_key, scenario_index, request, source, before_json, mine_json, \
                mine_decided_at \
           FROM feature_doc_conflicts WHERE analysis_id = ? AND status = ? \
          ORDER BY created_at, rowid",
    )
    .bind(previous)
    .bind(status::OPEN)
    .fetch_all(&state.db)
    .await?;
    for (edit_id, key, index, request, source, before, mine, decided_at) in conflicts {
        items.push(Inherited {
            edit_id,
            feature_key: key,
            scenario_index: index as usize,
            request,
            source,
            before: doc_edit::sentences_from_json(&before)?,
            after: doc_edit::scenarios_from_json(&mine)?,
            decided_at,
        });
    }

    // 앞선 분석에서 **지금 서 있는** 편집만 이어받는다 — 거기서 복원으로 잘려 나간
    // 편집이 다음 재분석에서 되살아나면 복원이 한 분석짜리 거짓말이 된다(0014).
    let standing = crate::doc_history::standing_edits(state, previous).await?;
    let edits: Vec<(String, String, i64, String, String, String, String, Option<i64>)> = sqlx::query_as(
        "SELECT id, feature_key, scenario_index, request, source, before_json, after_json, \
                decided_at \
           FROM feature_doc_edits WHERE analysis_id = ? AND status = ? \
          ORDER BY created_at, rowid",
    )
    .bind(previous)
    .bind(doc_edit::status::APPROVED)
    .fetch_all(&state.db)
    .await?;
    for (id, key, index, request, source, before, after, decided_at) in edits {
        if !standing.contains(&id) {
            continue;
        }
        items.push(Inherited {
            edit_id: id,
            feature_key: key,
            scenario_index: index as usize,
            request,
            source,
            before: doc_edit::sentences_from_json(&before)?,
            after: doc_edit::scenarios_from_json(&after)?,
            decided_at: decided_at.unwrap_or(0),
        });
    }
    Ok(items)
}

#[allow(clippy::too_many_arguments)]
async fn insert_edit(
    state: &AppState,
    analysis_id: &str,
    feature_key: &str,
    scenario_index: usize,
    request: &str,
    before: &Sentences,
    after: &[Sentences],
    status: &str,
    source: &str,
    carried_from: Option<&str>,
    created_at: i64,
    decided_at: Option<i64>,
) -> Result<String, AppError> {
    let row_id = uuid::Uuid::new_v4().to_string();
    // 이 행이 **어느 복원 뒤에** 서는지(0014). 이월로 새 분석에 심는 경우 그 분석에는
    // 아직 복원이 없으므로 NULL 이다.
    let after_restore = crate::doc_history::current_restore(state, analysis_id, feature_key).await?;
    sqlx::query(
        "INSERT INTO feature_doc_edits \
           (id, analysis_id, feature_key, scenario_index, request, before_json, after_json, \
            status, source, carried_from, created_at, decided_at, after_restore) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&row_id)
    .bind(analysis_id)
    .bind(feature_key)
    .bind(scenario_index as i64)
    .bind(request)
    .bind(to_json(before)?)
    .bind(to_json(&after.to_vec())?)
    .bind(status)
    .bind(source)
    .bind(carried_from)
    .bind(created_at)
    .bind(decided_at)
    .bind(after_restore.as_deref())
    .execute(&state.db)
    .await?;
    Ok(row_id)
}

fn to_json<T: Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(|e| AppError::internal(e.to_string()))
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<ListView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let rows = sqlx::query_as::<_, ConflictRow>(
        "SELECT id, previous_analysis_id, feature_key, scenario_index, request, source, \
                mine_json, before_json, auto_json, mine_decided_at, status, merge_edit_id, \
                decided_at \
           FROM feature_doc_conflicts WHERE analysis_id = ? ORDER BY created_at, rowid",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await?;
    let doc = raw_document(&state, &id).await?;
    let mut conflicts = Vec::with_capacity(rows.len());
    for row in rows {
        conflicts.push(view(&state, &doc, row).await?);
    }
    Ok(Json(ListView {
        open: conflicts.iter().filter(|c| c.status == status::OPEN).count(),
        conflicts,
    }))
}

async fn one(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, conflict)): Path<(String, String)>,
) -> Result<Json<ConflictView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let row = conflict_row(&state, &id, &conflict).await?;
    let doc = raw_document(&state, &id).await?;
    Ok(Json(view(&state, &doc, row).await?))
}

async fn decide(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, conflict)): Path<(String, String)>,
    Json(req): Json<DecisionReq>,
) -> Result<Json<ConflictView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let row = open_row(&state, &id, &conflict).await?;
    let now = now_unix();

    let next = match req.decision.as_str() {
        "auto" => status::AUTO,
        "mine" => {
            let auto = doc_edit::sentences_from_json(&row.auto_json)?;
            let mine = doc_edit::scenarios_from_json(&row.mine_json)?;
            let slot = slot_of(&state, &id, &row.feature_key, row.scenario_index as usize, &auto).await?;
            insert_edit(
                &state,
                &id,
                &row.feature_key,
                slot,
                &row.request,
                &auto,
                &mine,
                doc_edit::status::APPROVED,
                SOURCE_USER_DIRECT,
                None,
                now,
                Some(now),
            )
            .await?;
            status::MINE
        }
        _ => return Err(AppError::BadRequest("알 수 없는 결정입니다".into())),
    };

    sqlx::query("UPDATE feature_doc_conflicts SET status = ?, decided_at = ? WHERE id = ?")
        .bind(next)
        .bind(now)
        .bind(&conflict)
        .execute(&state.db)
        .await?;

    let row = conflict_row(&state, &id, &conflict).await?;
    let doc = raw_document(&state, &id).await?;
    Ok(Json(view(&state, &doc, row).await?))
}

/// 두 문장을 합친 제안을 받는다.
async fn merge(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, conflict)): Path<(String, String)>,
    Json(req): Json<MergeReq>,
) -> Result<Json<ConflictView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let row = open_row(&state, &id, &conflict).await?;

    if let Some(edit) = row.merge_edit_id.as_deref() {
        if pending_proposal(&state, &id, edit).await?.is_some() {
            let doc = raw_document(&state, &id).await?;
            return Ok(Json(view(&state, &doc, row).await?));
        }
    }

    let auto = doc_edit::sentences_from_json(&row.auto_json)?;
    let mine = doc_edit::scenarios_from_json(&row.mine_json)?;
    let rejected = doc_edit::rejections(&state, &id, &row.feature_key).await?;
    let avoid = doc_edit::avoid_list(&rejected);

    let (provider, key) = match crate::llmkey::active_key_for_user(&state, &user.id).await? {
        Some((provider, key)) => (
            llm::Provider::parse(&provider).unwrap_or(llm::DEFAULT_PROVIDER),
            Some(key),
        ),
        None => (llm::DEFAULT_PROVIDER, None),
    };

    let language = crate::settings::analysis_language(&state.db, &id).await?;

    let answer = llm::ask(
        &state.http,
        state.config.doubles.llm,
        provider,
        key.as_deref(),
        Ask {
            system: SYSTEM_MERGE,
            language,
            user: prompt(&mine, &auto, req.keep_mine, &avoid),
            schema: doc_edit::schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub_merge(&mine, &auto, req.keep_mine, &avoid),
        },
    )
    .await
    .map_err(AppError::BadRequest)?;

    let merged = doc_edit::proposed(&answer.content).ok_or_else(|| {
        AppError::BadRequest("모델이 합친 문장을 읽을 수 없었습니다".into())
    })?;

    let slot = slot_of(&state, &id, &row.feature_key, row.scenario_index as usize, &auto).await?;
    let request = if req.keep_mine { MERGE_REQUEST_KEEP } else { MERGE_REQUEST };
    let edit_id = insert_edit(
        &state,
        &id,
        &row.feature_key,
        slot,
        request,
        &auto,
        &merged,
        doc_edit::status::PROPOSED,
        doc_edit::SOURCE_USER_LLM,
        None,
        now_unix(),
        None,
    )
    .await?;
    sqlx::query(
        "UPDATE feature_doc_edits SET model = ?, input_tokens = ?, output_tokens = ? WHERE id = ?",
    )
    .bind(&answer.model)
    .bind(answer.input_tokens)
    .bind(answer.output_tokens)
    .bind(&edit_id)
    .execute(&state.db)
    .await?;
    sqlx::query("UPDATE feature_doc_conflicts SET merge_edit_id = ? WHERE id = ?")
        .bind(&edit_id)
        .bind(&conflict)
        .execute(&state.db)
        .await?;

    let row = conflict_row(&state, &id, &conflict).await?;
    let doc = raw_document(&state, &id).await?;
    Ok(Json(view(&state, &doc, row).await?))
}

/// 합친 제안의 승인이 곧 결정이다. 거부는 제안만 버리고 충돌은 열린 채 둔다.
async fn decide_merge(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, conflict)): Path<(String, String)>,
    Json(req): Json<DecisionReq>,
) -> Result<Json<ConflictView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let row = open_row(&state, &id, &conflict).await?;
    let edit = match row.merge_edit_id.as_deref() {
        Some(edit) if pending_proposal(&state, &id, edit).await?.is_some() => edit.to_string(),
        _ => return Err(AppError::Conflict("결정할 합친 제안이 없습니다".into())),
    };
    let now = now_unix();

    match req.decision.as_str() {
        "approve" => {
            sqlx::query("UPDATE feature_doc_edits SET status = ?, decided_at = ? WHERE id = ?")
                .bind(doc_edit::status::APPROVED)
                .bind(now)
                .bind(&edit)
                .execute(&state.db)
                .await?;
            sqlx::query(
                "UPDATE feature_doc_conflicts SET status = ?, decided_at = ? WHERE id = ?",
            )
            .bind(status::MERGED)
            .bind(now)
            .bind(&conflict)
            .execute(&state.db)
            .await?;
        }
        "reject" => {
            sqlx::query(
                "UPDATE feature_doc_edits SET status = ?, reason = ?, decided_at = ? WHERE id = ?",
            )
            .bind(doc_edit::status::REJECTED)
            .bind(REJECT_REASON)
            .bind(now)
            .bind(&edit)
            .execute(&state.db)
            .await?;
            sqlx::query("UPDATE feature_doc_conflicts SET merge_edit_id = NULL WHERE id = ?")
                .bind(&conflict)
                .execute(&state.db)
                .await?;
        }
        _ => return Err(AppError::BadRequest("알 수 없는 결정입니다".into())),
    }

    let row = conflict_row(&state, &id, &conflict).await?;
    let doc = raw_document(&state, &id).await?;
    Ok(Json(view(&state, &doc, row).await?))
}

/// 사람이 결정하는 시점에 자동 문장이 서 있는 자리. 이월된 편집이 앞자리를 늘렸을 수
/// 있으므로 저장된 번호가 아니라 **지금 보는 문서**에서 그 문장을 찾는다.
async fn slot_of(
    state: &AppState,
    analysis_id: &str,
    feature_key: &str,
    stored: usize,
    auto: &Sentences,
) -> Result<usize, AppError> {
    let doc = doc_edit::document_of(state, analysis_id).await?;
    let list = doc_edit::scenarios_of(&doc, feature_key)?;
    if let Some(slot) = list.iter().position(|s| same(s, auto)) {
        return Ok(slot);
    }
    if stored < list.len() {
        return Ok(stored);
    }
    Err(AppError::Conflict("자동 문장이 이제 그 자리에 없습니다".into()))
}

async fn conflict_row(state: &AppState, id: &str, conflict: &str) -> Result<ConflictRow, AppError> {
    sqlx::query_as::<_, ConflictRow>(
        "SELECT id, previous_analysis_id, feature_key, scenario_index, request, source, \
                mine_json, before_json, auto_json, mine_decided_at, status, merge_edit_id, \
                decided_at \
           FROM feature_doc_conflicts WHERE id = ? AND analysis_id = ?",
    )
    .bind(conflict)
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)
}

async fn open_row(state: &AppState, id: &str, conflict: &str) -> Result<ConflictRow, AppError> {
    let row = conflict_row(state, id, conflict).await?;
    if row.status != status::OPEN {
        return Err(AppError::Conflict("이미 결정된 충돌입니다".into()));
    }
    Ok(row)
}

/// 아직 결정되지 않은 합친 제안 — `(after_json,)`.
async fn pending_proposal(state: &AppState, id: &str, edit: &str) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT after_json FROM feature_doc_edits WHERE id = ? AND analysis_id = ? AND status = ?",
    )
    .bind(edit)
    .bind(id)
    .bind(doc_edit::status::PROPOSED)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.map(|(after,)| after))
}

/// 워커가 넣은 자동 문서 그대로 — feature 이름은 겹쳐 읽기와 무관하다.
async fn raw_document(state: &AppState, analysis_id: &str) -> Result<Value, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(analysis_id)
    .bind(pipeline::ACCEPTANCE_DEPENDENCIES)
    .fetch_optional(&state.db)
    .await?;
    Ok(row
        .and_then(|(content,)| serde_json::from_str(&content).ok())
        .unwrap_or_else(|| json!({})))
}

async fn view(state: &AppState, doc: &Value, row: ConflictRow) -> Result<ConflictView, AppError> {
    let auto = doc_edit::sentences_from_json(&row.auto_json)?;
    let merge_proposal = match row.merge_edit_id.as_deref() {
        Some(edit) => {
            let found: Option<(String, String, String)> = sqlx::query_as(
                "SELECT id, status, after_json FROM feature_doc_edits WHERE id = ?",
            )
            .bind(edit)
            .fetch_optional(&state.db)
            .await?;
            match found {
                Some((id, status, after_json)) => {
                    let after = doc_edit::scenarios_from_json(&after_json)?;
                    let (removed, added) = doc_edit::lines(&auto, &after);
                    Some(MergeProposalView {
                        id,
                        status,
                        changed_lines: removed.len() + added.len(),
                        after,
                        removed,
                        added,
                    })
                }
                None => None,
            }
        }
        None => None,
    };
    Ok(ConflictView {
        id: row.id,
        feature_name: diff::feature_name(doc, &row.feature_key)
            .unwrap_or_else(|| row.feature_key.clone()),
        feature_key: row.feature_key,
        scenario_index: row.scenario_index as usize,
        status: row.status,
        source: row.source,
        request: row.request,
        mine: doc_edit::scenarios_from_json(&row.mine_json)?,
        before: doc_edit::sentences_from_json(&row.before_json)?,
        auto,
        mine_decided_at: row.mine_decided_at,
        previous_analysis_id: row.previous_analysis_id,
        merge_proposal,
        decided_at: row.decided_at,
    })
}

fn prompt(mine: &[Sentences], auto: &Sentences, keep_mine: bool, avoid: &[String]) -> String {
    let mut text = String::new();
    text.push_str("The reader's version, as it stood after their edit:\n");
    for (i, s) in mine.iter().enumerate() {
        text.push_str(&format!(
            "{}. given: {}\n   when: {}\n   then: {}\n",
            i + 1,
            s.given,
            s.when,
            s.then
        ));
    }
    text.push_str("\nThe automatic analysis now says, after the code changed:\n");
    text.push_str(&format!(
        "given: {}\nwhen: {}\nthen: {}\n",
        auto.given, auto.when, auto.then
    ));
    if keep_mine {
        text.push_str(
            "\nThe reader asked that their own added wording be kept verbatim — do not reword \
             it; only add the new rule.\n",
        );
    }
    if !avoid.is_empty() {
        text.push_str("\nMerged sentences this reader already rejected — do not repeat them:\n");
        for rejected in avoid {
            text.push_str("- ");
            text.push_str(rejected);
            text.push('\n');
        }
    }
    text
}

/// 결정적 합치기: 사람의 마지막 문장을 그대로 두고 이번 자동 결과의 규칙을 뒤에 잇는다.
/// `doc_edit::stub_edit` 과 같은 이유로 회피도 스텁의 규칙이다 — 이미 버린 문면이 나오면
/// 한 겹 더 고쳐 쓴다.
fn stub_merge(mine: &[Sentences], auto: &Sentences, _keep_mine: bool, avoid: &[String]) -> Value {
    let last = mine.last().cloned().unwrap_or_else(|| auto.clone());
    let base = format!("{} — 그리고 {}", last.then, auto.then);
    let mut then = base.clone();
    let mut round = 1;
    while avoid.iter().any(|rejected| rejected == &then) {
        round += 1;
        then = format!("{base} ({round}번째로 다시 합친 문장)");
    }
    let mut scenarios: Vec<Value> = mine[..mine.len().saturating_sub(1)]
        .iter()
        .map(|s| json!({ "given": s.given, "when": s.when, "then": s.then }))
        .collect();
    scenarios.push(json!({ "given": auto.given, "when": auto.when, "then": then }));
    json!({ "scenarios": scenarios })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sentences(then: &str) -> Sentences {
        Sentences {
            given: "카드가 없는 사용자가".to_string(),
            when: "만료된 카드를 저장하면".to_string(),
            then: then.to_string(),
        }
    }

    #[test]
    fn whitespace_alone_does_not_make_two_sentences_differ() {
        assert!(same(&sentences("저장되지  않습니다"), &sentences("저장되지 않습니다")));
        assert!(!same(&sentences("저장됩니다"), &sentences("저장되지 않습니다")));
    }

    #[test]
    fn the_stub_keeps_the_readers_sentence_and_adds_the_new_rule() {
        let mine = [sentences("저장되지 않고 이유를 알려 줍니다")];
        let auto = sentences("만료일을 고치라고 안내합니다");
        let merged = doc_edit::proposed(&stub_merge(&mine, &auto, false, &[])).unwrap();
        assert_eq!(merged.len(), 1);
        assert!(merged[0].then.contains(&mine[0].then));
        assert!(merged[0].then.contains(&auto.then));
        assert_eq!(merged[0].given, auto.given);
    }

    #[test]
    fn the_stub_avoids_a_rejected_merge() {
        let mine = [sentences("저장되지 않고 이유를 알려 줍니다")];
        let auto = sentences("만료일을 고치라고 안내합니다");
        let first = doc_edit::proposed(&stub_merge(&mine, &auto, false, &[])).unwrap();
        let again =
            doc_edit::proposed(&stub_merge(&mine, &auto, false, &[first[0].then.clone()])).unwrap();
        assert_ne!(again[0].then, first[0].then);
        assert!(again[0].then.contains(&mine[0].then));
    }

    #[test]
    fn a_multi_scenario_edit_keeps_its_earlier_scenarios() {
        let mine = [sentences("원래 문장"), sentences("덧붙인 문장")];
        let auto = sentences("새 규칙");
        let merged = doc_edit::proposed(&stub_merge(&mine, &auto, false, &[])).unwrap();
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].then, "원래 문장");
        assert!(merged[1].then.contains("덧붙인 문장") && merged[1].then.contains("새 규칙"));
    }

    #[test]
    fn the_prompt_carries_both_sides_and_the_rejections() {
        let text = prompt(
            &[sentences("내 문장")],
            &sentences("자동 문장"),
            true,
            &["버린 문장".to_string()],
        );
        assert!(text.contains("내 문장") && text.contains("자동 문장"));
        assert!(text.contains("verbatim"));
        assert!(text.contains("버린 문장"));
        assert!(!prompt(&[sentences("a")], &sentences("b"), false, &[]).contains("rejected"));
    }
}

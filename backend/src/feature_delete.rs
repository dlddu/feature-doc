//! AC3.3: feature 문서의 삭제와 보존.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::state::AppState;
use crate::util::now_unix;

/// 되돌릴 수 있는 기간. 「일정 기간」의 값이고, 지운 시점의 행에 기한으로 적힌다.
pub const RETENTION_DAYS: i64 = 30;
const DAY_SECS: i64 = 24 * 60 * 60;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/analyses/{id}/features/deletions",
            get(list).post(delete),
        )
        .route(
            "/api/analyses/{id}/features/deletions/{deletion}/restore",
            post(restore),
        )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionView {
    id: String,
    key: String,
    name: String,
    reason: Option<String>,
    deleted_at: i64,
    restore_until: i64,
    restored_at: Option<i64>,
    restorable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListView {
    retention_days: i64,
    deletions: Vec<DeletionView>,
}

#[derive(Deserialize)]
struct DeleteReq {
    key: String,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DeletionRow {
    id: String,
    feature_key: String,
    name: String,
    reason: Option<String>,
    deleted_at: i64,
    restore_until: i64,
    restored_at: Option<i64>,
}

/// 재발견 표시에 실리는 앞선 삭제.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviousDeletion {
    pub reason: Option<String>,
    pub deleted_at: i64,
    pub analysis_id: String,
}

fn restorable(row: &DeletionRow, now: i64) -> bool {
    row.restored_at.is_none() && now <= row.restore_until
}

fn view(row: DeletionRow, now: i64) -> DeletionView {
    let restorable = restorable(&row, now);
    DeletionView {
        id: row.id,
        key: row.feature_key,
        name: row.name,
        reason: row.reason,
        deleted_at: row.deleted_at,
        restore_until: row.restore_until,
        restored_at: row.restored_at,
        restorable,
    }
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<ListView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let now = now_unix();
    let rows = open_rows(&state, &id).await?;
    Ok(Json(ListView {
        retention_days: RETENTION_DAYS,
        deletions: rows.into_iter().map(|r| view(r, now)).collect(),
    }))
}

async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(req): Json<DeleteReq>,
) -> Result<Json<DeletionView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let key = req.key.trim().to_string();
    if key.is_empty() {
        return Err(AppError::BadRequest("어느 기능을 지울지 있어야 합니다".into()));
    }
    let reason = req
        .reason
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty());

    let doc = current_document(&state, &id).await?;
    let name = crate::diff::feature_name(&doc, &key).ok_or(AppError::NotFound)?;

    let now = now_unix();
    let row_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO feature_deletions \
           (id, analysis_id, feature_key, name, reason, deleted_at, restore_until) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&row_id)
    .bind(&id)
    .bind(&key)
    .bind(&name)
    .bind(reason.as_deref())
    .bind(now)
    .bind(restore_until(now))
    .execute(&state.db)
    .await?;

    crate::audit::record(&state.db, Some(&user.id), "analysis.feature_deletion", Some("delete"))
        .await;

    let row = row_of(&state, &id, &row_id).await?;
    Ok(Json(view(row, now)))
}

/// 보관 기간 안의 열린 삭제만 되돌린다. 이미 되돌린 것은 409, 기한이 지난 것도 409 —
/// 둘 다 「지금은 할 수 없는 일」이지 「없는 것」이 아니다.
async fn restore(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, deletion)): Path<(String, String)>,
) -> Result<Json<DeletionView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let row = row_of(&state, &id, &deletion).await?;
    if row.restored_at.is_some() {
        return Err(AppError::Conflict("이미 복구한 기능입니다".into()));
    }
    let now = now_unix();
    if !restorable(&row, now) {
        return Err(AppError::Conflict(
            "보관 기간이 지나 복구할 수 없습니다".into(),
        ));
    }
    sqlx::query("UPDATE feature_deletions SET restored_at = ? WHERE id = ? AND restored_at IS NULL")
        .bind(now)
        .bind(&deletion)
        .execute(&state.db)
        .await?;

    crate::audit::record(&state.db, Some(&user.id), "analysis.feature_deletion", Some("restore"))
        .await;

    let row = row_of(&state, &id, &deletion).await?;
    Ok(Json(view(row, now)))
}

/// 열린 삭제가 있는 feature 를 문서에서 가린다. 확정된 추가 겹침 **뒤**, 편집 겹침 **앞**에
/// 불러야 한다 — 더해진 feature 도 지울 수 있고, 가려진 feature 의 편집은 얹을 자리가 없다.
pub async fn overlay(state: &AppState, analysis_id: &str, doc: &mut Value) -> Result<(), AppError> {
    let rows = open_rows(state, analysis_id).await?;
    if rows.is_empty() {
        return Ok(());
    }
    let deleted: Vec<String> = rows.into_iter().map(|r| r.feature_key).collect();
    hide(doc, &deleted);
    Ok(())
}

fn hide(doc: &mut Value, deleted: &[String]) {
    let Some(features) = doc.get_mut("features").and_then(Value::as_array_mut) else {
        return;
    };
    features.retain(|f| {
        !f.get("key")
            .and_then(Value::as_str)
            .is_some_and(|k| deleted.iter().any(|d| d == k))
    });
}

/// 같은 대상(사용자·저장소·브랜치)의 앞선 분석에서 이 키를 지웠고 아직 되돌리지 않은
/// 가장 최근 삭제. 후보 거부 이월과 같은 순서 규칙 — unix 초가 같으면 `rowid` 로 가른다.
#[allow(clippy::too_many_arguments)]
pub async fn previous_deletion(
    state: &AppState,
    user_id: &str,
    id: &str,
    owner: &str,
    name: &str,
    branch: &str,
    created_at: i64,
    key: &str,
) -> Result<Option<PreviousDeletion>, AppError> {
    let row: Option<(Option<String>, i64, String)> = sqlx::query_as(
        "SELECT d.reason, d.deleted_at, prev.id \
           FROM feature_deletions d \
           JOIN analyses prev ON prev.id = d.analysis_id \
          WHERE d.feature_key = ? AND d.restored_at IS NULL \
            AND prev.user_id = ? AND prev.repo_owner = ? AND prev.repo_name = ? \
            AND prev.branch = ? AND prev.id != ? \
            AND (prev.created_at < ? \
                 OR (prev.created_at = ? \
                     AND prev.rowid < (SELECT rowid FROM analyses WHERE id = ?))) \
          ORDER BY prev.created_at DESC, prev.rowid DESC LIMIT 1",
    )
    .bind(key)
    .bind(user_id)
    .bind(owner)
    .bind(name)
    .bind(branch)
    .bind(id)
    .bind(created_at)
    .bind(created_at)
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    Ok(row.map(|(reason, deleted_at, analysis_id)| PreviousDeletion {
        reason,
        deleted_at,
        analysis_id,
    }))
}

fn restore_until(deleted_at: i64) -> i64 {
    deleted_at + RETENTION_DAYS * DAY_SECS
}

async fn open_rows(state: &AppState, analysis_id: &str) -> Result<Vec<DeletionRow>, AppError> {
    Ok(sqlx::query_as::<_, DeletionRow>(
        "SELECT id, feature_key, name, reason, deleted_at, restore_until, restored_at \
           FROM feature_deletions \
          WHERE analysis_id = ? AND restored_at IS NULL \
          ORDER BY deleted_at DESC, rowid DESC",
    )
    .bind(analysis_id)
    .fetch_all(&state.db)
    .await?)
}

async fn row_of(state: &AppState, analysis_id: &str, id: &str) -> Result<DeletionRow, AppError> {
    sqlx::query_as::<_, DeletionRow>(
        "SELECT id, feature_key, name, reason, deleted_at, restore_until, restored_at \
           FROM feature_deletions WHERE analysis_id = ? AND id = ?",
    )
    .bind(analysis_id)
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)
}

/// 사람이 지금 보는 문서 — 편집 겹침은 이름·키를 바꾸지 않으므로 여기서는 얹지 않는다.
async fn current_document(state: &AppState, analysis_id: &str) -> Result<Value, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(analysis_id)
    .bind(crate::pipeline::ACCEPTANCE_DEPENDENCIES)
    .fetch_optional(&state.db)
    .await?;
    let content = row.ok_or(AppError::NotFound)?.0;
    let mut doc: Value = serde_json::from_str(&content)
        .map_err(|_| AppError::BadRequest("stored document is unreadable".into()))?;
    crate::feature_add::overlay(state, analysis_id, &mut doc).await?;
    overlay(state, analysis_id, &mut doc).await?;
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(deleted_at: i64, restored_at: Option<i64>) -> DeletionRow {
        DeletionRow {
            id: "d".into(),
            feature_key: "src/a.rs".into(),
            name: "a".into(),
            reason: None,
            deleted_at,
            restore_until: restore_until(deleted_at),
            restored_at,
        }
    }

    #[test]
    fn restorable_inside_the_window_and_not_after_it() {
        let deleted = 1_000_000;
        assert!(restorable(&row(deleted, None), deleted));
        assert!(restorable(&row(deleted, None), deleted + RETENTION_DAYS * DAY_SECS));
        assert!(!restorable(&row(deleted, None), deleted + RETENTION_DAYS * DAY_SECS + 1));
        assert!(!restorable(&row(deleted, Some(deleted + 5)), deleted + 5));
    }

    #[test]
    fn the_window_is_a_fact_of_the_deletion_not_of_now() {
        assert_eq!(restore_until(10), 10 + RETENTION_DAYS * DAY_SECS);
        assert!(RETENTION_DAYS > 0);
    }

    #[test]
    fn hiding_removes_only_the_deleted_keys_and_keeps_order() {
        let mut doc = json!({ "features": [
            { "key": "src/a.rs", "name": "a" },
            { "key": "added:1", "name": "b" },
            { "key": "src/c.rs", "name": "c" },
        ]});
        hide(&mut doc, &["added:1".to_string(), "nope".to_string()]);
        let keys: Vec<&str> = doc["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["key"].as_str().unwrap())
            .collect();
        assert_eq!(keys, vec!["src/a.rs", "src/c.rs"]);

        let mut none = json!({ "other": 1 });
        hide(&mut none, &["src/a.rs".to_string()]);
        assert_eq!(none, json!({ "other": 1 }));
    }
}

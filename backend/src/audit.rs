//! Append-only audit trail for credential-touching actions (AC4.3).
//!
//! `detail` must only ever carry non-secret context (provider name, account login,
//! row id) — never a key, token, or ciphertext.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::state::AppState;
use crate::util::now_unix;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/audit", get(list))
}

/// Records an action. Best-effort: a failure to write the audit row is logged but
/// must not fail the user's request.
pub async fn record(db: &SqlitePool, user_id: Option<&str>, action: &str, detail: Option<&str>) {
    let res = sqlx::query(
        "INSERT INTO audit_log (id, user_id, action, detail, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(action)
    .bind(detail)
    .bind(now_unix())
    .execute(db)
    .await;
    if let Err(e) = res {
        tracing::warn!(action, "failed to write audit entry: {e}");
    }
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct AuditView {
    id: String,
    action: String,
    detail: Option<String>,
    created_at: i64,
}

/// The current user's own audit history (AC4.3: usage history is user-visible).
async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<AuditView>>, AppError> {
    let rows = sqlx::query_as::<_, AuditView>(
        "SELECT id, action, detail, created_at FROM audit_log \
         WHERE user_id = ? ORDER BY created_at DESC, id DESC LIMIT 200",
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

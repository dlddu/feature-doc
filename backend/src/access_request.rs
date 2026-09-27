//! Asking the owner to share a repository you cannot see (AC4.10).
//!
//! Nothing about who owns the target is ever fetched, stored, or returned here, so
//! there is no owner identity on the requester's path to leak — resolving the owner
//! at request time is what would put that value on the requester's path.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::state::AppState;
use crate::util::now_unix;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/analyses/{id}/access-request", post(create))
}

/// The one shape this route ever answers with.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessRequestView {
    pub status: String,
}

const REQUESTED: &str = "requested";

async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<AccessRequestView>), AppError> {
    sqlx::query(
        "INSERT OR IGNORE INTO access_requests \
           (id, analysis_id, requester_user_id, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&id)
    .bind(&user.id)
    .bind(now_unix())
    .execute(&state.db)
    .await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(AccessRequestView {
            status: REQUESTED.into(),
        }),
    ))
}

//! Asking the owner to share a repository you cannot see (AC4.10).
//!
//! The screen behind this route is the one dead end in `JRN-understand-feature`:
//! a link to somebody else's analysis, which `analysis::owned_analysis` answers
//! with `404` rather than `403` so the API never confirms that an id exists
//! (AC4.7). AC4.10 puts a way forward at that dead end without reopening the door
//! it closed — the request only *prompts* the owner to widen their AC4.1 selection.
//!
//! Two properties do all the work here, and both are about what this handler does
//! **not** do:
//!
//!  * It never looks the target up. Reading `analyses` first would give the
//!    handler two paths, and two paths eventually differ — in a status code, a
//!    body, a latency. One path cannot leak what it never learned, so the reply is
//!    the same constant whether the id belongs to someone else or to nobody at all
//!    ("실재하는 남의 대상과 실재하지 않는 대상에 대한 요청의 응답·화면이 서로 구분되지
//!    않는다", test/04 시나리오 15).
//!  * It never resolves the owner. Nothing about who owns the target is fetched,
//!    stored, or returned, so there is no owner identity on the requester's path to
//!    leak. Deciding whom to notify is the notification channel's job, at the point
//!    where a notification actually goes out — that channel does not exist yet, and
//!    AC4.10's 검증 방법 defers its verification to the slice that builds it.
//!
//! What is left is one row (migration `0016`). The screen tells the requester the
//! request was received; that sentence is only true because the row outlives the
//! click.

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

/// The one shape this route ever answers with. It carries no field that could
/// differ between an id that exists and one that does not — not even an id of its
/// own, because a generated row id would be present on the first request and
/// absent on a repeat.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessRequestView {
    pub status: String,
}

const REQUESTED: &str = "requested";

/// `202` for every authenticated caller: the request is recorded, and what happens
/// next is the owner's to decide.
///
/// `INSERT OR IGNORE` makes a repeat click idempotent rather than a second row —
/// the requester sees the same screen either way, and the owner is not asked twice
/// by the same person for the same link.
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

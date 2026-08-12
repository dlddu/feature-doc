//! The queue protocol the analysis worker speaks (AC4.5).
//!
//! ## Why the worker talks HTTP instead of opening the database
//!
//! The database is SQLite on a ReadWriteOnce volume, and `db.rs` / the Deployment
//! pin that to exactly one writer (`replicas: 1`, `strategy: Recreate`). A worker
//! that opened the same file would break that invariant the moment it scaled past
//! one. So the worker owns *no* persistence: it claims work and reports progress
//! through these routes, and the API remains the single writer. AC4.5's
//! separation ("API 워크로드"/"백그라운드 워커 워크로드") and horizontal scaling
//! both hold, without a second datastore.
//!
//! ## Trust boundary
//!
//! These routes are not user-facing: they authenticate with a shared worker token
//! ([`WorkerAuth`]) rather than a session cookie, and they are refused outright
//! when no token is configured — a deployment that has not been given the secret
//! never exposes them. Claiming hands the worker a short-lived GitHub installation
//! token, minted on demand and never stored, exactly as the API mints one for its
//! own calls (AC4.1); it is scoped to the one repository the claimed job targets,
//! which is what keeps per-user isolation intact (AC4.7).

use axum::extract::{Path, State};
use axum::http::request::Parts;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::extract::FromRequestParts;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::pipeline::{self, stage_status, status};
use crate::state::AppState;
use crate::util::now_unix;
use crate::{github_app, installations};

/// How long a claim is held before another worker may reclaim the job. Long enough
/// to cover a stage plus the worker's HTTP timeouts; short enough that a killed
/// worker's job returns to the queue in bounded time.
pub const LEASE_SECONDS: i64 = 180;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/internal/analyses/claim", post(claim))
        .route("/internal/analyses/{id}/heartbeat", post(heartbeat))
        .route("/internal/analyses/{id}/stages/{key}", post(report_stage))
        .route("/internal/analyses/{id}/finish", post(finish))
}

// ── authentication ────────────────────────────────────────────────────────────

/// Proof that the caller presented the configured worker token.
///
/// An unset token is not "allow everything" — it is "these routes do not exist for
/// this deployment", so production stays closed until the secret is provisioned.
pub struct WorkerAuth;

impl FromRequestParts<AppState> for WorkerAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let configured = state.config.worker_token.as_bytes();
        if configured.is_empty() {
            return Err(AppError::Unauthorized);
        }
        let presented = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        if constant_time_eq(presented.as_bytes(), configured) {
            Ok(WorkerAuth)
        } else {
            Err(AppError::Unauthorized)
        }
    }
}

/// Length-independent, data-independent comparison — the token is a secret, so a
/// short-circuiting `==` would leak its prefix through timing.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut diff = (a.len() ^ b.len()) as u8;
    let n = a.len().max(b.len());
    for i in 0..n {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0
}

// ── claim ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaimReq {
    /// The worker's identity (pod name in k8s). Recorded on the job so an operator
    /// can see which worker holds it.
    worker_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClaimView {
    id: String,
    repo_owner: String,
    repo_name: String,
    branch: String,
    /// Stage keys the worker is expected to execute, in order. Today this is just
    /// `fetch`; later slices widen it as their stages become implemented.
    executable_stages: Vec<String>,
    lease_expires_at: i64,
    /// Short-lived GitHub installation token for this job's repository. `None` in
    /// stub mode (nothing to call). Never persisted, never logged.
    installation_token: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ClaimRow {
    id: String,
    user_id: String,
    installation_id: i64,
    repo_owner: String,
    repo_name: String,
    branch: String,
}

/// Atomically takes the oldest claimable job, or answers `204` when the queue is
/// empty.
///
/// Claimable means queued, **or** running under a lease that has expired — that
/// second arm is what returns a killed worker's job to the queue (test/04
/// scenario 7).
///
/// What makes this safe under N workers: SQLite serialises writers, so two racing
/// claims execute one after the other, and the *subquery* is re-evaluated inside
/// each write. The loser therefore never re-selects the row the winner just moved
/// out of `queued`; it picks the next eligible job, or none. The repeated
/// `status`/lease predicate on the `UPDATE` itself is belt-and-braces for the day
/// this runs on an engine with weaker write serialisation — measured to be
/// redundant today (removing it alone breaks no test; removing the subquery
/// filter breaks `concurrent_workers_take_disjoint_jobs`).
async fn claim(
    State(state): State<AppState>,
    _auth: WorkerAuth,
    Json(req): Json<ClaimReq>,
) -> Result<Response, AppError> {
    let now = now_unix();
    let lease_until = now + LEASE_SECONDS;

    let row: Option<ClaimRow> = sqlx::query_as(
        "UPDATE analyses \
           SET status = ?, claimed_by = ?, claimed_at = ?, lease_expires_at = ?, \
               started_at = COALESCE(started_at, ?) \
         WHERE id = ( \
                 SELECT id FROM analyses \
                  WHERE status = ? \
                     OR (status = ? AND lease_expires_at IS NOT NULL AND lease_expires_at < ?) \
                  ORDER BY created_at, id LIMIT 1 \
               ) \
           AND (status = ? \
                OR (status = ? AND lease_expires_at IS NOT NULL AND lease_expires_at < ?)) \
     RETURNING id, user_id, installation_id, repo_owner, repo_name, branch",
    )
    .bind(status::RUNNING)
    .bind(&req.worker_id)
    .bind(now)
    .bind(lease_until)
    .bind(now)
    .bind(status::QUEUED)
    .bind(status::RUNNING)
    .bind(now)
    .bind(status::QUEUED)
    .bind(status::RUNNING)
    .bind(now)
    .fetch_optional(&state.db)
    .await?;

    let Some(job) = row else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    };

    tracing::info!(
        analysis_id = %job.id,
        worker_id = %req.worker_id,
        "analysis claimed"
    );

    // Mint the job-scoped installation token here rather than storing one anywhere
    // (AC4.1/AC4.3). A job whose installation has since been removed simply gets
    // no token and its fetch stage fails with a clear reason.
    let installation_token = match installations::get_for_user(&state.db, &job.user_id).await? {
        Some(inst) if inst.installation_id == job.installation_id => {
            match github_app::mint_installation_token(&state, job.installation_id).await {
                Ok(t) => Some(t.token),
                Err(e) => {
                    tracing::warn!(analysis_id = %job.id, "installation token unavailable: {e:?}");
                    None
                }
            }
        }
        _ => None,
    };

    Ok(Json(ClaimView {
        id: job.id,
        repo_owner: job.repo_owner,
        repo_name: job.repo_name,
        branch: job.branch,
        executable_stages: vec![pipeline::FETCH.to_string()],
        lease_expires_at: lease_until,
        installation_token,
    })
    .into_response())
}

// ── lease + progress ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerIdReq {
    worker_id: String,
}

/// Extends the caller's lease. Only the holder may extend it.
async fn heartbeat(
    State(state): State<AppState>,
    _auth: WorkerAuth,
    Path(id): Path<String>,
    Json(req): Json<WorkerIdReq>,
) -> Result<StatusCode, AppError> {
    let now = now_unix();
    let res = sqlx::query(
        "UPDATE analyses SET lease_expires_at = ? \
          WHERE id = ? AND claimed_by = ? AND status = ?",
    )
    .bind(now + LEASE_SECONDS)
    .bind(&id)
    .bind(&req.worker_id)
    .bind(status::RUNNING)
    .execute(&state.db)
    .await?;

    if res.rows_affected() == 1 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::Conflict("lease no longer held".into()))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StageReq {
    worker_id: String,
    status: String,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// Records a stage transition. Rejected unless the caller still holds the lease,
/// so a worker that was reclaimed cannot overwrite its successor's progress.
async fn report_stage(
    State(state): State<AppState>,
    _auth: WorkerAuth,
    Path((id, key)): Path<(String, String)>,
    Json(req): Json<StageReq>,
) -> Result<StatusCode, AppError> {
    if pipeline::stage(&key).is_none() {
        return Err(AppError::BadRequest("unknown pipeline stage".into()));
    }
    if !stage_status::is_reportable(&req.status) {
        return Err(AppError::BadRequest("unknown stage status".into()));
    }
    require_lease(&state, &id, &req.worker_id).await?;

    let now = now_unix();
    let started_at = (req.status == stage_status::RUNNING).then_some(now);
    let finished_at =
        matches!(req.status.as_str(), stage_status::SUCCEEDED | stage_status::FAILED)
            .then_some(now);

    let res = sqlx::query(
        "UPDATE analysis_stages \
            SET status = ?, \
                detail = COALESCE(?, detail), \
                error = ?, \
                started_at = COALESCE(started_at, ?), \
                finished_at = ? \
          WHERE analysis_id = ? AND key = ?",
    )
    .bind(&req.status)
    .bind(req.detail.as_deref())
    .bind(req.error.as_deref())
    .bind(started_at)
    .bind(finished_at)
    .bind(&id)
    .bind(&key)
    .execute(&state.db)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FinishReq {
    worker_id: String,
    /// `awaiting_pipeline` (every implemented stage ran) or `failed`.
    status: String,
    #[serde(default)]
    error: Option<String>,
}

/// Closes out a claimed job and releases the lease.
async fn finish(
    State(state): State<AppState>,
    _auth: WorkerAuth,
    Path(id): Path<String>,
    Json(req): Json<FinishReq>,
) -> Result<StatusCode, AppError> {
    if !matches!(
        req.status.as_str(),
        status::AWAITING_PIPELINE | status::FAILED
    ) {
        return Err(AppError::BadRequest("unknown terminal status".into()));
    }
    require_lease(&state, &id, &req.worker_id).await?;

    sqlx::query(
        "UPDATE analyses \
            SET status = ?, error = ?, finished_at = ?, lease_expires_at = NULL \
          WHERE id = ? AND claimed_by = ?",
    )
    .bind(&req.status)
    .bind(req.error.as_deref())
    .bind(now_unix())
    .bind(&id)
    .bind(&req.worker_id)
    .execute(&state.db)
    .await?;

    tracing::info!(analysis_id = %id, worker_id = %req.worker_id, status = %req.status, "analysis finished");
    Ok(StatusCode::NO_CONTENT)
}

/// 409s unless `worker_id` currently holds an unexpired lease on `id`.
async fn require_lease(state: &AppState, id: &str, worker_id: &str) -> Result<(), AppError> {
    let held: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM analyses \
          WHERE id = ? AND claimed_by = ? AND status = ? \
            AND lease_expires_at IS NOT NULL AND lease_expires_at >= ?",
    )
    .bind(id)
    .bind(worker_id)
    .bind(status::RUNNING)
    .bind(now_unix())
    .fetch_optional(&state.db)
    .await?;

    held.map(|_| ())
        .ok_or_else(|| AppError::Conflict("lease no longer held".into()))
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn constant_time_eq_matches_equality() {
        assert!(constant_time_eq(b"token", b"token"));
        assert!(!constant_time_eq(b"token", b"tokeN"));
        assert!(!constant_time_eq(b"token", b"token-longer"));
        assert!(!constant_time_eq(b"", b"x"));
        assert!(constant_time_eq(b"", b""));
    }
}

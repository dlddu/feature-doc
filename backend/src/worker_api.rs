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
use crate::{analysis, github_app, installations};

/// How long a claim is held before another worker may reclaim the job. Long enough
/// to cover a stage plus the worker's HTTP timeouts; short enough that a killed
/// worker's job returns to the queue in bounded time.
pub const LEASE_SECONDS: i64 = 180;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/internal/analyses/claim", post(claim))
        .route("/internal/analyses/{id}/heartbeat", post(heartbeat))
        .route("/internal/analyses/{id}/stages/{key}", post(report_stage))
        .route("/internal/analyses/{id}/documents/{kind}", post(submit_document))
        // feature key 가 경로 구분자를 품으므로 결과는 경로가 아니라 본문으로 온다.
        .route("/internal/analyses/{id}/dependencies", post(submit_dependencies))
        .route("/internal/analyses/{id}/finish", post(finish))
}

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
    /// Stage keys the worker is expected to execute, in order.
    ///
    /// This is also where the two human gates live: they are properties of the queue
    /// rather than rules each worker is trusted to remember. A worker only runs keys
    /// it both knows and was offered.
    ///
    /// A stage that already **succeeded** is not offered again. That is what makes
    /// the post-approval re-queue safe: stage 4 opens without stages 2-3 re-running
    /// their LLM calls and overwriting the very document the user approved. The rule
    /// is spelled out in [`offered_stages`].
    executable_stages: Vec<String>,
    /// Stage 2's stored document, carried when stage 3 is offered **without** stage 2.
    /// `None` whenever stage 2 is offered too (the worker then uses this pass's).
    cross_cutting_document: Option<serde_json::Value>,
    /// The patterns the reviewer approved (AC1.3), when they have. Stage 4's input,
    /// carried on the claim so the worker needs no second round-trip — and so the
    /// gate and the input come from the same read of the same row.
    approved_patterns: Vec<String>,
    /// Stage 5's input and its gate, carried the same way and for the same reason as
    /// `approved_patterns`.
    approved_candidates: Vec<CandidateRef>,
    /// Carried for the same reason as the two above, and empty when nobody asked —
    /// which is what makes "아무도 요청하지 않았으면 아무것도 돌지 않는다" a property
    /// of the queue rather than a rule the worker remembers.
    dependency_requests: Vec<CandidateRef>,
    lease_expires_at: i64,
    /// Short-lived GitHub installation token for this job's repository. `None` in
    /// stub mode (nothing to call). Never persisted, never logged.
    installation_token: Option<String>,
    llm_provider: Option<String>,
    /// The owner's active LLM key, unsealed for this job only (AC1.2~AC1.4 stages).
    /// `None` when the user has no active key — the stage then fails with a clear
    /// reason instead of the worker inventing a result. Never persisted, never logged.
    llm_api_key: Option<String>,
    llm_language: Option<String>,
}

/// One approved feature candidate as stage 5 receives it.
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CandidateRef {
    pub key: String,
    pub name: String,
    pub location: String,
    pub symbol: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ClaimRow {
    id: String,
    user_id: String,
    installation_id: i64,
    repo_owner: String,
    repo_name: String,
    branch: String,
    llm_language: Option<String>,
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
/// The policy is honoured by *where this is called from*, not by anything here:
/// both call sites sit on a lease boundary, between the worker's calls.
async fn stop_for_revoked_access(state: &AppState, id: &str) -> Result<(), AppError> {
    let now = now_unix();
    sqlx::query(
        "UPDATE analyses \
            SET status = ?, error = ?, finished_at = ?, lease_expires_at = NULL, claimed_by = NULL \
          WHERE id = ?",
    )
    .bind(status::FAILED)
    .bind(analysis::ACCESS_REVOKED)
    .bind(now)
    .bind(id)
    .execute(&state.db)
    .await?;

    sqlx::query(
        "UPDATE analysis_stages SET status = ?, error = ?, finished_at = ? \
          WHERE analysis_id = ? AND status = ?",
    )
    .bind(stage_status::FAILED)
    .bind(analysis::ACCESS_REVOKED)
    .bind(now)
    .bind(id)
    .bind(stage_status::RUNNING)
    .execute(&state.db)
    .await?;

    tracing::info!(analysis_id = %id, "analysis stopped — repository access revoked");
    Ok(())
}

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
     RETURNING id, user_id, installation_id, repo_owner, repo_name, branch, llm_language",
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

    if !analysis::still_granted(&state, &job.user_id, &job.repo_owner, &job.repo_name).await? {
        stop_for_revoked_access(&state, &job.id).await?;
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

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

    // Unsealed here for the same reason as the installation token: the worker owns
    // no database, and a job-scoped plaintext that lives only for this response is
    // narrower than any stored alternative (AC4.1/AC4.3).
    let (llm_provider, llm_api_key) = match crate::llmkey::active_key_for_user(&state, &job.user_id)
        .await
    {
        Ok(Some((provider, key))) => (Some(provider), Some(key)),
        Ok(None) => (None, None),
        Err(e) => {
            tracing::warn!(analysis_id = %job.id, "llm key unavailable: {e:?}");
            (None, None)
        }
    };

    // AC1.3's gate. Read after the claim rather than inside it: the claim is an
    // atomic hand-off of one row and joining a second table into that statement
    // would put the gate inside the concurrency-critical path for no benefit.
    let approved_patterns = approved_patterns(&state, &job.id).await?;
    let approved_candidates = approved_candidates(&state, &job.id).await?;
    let dependency_requests = pending_dependency_requests(&state, &job.id).await?;
    let landscape = stored_landscape(&state, &job.id).await?;
    let executable_stages = offered_stages(
        &state,
        &job.id,
        Gates {
            strategy_approved: approved_patterns.is_some(),
            candidates_approved: !approved_candidates.is_empty(),
            acceptance_pending: acceptance_pending(&state, &job.id, &approved_candidates).await?,
            landscape_stored: landscape.is_some(),
        },
    )
    .await?;
    let offers = |k: &str| executable_stages.iter().any(|s| s == k);
    let cross_cutting_document =
        if offers(pipeline::DISCOVERY_STRATEGY) && !offers(pipeline::CROSS_CUTTING) {
            landscape
        } else {
            None
        };

    Ok(Json(ClaimView {
        id: job.id,
        repo_owner: job.repo_owner,
        repo_name: job.repo_name,
        branch: job.branch,
        executable_stages,
        cross_cutting_document,
        approved_patterns: approved_patterns.unwrap_or_default(),
        approved_candidates,
        dependency_requests,
        lease_expires_at: lease_until,
        installation_token,
        llm_provider,
        llm_api_key,
        llm_language: job.llm_language,
    })
    .into_response())
}

/// The approved strategy's patterns, or `None` when the reviewer has not approved
/// one — which is also AC1.3's gate value.
pub async fn approved_patterns(
    state: &AppState,
    analysis_id: &str,
) -> Result<Option<Vec<String>>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT entries FROM discovery_strategies \
          WHERE analysis_id = ? AND approved_at IS NOT NULL",
    )
    .bind(analysis_id)
    .fetch_optional(&state.db)
    .await?;
    let Some((entries,)) = row else {
        return Ok(None);
    };
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&entries).unwrap_or_default();
    Ok(Some(
        parsed
            .iter()
            .filter_map(|e| e.get("pattern").and_then(|p| p.as_str()))
            .map(str::to_string)
            .collect(),
    ))
}

/// The feature candidates the reviewer approved, in review order — stage 5's input
/// and, by being empty or not, its gate.
///
/// Merged-away rows are excluded: a candidate folded into another is no longer a
/// feature of its own.
pub async fn approved_candidates(
    state: &AppState,
    analysis_id: &str,
) -> Result<Vec<CandidateRef>, AppError> {
    Ok(sqlx::query_as(
        "SELECT key, name, location, symbol FROM feature_candidates \
          WHERE analysis_id = ? AND decision = 'approved' AND merged_into IS NULL \
          ORDER BY seq, rowid",
    )
    .bind(analysis_id)
    .fetch_all(&state.db)
    .await?)
}

/// Whether stage 5 has work to do: is there an approved feature the stored
/// acceptance document does not cover?
///
/// Not "has the stage succeeded" — that would be wrong here. A reviewer decides
/// candidates **one at a time**, so approving a second feature after the stage has
/// already run must re-open it; otherwise the second feature would never get a
/// document and the screen would silently show a shorter list than the reviewer
/// confirmed. Re-running rewrites the whole document (the submit route upserts on
/// `(analysis_id, kind)`), which keeps one document per analysis and keeps the
/// content hash meaningful.
pub async fn acceptance_pending(
    state: &AppState,
    analysis_id: &str,
    approved: &[CandidateRef],
) -> Result<bool, AppError> {
    if approved.is_empty() {
        return Ok(false);
    }
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(analysis_id)
    .bind(pipeline::ACCEPTANCE_DEPENDENCIES)
    .fetch_optional(&state.db)
    .await?;
    let Some((content,)) = row else {
        return Ok(true);
    };
    let doc: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
    let documented: Vec<String> = crate::acceptance::features(&doc)
        .into_iter()
        .map(|f| f.key)
        .collect();
    Ok(approved.iter().any(|c| !documented.contains(&c.key)))
}

/// The request row *is* the gate: until a run reaches a terminal status the row
/// keeps saying so. A failed run stays failed rather than retrying forever —
/// re-asking is a person's decision, and the request route upserts the row back to
/// `queued` when they make it.
pub async fn pending_dependency_requests(
    state: &AppState,
    analysis_id: &str,
) -> Result<Vec<CandidateRef>, AppError> {
    Ok(sqlx::query_as(
        "SELECT c.key, c.name, c.location, c.symbol \
           FROM feature_dependency_requests r \
           JOIN feature_candidates c \
             ON c.analysis_id = r.analysis_id AND c.key = r.feature_key \
          WHERE r.analysis_id = ? AND r.status = ? \
            AND c.decision = 'approved' AND c.merged_into IS NULL \
          ORDER BY c.seq, c.rowid",
    )
    .bind(analysis_id)
    .bind(crate::dependencies::request_status::QUEUED)
    .fetch_all(&state.db)
    .await?)
}

/// The gate values [`offered_stages`] decides on, read once per claim.
pub struct Gates {
    pub strategy_approved: bool,
    pub candidates_approved: bool,
    pub acceptance_pending: bool,
    pub landscape_stored: bool,
}

/// Which stages this claim offers.
///
/// The rule is "a stage that has not succeeded yet" and whose gate is open. A stage
/// that finished is back at `pending` only when a person asked for it to run again
/// (AC1.5's per-stage retry, which covers succeeded stages too) — so the same rule
/// is what re-runs exactly that stage and nothing behind it.
///
/// `cross_cutting` is also offered while `discovery_strategy` is pending **and** no
/// stage-2 document is stored: stage 3 plans over stage 2's landscape, and without
/// one it would arrive with nothing to plan over and silently do nothing. When the
/// document is stored the claim carries it instead (`crossCuttingDocument`), which is
/// what lets re-running stage 3 leave stage 2 untouched.
///
/// Stage 5 is offered when an approved feature is not yet documented (the reason
/// that does not look at stage status — see [`acceptance_pending`]) **or** when the
/// stage row is `pending` while candidates are approved, which is the re-run of a
/// stage whose document already covers every approved feature. `pending`, not "not
/// succeeded": a *failed* stage 5 waits for its own retry rather than riding along
/// on a re-run someone asked of a different stage.
///
/// `fetch` is always offered. It is the only non-LLM stage, it is what produces the
/// path list every later stage reads, and re-measuring the same tree costs a single
/// API call and yields the same numbers.
pub async fn offered_stages(
    state: &AppState,
    analysis_id: &str,
    gates: Gates,
) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT key, status FROM analysis_stages WHERE analysis_id = ?")
            .bind(analysis_id)
            .fetch_all(&state.db)
            .await?;
    let is = |key: &str, status: &str| rows.iter().any(|(k, s)| k == key && s == status);
    let done = |key: &str| is(key, stage_status::SUCCEEDED);

    let mut offered = vec![pipeline::FETCH.to_string()];
    if !done(pipeline::CROSS_CUTTING)
        || (!done(pipeline::DISCOVERY_STRATEGY) && !gates.landscape_stored)
    {
        offered.push(pipeline::CROSS_CUTTING.to_string());
    }
    if !done(pipeline::DISCOVERY_STRATEGY) {
        offered.push(pipeline::DISCOVERY_STRATEGY.to_string());
    }
    if gates.strategy_approved && !done(pipeline::FEATURE_CANDIDATES) {
        offered.push(pipeline::FEATURE_CANDIDATES.to_string());
    }
    if gates.acceptance_pending
        || (gates.candidates_approved
            && is(pipeline::ACCEPTANCE_DEPENDENCIES, stage_status::PENDING))
    {
        offered.push(pipeline::ACCEPTANCE_DEPENDENCIES.to_string());
    }
    Ok(offered)
}

async fn stored_landscape(
    state: &AppState,
    analysis_id: &str,
) -> Result<Option<serde_json::Value>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(analysis_id)
    .bind(pipeline::CROSS_CUTTING)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.and_then(|(content,)| serde_json::from_str(&content).ok()))
}

/// Whether a stage behind a human gate is still waiting — i.e. whether handing this
/// job back to the queue would accomplish anything. Used at [`finish`] to close the
/// one race the approvals cannot: an approval that lands *while* the worker is
/// running, when the re-queue is refused because the lease is live.
///
/// Only the gated stages count. `fetch` is always offered, so asking "is anything
/// offered" would re-queue every job forever.
async fn work_remains(state: &AppState, analysis_id: &str) -> Result<bool, AppError> {
    let strategy_approved = approved_patterns(state, analysis_id).await?.is_some();
    if !strategy_approved {
        return Ok(false);
    }
    let approved = approved_candidates(state, analysis_id).await?;
    let pending = acceptance_pending(state, analysis_id, &approved).await?;
    let gates = Gates {
        strategy_approved,
        candidates_approved: !approved.is_empty(),
        acceptance_pending: pending,
        landscape_stored: stored_landscape(state, analysis_id).await?.is_some(),
    };
    let offered = offered_stages(state, analysis_id, gates).await?;
    if offered
        .iter()
        .any(|k| k == pipeline::FEATURE_CANDIDATES || k == pipeline::ACCEPTANCE_DEPENDENCIES)
    {
        return Ok(true);
    }
    // A request that landed while this pass was running is the same race the
    // approvals have, and it gets the same answer.
    Ok(!pending_dependency_requests(state, analysis_id).await?.is_empty())
}

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

    if res.rows_affected() != 1 {
        return Err(AppError::Conflict("lease no longer held".into()));
    }

    let target: Option<(String, String, String)> = sqlx::query_as(
        "SELECT user_id, repo_owner, repo_name FROM analyses WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?;
    if let Some((user_id, owner, name)) = target {
        if !analysis::still_granted(&state, &user_id, &owner, &name).await? {
            stop_for_revoked_access(&state, &id).await?;
            return Err(AppError::Conflict("lease no longer held".into()));
        }
    }

    Ok(StatusCode::NO_CONTENT)
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

    // "Every implemented stage ran" can stop being true between the claim and this
    // call: approving a strategy opens stage 4, and the user may do it while this
    // pass is still running — in which case `approve_strategy`'s own re-queue is
    // refused (it never re-queues a job under a live lease). Landing on `queued`
    // here is the other half of that pair; without it the approval would open a
    // stage no claim would ever offer again.
    let landed = if req.status == status::AWAITING_PIPELINE && work_remains(&state, &id).await? {
        status::QUEUED
    } else {
        req.status.as_str()
    };
    let finished_at = (landed != status::QUEUED).then(now_unix);

    sqlx::query(
        "UPDATE analyses \
            SET status = ?, error = ?, finished_at = ?, lease_expires_at = NULL, \
                claimed_by = CASE WHEN ? THEN NULL ELSE claimed_by END \
          WHERE id = ? AND claimed_by = ?",
    )
    .bind(landed)
    .bind(req.error.as_deref())
    .bind(finished_at)
    .bind(landed == status::QUEUED)
    .bind(&id)
    .bind(&req.worker_id)
    .execute(&state.db)
    .await?;

    tracing::info!(analysis_id = %id, worker_id = %req.worker_id, status = %landed, "analysis finished");
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocumentReq {
    worker_id: String,
    content: serde_json::Value,
    model: String,
    /// How many provider calls this document cost. Defaults to 1 so a worker that
    /// predates the field still reports the common case rather than zero.
    #[serde(default = "one_call")]
    calls: i64,
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
}

fn one_call() -> i64 {
    1
}

/// Stores a stage's document output. Lease-guarded like [`report_stage`], so a
/// reclaimed worker cannot overwrite its successor's result.
///
/// Upsert rather than insert: AC1.5's per-stage retry re-runs the stage, and the
/// re-run's output replaces the previous one for that (analysis, kind) — the
/// analysis is the unit of history, not the document.
async fn submit_document(
    State(state): State<AppState>,
    _auth: WorkerAuth,
    Path((id, kind)): Path<(String, String)>,
    Json(req): Json<DocumentReq>,
) -> Result<StatusCode, AppError> {
    if pipeline::stage(&kind).is_none() {
        return Err(AppError::BadRequest("unknown pipeline stage".into()));
    }
    require_lease(&state, &id, &req.worker_id).await?;

    let serialized = serde_json::to_string(&req.content)
        .map_err(|_| AppError::BadRequest("document is not serializable".into()))?;
    let hash = content_hash(&serialized);

    sqlx::query(
        "INSERT INTO analysis_documents \
         (id, analysis_id, kind, content, content_hash, model, calls, input_tokens, output_tokens, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(analysis_id, kind) DO UPDATE SET \
           content = excluded.content, content_hash = excluded.content_hash, \
           model = excluded.model, calls = excluded.calls, \
           input_tokens = excluded.input_tokens, \
           output_tokens = excluded.output_tokens, created_at = excluded.created_at",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&id)
    .bind(&kind)
    .bind(&serialized)
    .bind(&hash)
    .bind(&req.model)
    .bind(req.calls.max(0))
    .bind(req.input_tokens)
    .bind(req.output_tokens)
    .bind(now_unix())
    .execute(&state.db)
    .await?;

    // 운영자가 보는 자리 — 단계별 호출·토큰이 로그에도 남는다(AC4.6). 화면은 집계를
    // 읽지만 그때는 이미 합쳐진 뒤라, 어느 단계가 얼마를 썼는지는 여기서만 보인다.
    tracing::info!(
        analysis_id = %id,
        stage = %kind,
        model = %req.model,
        calls = req.calls,
        input_tokens = req.input_tokens,
        output_tokens = req.output_tokens,
        "llm usage recorded"
    );

    // 저장과 같은 요청 안에서 이어받아야 워커가 5단계를 `succeeded` 로 보고하기 전에 충돌이 서 있다.
    if kind == pipeline::ACCEPTANCE_DEPENDENCIES {
        crate::doc_conflict::inherit(&state, &id, &req.content).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DependencyReq {
    worker_id: String,
    /// Which feature this run traced. In the body rather than the path because a
    /// candidate key carries the location it was found at (`src/api/routes.rs`).
    feature_key: String,
    status: String,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    content: serde_json::Value,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
}

/// Stores one feature's dependency extraction as rows.
///
/// A successful re-run **replaces** that feature's rows in the same transaction
/// that moves the request to its terminal status: leaving the old rows would make
/// the screen show a union of two readings of the code. A failed run replaces
/// nothing — a previous good answer outlives a bad attempt, and the row carries
/// the reason instead.
async fn submit_dependencies(
    State(state): State<AppState>,
    _auth: WorkerAuth,
    Path(id): Path<String>,
    Json(req): Json<DependencyReq>,
) -> Result<StatusCode, AppError> {
    use crate::dependencies::request_status;

    if !request_status::is_terminal(&req.status) {
        return Err(AppError::BadRequest("unknown dependency request status".into()));
    }
    let error = req.error.filter(|e| !e.trim().is_empty());
    if req.status == request_status::FAILED && error.is_none() {
        return Err(AppError::BadRequest("a failed run must carry a reason".into()));
    }
    require_lease(&state, &id, &req.worker_id).await?;

    let items = crate::dependencies::items(&req.content);
    let now = now_unix();
    let mut tx = state.db.begin().await?;

    let res = sqlx::query(
        "UPDATE feature_dependency_requests             SET status = ?, error = ?, model = ?, input_tokens = ?, output_tokens = ?,                 updated_at = ?           WHERE analysis_id = ? AND feature_key = ?",
    )
    .bind(&req.status)
    .bind(error.as_deref())
    .bind(req.model.as_deref())
    .bind(req.input_tokens)
    .bind(req.output_tokens)
    .bind(now)
    .bind(&id)
    .bind(&req.feature_key)
    .execute(&mut *tx)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    if req.status == request_status::SUCCEEDED {
        sqlx::query("DELETE FROM feature_dependencies WHERE analysis_id = ? AND feature_key = ?")
            .bind(&id)
            .bind(&req.feature_key)
            .execute(&mut *tx)
            .await?;
        for (seq, item) in items.iter().enumerate() {
            sqlx::query(
                "INSERT INTO feature_dependencies                    (id, analysis_id, feature_key, seq, category, name, evidence, created_at)                  VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&id)
            .bind(&req.feature_key)
            .bind(seq as i64)
            .bind(&item.category)
            .bind(&item.name)
            .bind(item.evidence.as_deref())
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Stable fingerprint of a document's canonical JSON. Comparing this across
/// analyses of the same target is what makes AC1.2's "결정적으로 재현되거나 차이가
/// 명시된다" observable without diffing documents client-side.
pub fn content_hash(serialized: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    format!("{:x}", hasher.finalize())
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

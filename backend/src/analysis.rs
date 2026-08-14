//! Analysis jobs: connect a repository, trigger an analysis (AC1.1), and follow it
//! while it runs (AC1.5).
//!
//! The enqueue half (AC1.1):
//!  - list the repositories the App can access (candidates to analyze),
//!  - pre-flight an estimated call count / cost so the user sees the scale before
//!    triggering (S03, journey F2), and
//!  - trigger — a user-initiated request lands a `queued` row after the target is
//!    confirmed within the App's granted access. An out-of-scope target is rejected
//!    with a clear, actionable message and nothing is queued (test scenario #2).
//!
//! Draining the queue is a separate workload (AC4.5): enqueue seeds one
//! `analysis_stages` row per [`crate::pipeline`] stage and the worker claims the
//! job through `/internal/*` (see [`crate::worker_api`]).
//!
//! The progress half (AC1.5) reads that same persisted state back:
//!  - `GET /api/analyses/{id}` — the job and its stages, which is everything S04
//!    draws. Nothing about the run lives in the client, so closing the app and
//!    coming back shows the same progress (test/01 시나리오 5).
//!  - `POST /api/analyses/{id}/stages/{key}/retry` — re-run one *failed* stage and
//!    only that one (시나리오 6).
//!
//! Real per-call cost accounting is still AC4.6: what these views report is the
//! pre-flight estimate, never a measured spend.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::github_app::{self, RepoRef};
use crate::installations;
use crate::pipeline;
use crate::state::AppState;
use crate::util::now_unix;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/repositories", get(list_repositories))
        .route("/api/analyses", post(create).get(list))
        .route("/api/analyses/preflight", post(preflight))
        .route("/api/analyses/{id}", get(detail))
        .route("/api/analyses/{id}/stages/{key}/retry", post(retry_stage))
}

// ── views / rows ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoView {
    owner: String,
    name: String,
    full_name: String,
    default_branch: String,
}

impl From<&RepoRef> for RepoView {
    fn from(r: &RepoRef) -> Self {
        Self {
            owner: r.owner.clone(),
            name: r.name.clone(),
            full_name: r.full_name.clone(),
            default_branch: r.default_branch.clone(),
        }
    }
}

/// What the API exposes for an analysis job (S02 home list).
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct AnalysisView {
    id: String,
    repo_owner: String,
    repo_name: String,
    branch: String,
    status: String,
    est_llm_calls: i64,
    est_cost_cents: i64,
    created_at: i64,
    /// Pipeline progress as a fraction, so the S02 card can read "step 2 of 5"
    /// without one request per row (AC1.5). The stages themselves belong to S04 —
    /// see [`detail`].
    stages_total: i64,
    stages_done: i64,
}

/// One pipeline step as S04 renders it, straight off `analysis_stages`.
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct StageView {
    seq: i64,
    key: String,
    title: String,
    status: String,
    /// The user-facing one-liner the worker measured ("766 files · 2.2 MB").
    detail: Option<String>,
    /// Why the stage failed, when it did. Retryable on its own (AC1.5).
    error: Option<String>,
    started_at: Option<i64>,
    finished_at: Option<i64>,
}

/// S04's whole payload: the job, its stages, and the run's own timestamps.
///
/// Everything on this view is *persisted* state — the screen keeps no progress of
/// its own, which is what makes "종료 후 복귀 시 동일한 진행률" (test/01 시나리오 5)
/// true by construction rather than by client-side bookkeeping.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisDetailView {
    #[serde(flatten)]
    analysis: AnalysisView,
    error: Option<String>,
    started_at: Option<i64>,
    finished_at: Option<i64>,
    stages: Vec<StageView>,
}

/// The analysis-level columns S04 adds on top of [`AnalysisView`].
#[derive(sqlx::FromRow)]
struct RunRow {
    error: Option<String>,
    started_at: Option<i64>,
    finished_at: Option<i64>,
}

/// `SELECT` list shared by the home list and the detail read. The two progress
/// counts are subqueries rather than a join so a job with no stage rows (none
/// exist today — enqueue seeds them in the same transaction) still returns a row.
/// Built rather than `const` so the "done" predicate keeps reading its value from
/// [`pipeline::stage_status`] instead of a second copy of the string.
fn analysis_columns() -> String {
    format!(
        "a.id, a.repo_owner, a.repo_name, a.branch, a.status, \
         a.est_llm_calls, a.est_cost_cents, a.created_at, \
         (SELECT COUNT(*) FROM analysis_stages s WHERE s.analysis_id = a.id) AS stages_total, \
         (SELECT COUNT(*) FROM analysis_stages s \
           WHERE s.analysis_id = a.id AND s.status = '{done}') AS stages_done",
        done = pipeline::stage_status::SUCCEEDED
    )
}

// ── handlers ──────────────────────────────────────────────────────────────────

/// The repositories the installation can access (candidates to analyze). Empty when
/// the user has not connected the App yet — the S02/S03 UI routes those users to S01.
async fn list_repositories(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<RepoView>>, AppError> {
    let repos = accessible_repos(&state, &user.id).await?;
    Ok(Json(repos.iter().map(RepoView::from).collect()))
}

/// The user's analysis jobs, newest first (S02 home list). Scoped to the owner (AC4.7).
async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<AnalysisView>>, AppError> {
    let rows = sqlx::query_as::<_, AnalysisView>(&format!(
        "SELECT {} FROM analyses a WHERE a.user_id = ? ORDER BY a.created_at DESC, a.id DESC",
        analysis_columns()
    ))
    .bind(&user.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

/// One analysis with its pipeline stages — the S04 screen's read (AC1.5).
///
/// Scoped to the owner (AC4.7): another user's id is `404`, not `403`, so the API
/// does not confirm that the id exists.
async fn detail(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AnalysisDetailView>, AppError> {
    Ok(Json(load_detail(&state, &user.id, &id).await?))
}

/// Re-runs one failed stage and nothing else (AC1.5: "실패한 단계는 그 단계만
/// 재시도할 수 있다", test/01 시나리오 6).
///
/// The retry is expressed as a *queue* operation rather than a second worker
/// protocol: the failed stage row goes back to `pending` and the job goes back to
/// `queued`, so the existing claim/lease path in [`crate::worker_api`] performs the
/// re-run. Sibling stage rows are not touched, which is what keeps already-finished
/// work (and its measured detail) intact.
async fn retry_stage(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, key)): Path<(String, String)>,
) -> Result<Json<AnalysisDetailView>, AppError> {
    if pipeline::stage(&key).is_none() {
        return Err(AppError::BadRequest("unknown pipeline stage".into()));
    }

    let current: Option<(String,)> =
        sqlx::query_as("SELECT status FROM analyses WHERE id = ? AND user_id = ?")
            .bind(&id)
            .bind(&user.id)
            .fetch_optional(&state.db)
            .await?;
    let (analysis_status,) = current.ok_or(AppError::NotFound)?;

    // A job still inside a worker's lease must not be re-queued underneath it: the
    // holder would keep reporting into the run we just reset.
    if analysis_status == pipeline::status::RUNNING {
        return Err(AppError::Conflict(
            "이 분석은 아직 실행 중입니다. 끝난 뒤 다시 시도해 주세요.".into(),
        ));
    }

    let mut tx = state.db.begin().await?;
    let reset = sqlx::query(
        "UPDATE analysis_stages \
            SET status = ?, detail = NULL, error = NULL, started_at = NULL, finished_at = NULL \
          WHERE analysis_id = ? AND key = ? AND status = ?",
    )
    .bind(pipeline::stage_status::PENDING)
    .bind(&id)
    .bind(&key)
    .bind(pipeline::stage_status::FAILED)
    .execute(&mut *tx)
    .await?;
    if reset.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "실패한 단계만 다시 시도할 수 있습니다.".into(),
        ));
    }

    sqlx::query(
        "UPDATE analyses \
            SET status = ?, error = NULL, claimed_by = NULL, claimed_at = NULL, \
                lease_expires_at = NULL, finished_at = NULL \
          WHERE id = ?",
    )
    .bind(pipeline::status::QUEUED)
    .bind(&id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    crate::audit::record(&state.db, Some(&user.id), "analysis.stage_retry", Some(&key)).await;

    Ok(Json(load_detail(&state, &user.id, &id).await?))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetReq {
    /// A GitHub repo URL (or `owner/name`) the user typed on S03.
    repo_url: String,
    /// Optional branch override; falls back to the repo's default branch.
    #[serde(default)]
    branch: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PreflightView {
    /// Whether the target is within the App's granted access. `false` is not an
    /// error — S03 renders the "add this repo to the App" recovery path.
    has_access: bool,
    owner: String,
    name: String,
    full_name: String,
    branch: String,
    files_to_scan: i64,
    size_bytes: i64,
    est_llm_calls: i64,
    est_cost_cents: i64,
    est_duration_min: i64,
}

/// Pre-flight estimate for S03: resolves the typed target, reports whether it is
/// within the App's granted access, and — when it is — the expected analysis scale
/// (files, LLM calls, cost, duration) so the user sees the cost before triggering.
async fn preflight(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<TargetReq>,
) -> Result<Json<PreflightView>, AppError> {
    let (owner, name) = parse_repo(&req.repo_url)?;
    let repos = accessible_repos(&state, &user.id).await?;
    let matched = repos
        .iter()
        .find(|r| r.owner.eq_ignore_ascii_case(&owner) && r.name.eq_ignore_ascii_case(&name));

    match matched {
        None => {
            let full_name = format!("{owner}/{name}");
            Ok(Json(PreflightView {
                has_access: false,
                owner,
                name,
                full_name,
                branch: req.branch.unwrap_or_default(),
                files_to_scan: 0,
                size_bytes: 0,
                est_llm_calls: 0,
                est_cost_cents: 0,
                est_duration_min: 0,
            }))
        }
        Some(repo) => {
            let branch = resolve_branch(req.branch, repo);
            let est = Estimate::from_size_kb(repo.size_kb);
            Ok(Json(PreflightView {
                has_access: true,
                owner: repo.owner.clone(),
                name: repo.name.clone(),
                full_name: repo.full_name.clone(),
                branch,
                files_to_scan: est.files,
                size_bytes: repo.size_kb * 1024,
                est_llm_calls: est.llm_calls,
                est_cost_cents: est.cost_cents,
                est_duration_min: est.duration_min,
            }))
        }
    }
}

/// Explicitly triggers an analysis (S03 "분석 시작"). Confirms the target is within
/// the App's granted access, then enqueues a `queued` job (201). An out-of-scope
/// target — or one with no App installed — is rejected with a clear message and
/// nothing is queued (AC1.1 / test scenario #2). Analysis never starts implicitly.
async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<TargetReq>,
) -> Result<(StatusCode, Json<AnalysisView>), AppError> {
    let (owner, name) = parse_repo(&req.repo_url)?;

    let installation = installations::get_for_user(&state.db, &user.id)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest("GitHub App이 아직 설치되지 않았습니다. 먼저 App을 설치해 주세요.".into())
        })?;

    let repos = github_app::list_repositories(&state, installation.installation_id).await?;
    let repo = repos
        .iter()
        .find(|r| r.owner.eq_ignore_ascii_case(&owner) && r.name.eq_ignore_ascii_case(&name))
        .ok_or_else(|| {
            AppError::BadRequest(
                "이 저장소에 접근할 수 없습니다. GitHub App 설치 범위에 이 저장소를 추가해 주세요.".into(),
            )
        })?;

    let branch = resolve_branch(req.branch, repo);
    let est = Estimate::from_size_kb(repo.size_kb);

    let id = uuid::Uuid::new_v4().to_string();
    let now = now_unix();

    // The job and its stage rows land together: a worker that claims between the
    // two would otherwise find a job with no stages to report against.
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO analyses \
         (id, user_id, installation_id, repo_owner, repo_name, branch, status, est_llm_calls, est_cost_cents, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(installation.installation_id)
    .bind(&repo.owner)
    .bind(&repo.name)
    .bind(&branch)
    .bind(pipeline::status::QUEUED)
    .bind(est.llm_calls)
    .bind(est.cost_cents)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    for stage in pipeline::STAGES.iter() {
        sqlx::query(
            "INSERT INTO analysis_stages (id, analysis_id, seq, key, title, status) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&id)
        .bind(stage.seq)
        .bind(stage.key)
        .bind(stage.title)
        .bind(pipeline::stage_status::PENDING)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    crate::audit::record(
        &state.db,
        Some(&user.id),
        "analysis.enqueue",
        Some(&repo.full_name),
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(AnalysisView {
            id,
            repo_owner: repo.owner.clone(),
            repo_name: repo.name.clone(),
            branch,
            status: pipeline::status::QUEUED.to_string(),
            est_llm_calls: est.llm_calls,
            est_cost_cents: est.cost_cents,
            created_at: now,
            stages_total: pipeline::STAGES.len() as i64,
            stages_done: 0,
        }),
    ))
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Reads one analysis and its stages under the owner's scope, or `404`.
async fn load_detail(
    state: &AppState,
    user_id: &str,
    id: &str,
) -> Result<AnalysisDetailView, AppError> {
    let analysis = sqlx::query_as::<_, AnalysisView>(&format!(
        "SELECT {} FROM analyses a WHERE a.id = ? AND a.user_id = ?",
        analysis_columns()
    ))
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let run = sqlx::query_as::<_, RunRow>(
        "SELECT error, started_at, finished_at FROM analyses WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    let stages = sqlx::query_as::<_, StageView>(
        "SELECT seq, key, title, status, detail, error, started_at, finished_at \
           FROM analysis_stages WHERE analysis_id = ? ORDER BY seq",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(AnalysisDetailView {
        analysis,
        error: run.error,
        started_at: run.started_at,
        finished_at: run.finished_at,
        stages,
    })
}

/// The repositories the user's installation can access, or empty when not installed.
async fn accessible_repos(state: &AppState, user_id: &str) -> Result<Vec<RepoRef>, AppError> {
    match installations::get_for_user(&state.db, user_id).await? {
        None => Ok(Vec::new()),
        Some(inst) => github_app::list_repositories(state, inst.installation_id).await,
    }
}

/// The requested branch when non-blank, else the repo's default branch.
fn resolve_branch(requested: Option<String>, repo: &RepoRef) -> String {
    requested
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty())
        .unwrap_or_else(|| repo.default_branch.clone())
}

/// Parses a repo target. Accepts `owner/name`, `github.com/owner/name`, and full
/// `https://github.com/owner/name(.git)` URLs. Anything else is a validation error
/// (S03 URL typo → immediate feedback).
fn parse_repo(input: &str) -> Result<(String, String), AppError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(AppError::BadRequest("저장소 URL을 입력해 주세요".into()));
    }
    // Drop scheme + host if present, keep the path.
    let path = s.split_once("://").map(|(_, rest)| rest).unwrap_or(s);
    let path = path.strip_prefix("github.com/").unwrap_or(path);
    let mut parts = path.split('/').filter(|p| !p.is_empty());
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => {
            let name = name.strip_suffix(".git").unwrap_or(name);
            if owner.is_empty() || name.is_empty() {
                return Err(AppError::BadRequest("저장소 URL 형식이 올바르지 않습니다".into()));
            }
            Ok((owner.to_string(), name.to_string()))
        }
        _ => Err(AppError::BadRequest("저장소 URL 형식이 올바르지 않습니다".into())),
    }
}

/// Deterministic pre-flight heuristic (AC1.1: show the expected scale before the user
/// triggers). Derived only from the repo's reported size — an order-of-magnitude the
/// user sees on S03, never a hard cost. Real per-call accounting lands with AC4.6.
struct Estimate {
    files: i64,
    llm_calls: i64,
    cost_cents: i64,
    duration_min: i64,
}

impl Estimate {
    fn from_size_kb(size_kb: i64) -> Self {
        let size_kb = size_kb.max(1);
        let files = (size_kb / 3).max(1); // ~3 KiB per source file
        let llm_calls = (files / 8 + 10).max(1); // batched scans + fixed pipeline steps
        let cost_cents = (llm_calls * 6 / 10).max(1); // ~$0.006 per call
        let duration_min = (llm_calls / 20 + 1).max(1);
        Self {
            files,
            llm_calls,
            cost_cents,
            duration_min,
        }
    }
}

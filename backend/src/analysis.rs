//! Analysis jobs (AC1.1): connect a repository and explicitly trigger an analysis.
//!
//! This slice implements the *enqueue* contract only:
//!  - list the repositories the App can access (candidates to analyze),
//!  - pre-flight an estimated call count / cost so the user sees the scale before
//!    triggering (S03, journey F2), and
//!  - trigger — a user-initiated request lands a `queued` row after the target is
//!    confirmed within the App's granted access. An out-of-scope target is rejected
//!    with a clear, actionable message and nothing is queued (test scenario #2).
//!
//! Draining the queue (the async worker) and per-stage progress arrive with the
//! analysis pipeline (AC1.5 / AC4.5); real per-call cost accounting with AC4.6.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::github_app::{self, RepoRef};
use crate::installations;
use crate::state::AppState;
use crate::util::now_unix;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/repositories", get(list_repositories))
        .route("/api/analyses", post(create).get(list))
        .route("/api/analyses/preflight", post(preflight))
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
    let rows = sqlx::query_as::<_, AnalysisView>(
        "SELECT id, repo_owner, repo_name, branch, status, est_llm_calls, est_cost_cents, created_at \
         FROM analyses WHERE user_id = ? ORDER BY created_at DESC, id DESC",
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
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
    sqlx::query(
        "INSERT INTO analyses \
         (id, user_id, installation_id, repo_owner, repo_name, branch, status, est_llm_calls, est_cost_cents, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, 'queued', ?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(installation.installation_id)
    .bind(&repo.owner)
    .bind(&repo.name)
    .bind(&branch)
    .bind(est.llm_calls)
    .bind(est.cost_cents)
    .bind(now)
    .execute(&state.db)
    .await?;
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
            status: "queued".to_string(),
            est_llm_calls: est.llm_calls,
            est_cost_cents: est.cost_cents,
            created_at: now,
        }),
    ))
}

// ── helpers ───────────────────────────────────────────────────────────────────

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

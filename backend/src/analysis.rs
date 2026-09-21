//! Analysis jobs: connect a repository, trigger an analysis (AC1.1), and follow it
//! while it runs (AC1.5).
//!
//! The enqueue half (AC1.1):
//!  - list the repositories the App can access (candidates to analyze),
//!  - pre-flight an estimated call count / cost so the user sees the scale before
//!    triggering (Connect Repository, journey F2), and
//!  - trigger — a user-initiated request lands a `queued` row after the target is
//!    confirmed within the App's granted access. An out-of-scope target is rejected
//!    with a clear, actionable message and nothing is queued (test scenario #2).
//!
//! Draining the queue is a separate workload (AC4.5): enqueue seeds one
//! `analysis_stages` row per [`crate::pipeline`] stage and the worker claims the
//! job through `/internal/*` (see [`crate::worker_api`]).
//!
//! The progress half (AC1.5) reads that same persisted state back:
//!  - `GET /api/analyses/{id}` — the job and its stages, which is everything Analysis Progress
//!    draws. Nothing about the run lives in the client, so closing the app and
//!    coming back shows the same progress (test/01 시나리오 5).
//!  - the discovery-strategy review routes (AC1.3) — read the strategy stage 3
//!    proposed, edit the list, and approve it. Approval is what opens the next
//!    pipeline stage in the queue, so "승인된 전략만 다음 단계의 입력이 된다" is
//!    enforced in one place rather than trusted to each caller.
//!  - `POST /api/analyses/{id}/stages/{key}/retry` — re-run one *failed* stage and
//!    only that one (시나리오 6).
//!
//! And the documents the pipeline produces (AC1.2~AC1.4):
//!  - `GET /api/analyses/{id}/documents/{kind}` — one stage's output, plus whether
//!    it reproduced the previous analysis of the same target. AC1.2 requires that a
//!    re-analysis either reproduce deterministically *or* state the difference;
//!    comparing the stored content hash is what turns that into something the
//!    screen can show rather than something the reader has to take on trust.
//!
//! Real per-call cost accounting is still AC4.6: what these views report is the
//! pre-flight estimate, never a measured spend.

use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::auth::CurrentUser;
use crate::dependencies;
use crate::diff;
use crate::doc_edit;
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
        .route("/api/analyses/{id}/documents/{kind}", get(document))
        .route("/api/analyses/{id}/discovery-strategy", get(strategy))
        .route(
            "/api/analyses/{id}/discovery-strategy/entries",
            put(update_strategy),
        )
        .route(
            "/api/analyses/{id}/discovery-strategy/approve",
            post(approve_strategy),
        )
        // The candidate key carries the path it was found at (`src/routes/auth.ts`),
        // so it cannot be a path segment — one candidate would span several. It
        // travels in the body of each action instead.
        .route("/api/analyses/{id}/candidates", get(candidates))
        .route("/api/analyses/{id}/candidates/decision", post(decide_candidate))
        .route("/api/analyses/{id}/candidates/rename", post(rename_candidate))
        .route("/api/analyses/{id}/candidates/merge", post(merge_candidates))
        // feature key 는 경로 구분자를 품으므로 본문·쿼리로 받는다.
        .route(
            "/api/analyses/{id}/features/dependencies",
            get(feature_dependencies).post(request_dependencies),
        )
        .route("/api/analyses/{id}/dependencies/export", get(export_dependencies))
        .route("/api/analyses/{id}/diff", get(analysis_diff))
        // 역방향 질의만 분석 하나에 매이지 않는다 — 이 사용자의 모든 분석을 가로지른다.
        .route("/api/dependencies/features", get(dependents))
}

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
    /// Pipeline progress as a fraction, so the Home card can read "step 2 of 5"
    /// without one request per row (AC1.5). The stages themselves belong to Analysis Progress —
    /// see [`detail`].
    stages_total: i64,
    stages_done: i64,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct StageView {
    seq: i64,
    key: String,
    title: String,
    status: String,
    detail: Option<String>,
    /// Why the stage failed, when it did. Retryable on its own (AC1.5).
    error: Option<String>,
    started_at: Option<i64>,
    finished_at: Option<i64>,
}

/// The Analysis Progress payload: the job, its stages, and the run's own timestamps.
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

/// The repositories the installation can access (candidates to analyze). Empty when
/// the user has not connected the App yet — the Home and Connect Repository screens route those users to Credentials Setup.
async fn list_repositories(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<RepoView>>, AppError> {
    let repos = accessible_repos(&state, &user.id).await?;
    Ok(Json(repos.iter().map(RepoView::from).collect()))
}

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

/// One analysis with its pipeline stages — the Analysis Progress screen's read (AC1.5).
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

/// Whether this document reproduced the previous analysis of the same target.
///
/// `first` — no earlier analysis of this repository+branch produced this document,
/// so there is nothing to compare against. `unchanged` / `changed` — there was one,
/// and the content hash did or did not match.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReproducibilityView {
    verdict: &'static str,
    /// The analysis this was compared against, when there was one.
    compared_to: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentView {
    kind: String,
    content: serde_json::Value,
    model: String,
    created_at: i64,
    reproducibility: ReproducibilityView,
}

#[derive(sqlx::FromRow)]
struct DocumentRow {
    content: String,
    content_hash: String,
    model: String,
    created_at: i64,
}

/// One pipeline document for an analysis the caller owns.
///
/// Scoped through `analyses.user_id` rather than trusting the analysis id, so
/// another user's document id is a 404 like every other read here (AC4.7).
async fn document(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, kind)): Path<(String, String)>,
) -> Result<Json<DocumentView>, AppError> {
    // The wire form is hyphenated (`cross-cutting`); the stage key is not.
    let key = kind.replace('-', "_");
    if pipeline::stage(&key).is_none() {
        return Err(AppError::BadRequest("unknown pipeline stage".into()));
    }

    let row = sqlx::query_as::<_, DocumentRow>(
        "SELECT d.content, d.content_hash, d.model, d.created_at \
           FROM analysis_documents d JOIN analyses a ON a.id = d.analysis_id \
          WHERE d.analysis_id = ? AND d.kind = ? AND a.user_id = ?",
    )
    .bind(&id)
    .bind(&key)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let mut content: serde_json::Value = serde_json::from_str(&row.content)
        .map_err(|_| AppError::BadRequest("stored document is unreadable".into()))?;

    // 사람이 확정한 추가와 승인한 편집은 저장을 고치지 않고 읽는 자리에서 겹쳐진다
    // (AC3.2·AC3.1·AC3.4). 추가가 먼저다 — 더해진 feature 도 편집의 대상이라야 한다.
    // 아래 재현성 판정이 `row.content_hash` 를 쓰는 것은 그래서 그대로다 — AC1.2 가
    // 묻는 것은 「재분석이 같은 결과를 냈는가」이지 「사람이 그 뒤에 문장을
    // 다듬었거나 feature 를 더했는가」가 아니다.
    crate::feature_add::overlay(&state, &id, &mut content).await?;
    doc_edit::overlay(&state, &id, &mut content).await?;

    // The most recent *earlier* analysis of the same repository and branch that
    // produced this document. Same target, so a differing hash is a real difference
    // in the result rather than a difference in what was analyzed.
    //
    // "Earlier" is `(created_at, rowid)`, not `created_at` alone. Timestamps here
    // are unix *seconds*, so two analyses triggered in the same second compare
    // equal — ordering on the timestamp alone dropped the real predecessor and
    // reported a re-run as if it were a first run. `rowid` is SQLite's insertion
    // order and breaks the tie the way the user experienced it.
    let previous: Option<(String, String)> = sqlx::query_as(
        "SELECT prev.id, d.content_hash \
           FROM analyses cur \
           JOIN analyses prev \
             ON prev.user_id = cur.user_id AND prev.repo_owner = cur.repo_owner \
            AND prev.repo_name = cur.repo_name AND prev.branch = cur.branch \
            AND (prev.created_at < cur.created_at \
                 OR (prev.created_at = cur.created_at AND prev.rowid < cur.rowid)) \
           JOIN analysis_documents d ON d.analysis_id = prev.id AND d.kind = ? \
          WHERE cur.id = ? AND cur.user_id = ? \
          ORDER BY prev.created_at DESC, prev.rowid DESC LIMIT 1",
    )
    .bind(&key)
    .bind(&id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await?;

    let reproducibility = match previous {
        None => ReproducibilityView {
            verdict: "first",
            compared_to: None,
        },
        Some((prev_id, prev_hash)) => ReproducibilityView {
            verdict: if prev_hash == row.content_hash {
                "unchanged"
            } else {
                "changed"
            },
            compared_to: Some(prev_id),
        },
    };

    Ok(Json(DocumentView {
        kind: key,
        content,
        model: row.model,
        created_at: row.created_at,
        reproducibility,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetReq {
    /// A GitHub repo URL (or `owner/name`) the user typed on Connect Repository.
    repo_url: String,
    /// Optional branch override; falls back to the repo's default branch.
    #[serde(default)]
    branch: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PreflightView {
    /// Whether the target is within the App's granted access. `false` is not an
    /// error — Connect Repository renders the "add this repo to the App" recovery path.
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

/// Pre-flight estimate for Connect Repository: resolves the typed target, reports whether it is
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

/// Explicitly triggers an analysis (Connect Repository "분석 시작"). Confirms the target is within
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

async fn accessible_repos(state: &AppState, user_id: &str) -> Result<Vec<RepoRef>, AppError> {
    match installations::get_for_user(&state.db, user_id).await? {
        None => Ok(Vec::new()),
        Some(inst) => github_app::list_repositories(state, inst.installation_id).await,
    }
}

fn resolve_branch(requested: Option<String>, repo: &RepoRef) -> String {
    requested
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty())
        .unwrap_or_else(|| repo.default_branch.clone())
}

fn parse_repo(input: &str) -> Result<(String, String), AppError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(AppError::BadRequest("저장소 URL을 입력해 주세요".into()));
    }
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
/// user sees on Connect Repository, never a hard cost. Real per-call accounting lands with AC4.6.
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

// Stage 3 proposes; this is where a person decides. The proposal itself stays in
// `analysis_documents` untouched (so re-running the stage and reproducibility keep
// working); what the user edits lives in `discovery_strategies` (migration 0006)
// and is what AC1.3 calls "승인된 전략".

/// How long the reviewable list may get once the user starts adding to it. Twice
/// the model's cap: the person adding non-standard entry points knows their own
/// codebase, but a list this screen cannot show is not reviewable either.
const MAX_STRATEGY_ENTRIES: usize = 24;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StrategyEntry {
    pattern: String,
    /// `generated` — proposed by stage 3. `user` — added by the reviewer, and the
    /// only kind carried into the next analysis of the same target.
    source: String,
}

const SOURCE_GENERATED: &str = "generated";
const SOURCE_USER: &str = "user";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StrategyView {
    entries: Vec<StrategyEntry>,
    /// AC1.3's gate, as a single boolean the screen can render and the queue reads.
    approved: bool,
    approved_at: Option<i64>,
    updated_at: i64,
}

#[derive(sqlx::FromRow)]
struct StrategyRow {
    entries: String,
    approved_at: Option<i64>,
    updated_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntriesReq {
    /// The list as the screen is showing it — deletions are absences, additions are
    /// new members. The screen PUTs what it has rather than sending a diff, because
    /// a diff of an unordered list is ambiguous about which duplicate went away.
    patterns: Vec<String>,
}

/// The analysis, if it belongs to this user. `404` (not `403`) for someone else's
/// id, so the API never confirms that an id exists (AC4.7).
pub(crate) async fn owned_analysis(
    state: &AppState,
    user_id: &str,
    id: &str,
) -> Result<(String, String, String, i64), AppError> {
    sqlx::query_as::<_, (String, String, String, i64)>(
        "SELECT repo_owner, repo_name, branch, created_at FROM analyses \
          WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)
}

/// The patterns stage 3 proposed for this analysis, or `None` when the stage has
/// not produced its document yet.
async fn proposed_patterns(state: &AppState, id: &str) -> Result<Option<Vec<String>>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(id)
    .bind(pipeline::DISCOVERY_STRATEGY)
    .fetch_optional(&state.db)
    .await?;
    let Some((content,)) = row else {
        return Ok(None);
    };
    let doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|_| AppError::internal("stored strategy document is unreadable"))?;
    Ok(Some(crate::discovery_strategy::patterns(&doc)))
}

/// The reviewer's own entries from the most recent *earlier* approved strategy for
/// the same target — the mockup's promise that "여기서 보탠 항목은 다음 분석에서도
/// 그대로 참조됩니다". Only `user` entries carry over: a generated pattern belongs
/// to the analysis that generated it.
async fn carried_over(
    state: &AppState,
    user_id: &str,
    id: &str,
    owner: &str,
    name: &str,
    branch: &str,
    created_at: i64,
) -> Result<Vec<StrategyEntry>, AppError> {
    // Ordered by `(created_at, rowid)` for the reason documented on the
    // reproducibility query: unix *seconds* tie for analyses triggered together.
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT s.entries FROM discovery_strategies s \
           JOIN analyses prev ON prev.id = s.analysis_id \
          WHERE prev.user_id = ? AND prev.repo_owner = ? AND prev.repo_name = ? \
            AND prev.branch = ? AND prev.id != ? AND s.approved_at IS NOT NULL \
            AND (prev.created_at < ? \
                 OR (prev.created_at = ? \
                     AND prev.rowid < (SELECT rowid FROM analyses WHERE id = ?))) \
          ORDER BY prev.created_at DESC, prev.rowid DESC LIMIT 1",
    )
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

    let Some((entries,)) = row else {
        return Ok(Vec::new());
    };
    let parsed: Vec<StrategyEntry> = serde_json::from_str(&entries).unwrap_or_default();
    Ok(parsed.into_iter().filter(|e| e.source == SOURCE_USER).collect())
}

/// Appends `entry` unless an equal pattern is already present. Order is the review
/// order the screen shows, so first occurrence wins.
fn push_unique(into: &mut Vec<StrategyEntry>, entry: StrategyEntry) {
    if !into.iter().any(|e| e.pattern == entry.pattern) {
        into.push(entry);
    }
}

/// The strategy for one analysis, seeding it from stage 3's proposal on first read.
///
/// Seeding lazily (rather than when the stage reports) keeps the worker protocol
/// unchanged and makes the seed idempotent: whoever reads first materialises it,
/// and every later read returns the row that person edited.
async fn strategy(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<StrategyView>, AppError> {
    let (owner, name, branch, created_at) = owned_analysis(&state, &user.id, &id).await?;

    if let Some(row) = load_strategy(&state, &id).await? {
        return Ok(Json(row));
    }

    // 404 rather than an empty strategy: "아직 제안되지 않았다" and "제안했는데
    // 비었다"는 사용자에게 다른 상태다 (stage 2's document read makes the same
    // distinction).
    let Some(proposed) = proposed_patterns(&state, &id).await? else {
        return Err(AppError::NotFound);
    };

    let mut entries: Vec<StrategyEntry> = Vec::new();
    for pattern in proposed {
        push_unique(
            &mut entries,
            StrategyEntry { pattern, source: SOURCE_GENERATED.to_string() },
        );
    }
    for entry in carried_over(&state, &user.id, &id, &owner, &name, &branch, created_at).await? {
        push_unique(&mut entries, entry);
    }
    entries.truncate(MAX_STRATEGY_ENTRIES);

    let now = now_unix();
    // `OR IGNORE`: two concurrent first reads would otherwise race to insert the
    // same seed. The loser simply reads the winner's row below.
    sqlx::query(
        "INSERT OR IGNORE INTO discovery_strategies (analysis_id, entries, approved_at, updated_at) \
         VALUES (?, ?, NULL, ?)",
    )
    .bind(&id)
    .bind(serde_json::to_string(&entries).map_err(|_| AppError::internal("strategy encode"))?)
    .bind(now)
    .execute(&state.db)
    .await?;

    load_strategy(&state, &id).await?.ok_or(AppError::NotFound).map(Json)
}

async fn load_strategy(state: &AppState, id: &str) -> Result<Option<StrategyView>, AppError> {
    let row: Option<StrategyRow> = sqlx::query_as(
        "SELECT entries, approved_at, updated_at FROM discovery_strategies WHERE analysis_id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.map(|r| StrategyView {
        entries: serde_json::from_str(&r.entries).unwrap_or_default(),
        approved: r.approved_at.is_some(),
        approved_at: r.approved_at,
        updated_at: r.updated_at,
    }))
}

/// Replaces the reviewable list (AC1.3's "검토·수정").
///
/// Rejected once approved: the approved strategy is the pipeline's input, and
/// letting it change underneath a running or finished stage 4 would make "이 후보는
/// 어느 전략에서 나왔나"를 답할 수 없게 만든다. Re-opening it would be a new
/// analysis, which is already how this product expresses "다시 해 보자".
async fn update_strategy(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(req): Json<EntriesReq>,
) -> Result<Json<StrategyView>, AppError> {
    owned_analysis(&state, &user.id, &id).await?;
    let current = load_strategy(&state, &id).await?.ok_or(AppError::NotFound)?;
    if current.approved {
        return Err(AppError::Conflict(
            "승인된 전략은 수정할 수 없어요. 다시 분석하면 새 전략을 검토할 수 있습니다.".into(),
        ));
    }

    // A pattern the model proposed keeps its provenance even if the user deleted and
    // re-added it; anything else is theirs, and only theirs carries forward.
    let proposed = proposed_patterns(&state, &id).await?.unwrap_or_default();

    let mut entries: Vec<StrategyEntry> = Vec::new();
    for pattern in req.patterns {
        let pattern = pattern.trim().to_string();
        if pattern.is_empty() {
            continue;
        }
        if pattern.chars().count() > 200 {
            return Err(AppError::BadRequest("탐색 대상이 너무 길어요".into()));
        }
        let source = if proposed.contains(&pattern) { SOURCE_GENERATED } else { SOURCE_USER };
        push_unique(&mut entries, StrategyEntry { pattern, source: source.to_string() });
    }
    if entries.len() > MAX_STRATEGY_ENTRIES {
        return Err(AppError::BadRequest(format!(
            "탐색 대상은 최대 {MAX_STRATEGY_ENTRIES}건까지예요"
        )));
    }

    sqlx::query("UPDATE discovery_strategies SET entries = ?, updated_at = ? WHERE analysis_id = ?")
        .bind(serde_json::to_string(&entries).map_err(|_| AppError::internal("strategy encode"))?)
        .bind(now_unix())
        .bind(&id)
        .execute(&state.db)
        .await?;

    load_strategy(&state, &id).await?.ok_or(AppError::NotFound).map(Json)
}

/// Approves the strategy (AC1.3's "승인").
///
/// This is the only thing that opens `feature_candidates` in the queue — see the
/// `executable_stages` comment in [`crate::worker_api`]. An empty strategy cannot be
/// approved: a scan with no entry points would silently produce no candidates and
/// read as "이 저장소에는 기능이 없다".
async fn approve_strategy(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<StrategyView>, AppError> {
    owned_analysis(&state, &user.id, &id).await?;
    let current = load_strategy(&state, &id).await?.ok_or(AppError::NotFound)?;
    if current.approved {
        return Err(AppError::Conflict("이미 승인된 전략이에요.".into()));
    }
    if current.entries.is_empty() {
        return Err(AppError::BadRequest(
            "탐색 대상이 하나도 없어요. 최소 한 곳은 남겨 주세요.".into(),
        ));
    }

    // Approval and the re-queue are one transaction because they are one fact:
    // "this analysis may now run stage 4". Stage 3 leaves the job in
    // `awaiting_pipeline`, and `worker_api::claim` only selects `queued` (or an
    // expired lease) — so without this write the approved strategy would open a
    // stage no worker could ever be handed. The `status <> running` guard is
    // `retry_stage`'s invariant: never re-queue a job underneath its lease holder.
    // The job finishing under that lease lands on `queued` itself (worker_api::finish).
    let now = now_unix();
    let mut tx = state.db.begin().await?;
    sqlx::query("UPDATE discovery_strategies SET approved_at = ?, updated_at = ? WHERE analysis_id = ?")
        .bind(now)
        .bind(now)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    let requeued = requeue(&mut tx, &id).await?;
    tx.commit().await?;

    tracing::info!(
        analysis_id = %id,
        entries = current.entries.len(),
        requeued,
        "discovery strategy approved"
    );
    load_strategy(&state, &id).await?.ok_or(AppError::NotFound).map(Json)
}

/// Puts an analysis back in the queue so the stage an approval just opened gets a
/// turn.
///
/// Never touches a job under a live lease (`status <> running`): re-queueing
/// underneath its holder is exactly what `retry_stage`'s invariant forbids, and
/// `worker_api::finish` lands such a job on `queued` itself when it sees that work
/// remains.
async fn requeue(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    id: &str,
) -> Result<bool, AppError> {
    let res = sqlx::query(
        "UPDATE analyses \
            SET status = ?, error = NULL, claimed_by = NULL, claimed_at = NULL, \
                lease_expires_at = NULL, finished_at = NULL \
          WHERE id = ? AND status <> ?",
    )
    .bind(pipeline::status::QUEUED)
    .bind(id)
    .bind(pipeline::status::RUNNING)
    .execute(&mut **tx)
    .await?;
    Ok(res.rows_affected() == 1)
}

// Stage 4 extracts; this is where a person sifts. Same split as AC1.3: the
// generated document stays in `analysis_documents` untouched (reproducibility keeps
// working), and what the user decides lives in `feature_candidates` (migration
// 0007). AC1.4's 검증 방법 names four actions — 승인 · 거부 · 병합 · 이름 변경 — and
// requires that a rejection's reason be recorded "다음 분석 시 참고될 수 있도록",
// which is what [`previous_rejection`] reads back.

const DECISION_UNDECIDED: &str = "undecided";
const DECISION_APPROVED: &str = "approved";
const DECISION_REJECTED: &str = "rejected";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CandidateView {
    key: String,
    name: String,
    location: String,
    symbol: Option<String>,
    rationale: String,
    decision: String,
    reject_reason: Option<String>,
    /// The surviving candidate's key when this one was merged away.
    merged_into: Option<String>,
    /// AC1.4's "거부된 후보의 사유는 다음 분석 시 참고될 수 있도록 기록된다", read
    /// back: an earlier analysis of the same target rejected this same place.
    /// Present as information, never as an automatic decision — the mockup is
    /// explicit that the reviewer decides again ("자동으로 다시 채택하지 않았으니").
    previously_rejected: Option<PreviousRejection>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PreviousRejection {
    reason: String,
    rejected_at: i64,
    analysis_id: String,
}

/// The Feature Candidates payload. `undecided` is a count rather than a client-side filter so
/// the screen and the "결정 끝" gate read the same number the server does.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CandidateListView {
    candidates: Vec<CandidateView>,
    undecided: usize,
    /// Whether stage 4 has produced its document yet. `false` is not an error —
    /// it is "this analysis has not got there".
    extracted: bool,
}

#[derive(sqlx::FromRow)]
struct CandidateRow {
    key: String,
    name: String,
    location: String,
    symbol: Option<String>,
    rationale: String,
    decision: String,
    reject_reason: Option<String>,
    merged_into: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecisionReq {
    key: String,
    decision: String,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameReq {
    key: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergeReq {
    /// The candidate that survives.
    into: String,
    /// The candidates folded into it.
    keys: Vec<String>,
}

/// The reviewable candidate list, seeding it from stage 4's document on first read.
///
/// Seeded lazily for the same reason the strategy is (AC1.3): the worker protocol
/// stays unchanged and the seed is idempotent — whoever reads first materialises
/// the rows, and every later read returns the rows that person decided on.
async fn candidates(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<CandidateListView>, AppError> {
    let (owner, name, branch, created_at) = owned_analysis(&state, &user.id, &id).await?;
    seed_candidates(&state, &id).await?;
    load_candidates(&state, &user.id, &id, &owner, &name, &branch, created_at)
        .await
        .map(Json)
}

async fn seed_candidates(state: &AppState, id: &str) -> Result<bool, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(id)
    .bind(pipeline::FEATURE_CANDIDATES)
    .fetch_optional(&state.db)
    .await?;
    let Some((content,)) = row else {
        return Ok(false);
    };
    let doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|_| AppError::internal("stored candidate document is unreadable"))?;

    let now = now_unix();
    for (seq, candidate) in crate::feature_candidates::candidates(&doc).iter().enumerate() {
        // `INSERT OR IGNORE` on `(analysis_id, key)`: re-running stage 4 (AC1.5's
        // partial retry) must not erase decisions the reviewer already made.
        sqlx::query(
            "INSERT OR IGNORE INTO feature_candidates \
               (id, analysis_id, key, seq, name, location, symbol, rationale, \
                decision, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(id)
        .bind(&candidate.key)
        .bind(seq as i64)
        .bind(&candidate.name)
        .bind(&candidate.location)
        .bind(candidate.symbol.as_deref())
        .bind(&candidate.rationale)
        .bind(DECISION_UNDECIDED)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await?;
    }
    Ok(true)
}

async fn load_candidates(
    state: &AppState,
    user_id: &str,
    id: &str,
    owner: &str,
    name: &str,
    branch: &str,
    created_at: i64,
) -> Result<CandidateListView, AppError> {
    let rows: Vec<CandidateRow> = sqlx::query_as(
        "SELECT key, name, location, symbol, rationale, decision, reject_reason, merged_into \
           FROM feature_candidates WHERE analysis_id = ? ORDER BY seq, rowid",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let mut candidates = Vec::with_capacity(rows.len());
    for row in rows {
        let previously_rejected =
            previous_rejection(state, user_id, id, owner, name, branch, created_at, &row.key)
                .await?;
        candidates.push(CandidateView {
            key: row.key,
            name: row.name,
            location: row.location,
            symbol: row.symbol,
            rationale: row.rationale,
            decision: row.decision,
            reject_reason: row.reject_reason,
            merged_into: row.merged_into,
            previously_rejected,
        });
    }
    let undecided = candidates
        .iter()
        .filter(|c| c.decision == DECISION_UNDECIDED && c.merged_into.is_none())
        .count();
    let extracted = !candidates.is_empty();
    Ok(CandidateListView {
        candidates,
        undecided,
        extracted,
    })
}

/// The most recent *earlier* rejection of this same candidate for the same target
/// (test/01 시나리오 7). Same ordering rule as [`carried_over`]: unix seconds tie for
/// analyses triggered together, so `rowid` breaks it.
#[allow(clippy::too_many_arguments)]
async fn previous_rejection(
    state: &AppState,
    user_id: &str,
    id: &str,
    owner: &str,
    name: &str,
    branch: &str,
    created_at: i64,
    key: &str,
) -> Result<Option<PreviousRejection>, AppError> {
    let row: Option<(String, i64, String)> = sqlx::query_as(
        "SELECT c.reject_reason, c.updated_at, prev.id \
           FROM feature_candidates c \
           JOIN analyses prev ON prev.id = c.analysis_id \
          WHERE c.key = ? AND c.decision = ? AND c.reject_reason IS NOT NULL \
            AND prev.user_id = ? AND prev.repo_owner = ? AND prev.repo_name = ? \
            AND prev.branch = ? AND prev.id != ? \
            AND (prev.created_at < ? \
                 OR (prev.created_at = ? \
                     AND prev.rowid < (SELECT rowid FROM analyses WHERE id = ?))) \
          ORDER BY prev.created_at DESC, prev.rowid DESC LIMIT 1",
    )
    .bind(key)
    .bind(DECISION_REJECTED)
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

    Ok(row.map(|(reason, rejected_at, analysis_id)| PreviousRejection {
        reason,
        rejected_at,
        analysis_id,
    }))
}

/// Approve or reject one candidate. A rejection without a reason is refused —
/// AC1.4 requires the reason, and a reason recorded later is a reason nobody wrote.
async fn decide_candidate(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(req): Json<DecisionReq>,
) -> Result<Json<CandidateListView>, AppError> {
    let (owner, name, branch, created_at) = owned_analysis(&state, &user.id, &id).await?;
    // Same lazy seed as the read: a reviewer who deep-links straight into a decision
    // must not have to have loaded the list first for it to exist.
    seed_candidates(&state, &id).await?;
    let key = req.key;

    let (decision, reason) = match req.decision.as_str() {
        "approve" => (DECISION_APPROVED, None),
        "reject" => {
            let reason = req.reason.unwrap_or_default().trim().to_string();
            if reason.is_empty() {
                return Err(AppError::BadRequest(
                    "사유를 적어야 거부를 확정할 수 있어요. 다음 분석에서 같은 판단을 반복하지 않으려면 한 줄이라도 남겨 주세요.".into(),
                ));
            }
            (DECISION_REJECTED, Some(reason))
        }
        _ => return Err(AppError::BadRequest("unknown decision".into())),
    };

    // The decision and the re-queue are one transaction: they are one fact. A
    // rejection opens nothing, so it re-queues nothing — an analysis whose every
    // candidate was rejected must not spin.
    let now = now_unix();
    let mut tx = state.db.begin().await?;
    let res = sqlx::query(
        "UPDATE feature_candidates SET decision = ?, reject_reason = ?, updated_at = ? \
          WHERE analysis_id = ? AND key = ? AND merged_into IS NULL",
    )
    .bind(decision)
    .bind(reason.as_deref())
    .bind(now)
    .bind(&id)
    .bind(&key)
    .execute(&mut *tx)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    if decision == DECISION_APPROVED {
        requeue(&mut tx, &id).await?;
    }
    tx.commit().await?;

    crate::audit::record(
        &state.db,
        Some(&user.id),
        "analysis.candidate_decision",
        Some(decision),
    )
    .await;

    load_candidates(&state, &user.id, &id, &owner, &name, &branch, created_at)
        .await
        .map(Json)
}

/// Rename one candidate (AC1.4). The key does not move — identity is where the
/// candidate was found, not what it is called, which is what lets the next analysis
/// still recognise a renamed candidate.
async fn rename_candidate(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(req): Json<RenameReq>,
) -> Result<Json<CandidateListView>, AppError> {
    let (owner, name, branch, created_at) = owned_analysis(&state, &user.id, &id).await?;
    seed_candidates(&state, &id).await?;
    let key = req.key;
    let renamed = req.name.trim().to_string();
    if renamed.is_empty() {
        return Err(AppError::BadRequest("이름을 비워 둘 수 없어요.".into()));
    }

    let res = sqlx::query(
        "UPDATE feature_candidates SET name = ?, updated_at = ? \
          WHERE analysis_id = ? AND key = ? AND merged_into IS NULL",
    )
    .bind(&renamed)
    .bind(now_unix())
    .bind(&id)
    .bind(&key)
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    load_candidates(&state, &user.id, &id, &owner, &name, &branch, created_at)
        .await
        .map(Json)
}

/// Merge candidates into one (AC1.4). The folded rows are kept, not deleted: the
/// merge stays visible and reversible, and the next analysis can still see that
/// those places were looked at.
async fn merge_candidates(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(req): Json<MergeReq>,
) -> Result<Json<CandidateListView>, AppError> {
    let (owner, name, branch, created_at) = owned_analysis(&state, &user.id, &id).await?;
    seed_candidates(&state, &id).await?;
    let folded: Vec<String> = req.keys.into_iter().filter(|k| *k != req.into).collect();
    if folded.is_empty() {
        return Err(AppError::BadRequest(
            "합칠 후보를 하나 이상 골라 주세요.".into(),
        ));
    }

    let survivor: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM feature_candidates \
          WHERE analysis_id = ? AND key = ? AND merged_into IS NULL",
    )
    .bind(&id)
    .bind(&req.into)
    .fetch_optional(&state.db)
    .await?;
    if survivor.is_none() {
        return Err(AppError::NotFound);
    }

    let mut tx = state.db.begin().await?;
    for key in &folded {
        // A folded candidate is no longer a thing to decide, so it stops counting
        // as undecided; its own decision is cleared to keep the two axes
        // (decided / merged away) from contradicting each other on the screen.
        let res = sqlx::query(
            "UPDATE feature_candidates \
                SET merged_into = ?, decision = ?, reject_reason = NULL, updated_at = ? \
              WHERE analysis_id = ? AND key = ? AND merged_into IS NULL",
        )
        .bind(&req.into)
        .bind(DECISION_UNDECIDED)
        .bind(now_unix())
        .bind(&id)
        .bind(key)
        .execute(&mut *tx)
        .await?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
    }
    tx.commit().await?;

    load_candidates(&state, &user.id, &id, &owner, &name, &branch, created_at)
        .await
        .map(Json)
}

// 의존성은 단계가 아니라 **feature 하나에 대한 요청**이라, 후보 승인이 5단계를 여는
// 것과 같은 방식으로 움직인다 — 행 하나를 남기고 분석을 재큐잉한다. 무엇을 돌릴지는
// 큐가 알고, 워커는 제안받은 것만 한다.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DependencyRequestReq {
    key: String,
}

#[derive(Deserialize)]
struct FeatureQuery {
    key: String,
}

#[derive(Deserialize)]
struct DependentsQuery {
    category: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DependencyView {
    category: String,
    name: String,
    /// `None` is the recorded fact 「근거 없음」, not a missing field.
    evidence: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DependencyGroupView {
    category: String,
    items: Vec<DependencyView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FeatureDependencyView {
    feature_key: String,
    feature_name: Option<String>,
    /// `None` until someone asks — a different thing from "asked and got nothing".
    status: Option<String>,
    error: Option<String>,
    /// Empty categories are carried too: "none of that kind was found" is an answer.
    categories: Vec<DependencyGroupView>,
    total: usize,
    without_evidence: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DependentFeatureView {
    analysis_id: String,
    repo_owner: String,
    repo_name: String,
    branch: String,
    feature_key: String,
    feature_name: Option<String>,
    evidence: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DependentsView {
    category: String,
    name: String,
    features: Vec<DependentFeatureView>,
}

#[derive(sqlx::FromRow)]
struct DependencyRow {
    category: String,
    name: String,
    evidence: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DependentRow {
    analysis_id: String,
    repo_owner: String,
    repo_name: String,
    branch: String,
    feature_key: String,
    feature_name: Option<String>,
    evidence: Option<String>,
}

/// Dependencies are traced for **confirmed** features only — an approved
/// candidate, or a feature the person added themselves and confirmed (AC3.2).
async fn approved_candidate_name(
    state: &AppState,
    analysis_id: &str,
    key: &str,
) -> Result<String, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM feature_candidates \
          WHERE analysis_id = ? AND key = ? AND decision = ? AND merged_into IS NULL",
    )
    .bind(analysis_id)
    .bind(key)
    .bind(DECISION_APPROVED)
    .fetch_optional(&state.db)
    .await?;
    if let Some((name,)) = row {
        return Ok(name);
    }
    crate::feature_add::confirmed_name(state, analysis_id, key)
        .await?
        .ok_or(AppError::NotFound)
}

async fn load_feature_dependencies(
    state: &AppState,
    analysis_id: &str,
    key: &str,
    feature_name: Option<String>,
) -> Result<FeatureDependencyView, AppError> {
    let request: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT status, error FROM feature_dependency_requests \
          WHERE analysis_id = ? AND feature_key = ?",
    )
    .bind(analysis_id)
    .bind(key)
    .fetch_optional(&state.db)
    .await?;

    let rows: Vec<DependencyRow> = sqlx::query_as(
        "SELECT category, name, evidence FROM feature_dependencies \
          WHERE analysis_id = ? AND feature_key = ? ORDER BY seq, rowid",
    )
    .bind(analysis_id)
    .bind(key)
    .fetch_all(&state.db)
    .await?;

    let total = rows.len();
    let without_evidence = rows.iter().filter(|r| r.evidence.is_none()).count();
    let categories = dependencies::CATEGORIES
        .iter()
        .map(|category| DependencyGroupView {
            category: (*category).to_string(),
            items: rows
                .iter()
                .filter(|r| r.category == **category)
                .map(|r| DependencyView {
                    category: r.category.clone(),
                    name: r.name.clone(),
                    evidence: r.evidence.clone(),
                })
                .collect(),
        })
        .collect();

    Ok(FeatureDependencyView {
        feature_key: key.to_string(),
        feature_name,
        status: request.as_ref().map(|(status, _)| status.clone()),
        error: request.and_then(|(_, error)| error),
        categories,
        total,
        without_evidence,
    })
}

/// What this feature depends on, as far as the last run got.
async fn feature_dependencies(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Query(query): Query<FeatureQuery>,
) -> Result<Json<FeatureDependencyView>, AppError> {
    owned_analysis(&state, &user.id, &id).await?;
    let name = approved_candidate_name(&state, &id, &query.key).await?;
    load_feature_dependencies(&state, &id, &query.key, Some(name))
        .await
        .map(Json)
}

/// Records the request and re-queues the analysis in **one transaction**: a request
/// that survives without the re-queue is a request nobody will ever run.
///
/// Re-asking after a failure resets the row to `queued`; the rows a previous
/// successful run stored stay until a new run replaces them.
async fn request_dependencies(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(req): Json<DependencyRequestReq>,
) -> Result<Json<FeatureDependencyView>, AppError> {
    owned_analysis(&state, &user.id, &id).await?;
    let name = approved_candidate_name(&state, &id, &req.key).await?;

    let now = now_unix();
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO feature_dependency_requests \
           (id, analysis_id, feature_key, status, error, requested_at, updated_at) \
         VALUES (?, ?, ?, ?, NULL, ?, ?) \
         ON CONFLICT(analysis_id, feature_key) DO UPDATE SET \
           status = excluded.status, error = NULL, updated_at = excluded.updated_at",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&id)
    .bind(&req.key)
    .bind(dependencies::request_status::QUEUED)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    requeue(&mut tx, &id).await?;
    tx.commit().await?;

    load_feature_dependencies(&state, &id, &req.key, Some(name))
        .await
        .map(Json)
}

/// Every feature of **this user's** analyses that depends on the named thing.
async fn dependents(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<DependentsQuery>,
) -> Result<Json<DependentsView>, AppError> {
    if !dependencies::is_category(&query.category) {
        return Err(AppError::BadRequest(
            "알 수 없는 분류예요. 의존성 분류는 7종 중 하나입니다.".into(),
        ));
    }
    let rows: Vec<DependentRow> = sqlx::query_as(
        "SELECT d.analysis_id AS analysis_id, a.repo_owner AS repo_owner, \
                a.repo_name AS repo_name, a.branch AS branch, \
                d.feature_key AS feature_key, c.name AS feature_name, d.evidence AS evidence \
           FROM feature_dependencies d \
           JOIN analyses a ON a.id = d.analysis_id \
           LEFT JOIN feature_candidates c \
                  ON c.analysis_id = d.analysis_id AND c.key = d.feature_key \
          WHERE a.user_id = ? AND d.category = ? AND d.name = ? \
          ORDER BY a.created_at DESC, d.analysis_id, d.feature_key",
    )
    .bind(&user.id)
    .bind(&query.category)
    .bind(&query.name)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(DependentsView {
        category: query.category,
        name: query.name,
        features: rows
            .into_iter()
            .map(|r| DependentFeatureView {
                analysis_id: r.analysis_id,
                repo_owner: r.repo_owner,
                repo_name: r.repo_name,
                branch: r.branch,
                feature_key: r.feature_key,
                feature_name: r.feature_name,
                evidence: r.evidence,
            })
            .collect(),
    }))
}

/// An attachment rather than a rendered view — export means a file leaves the
/// product, not that a screen shows JSON.
async fn export_dependencies(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let (owner, name, branch, _created_at) = owned_analysis(&state, &user.id, &id).await?;

    let rows: Vec<(String, Option<String>, String, String, Option<String>)> = sqlx::query_as(
        "SELECT d.feature_key, c.name, d.category, d.name, d.evidence \
           FROM feature_dependencies d \
           LEFT JOIN feature_candidates c \
                  ON c.analysis_id = d.analysis_id AND c.key = d.feature_key \
          WHERE d.analysis_id = ? \
          ORDER BY d.feature_key, d.seq, d.rowid",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await?;

    let mut features: Vec<serde_json::Value> = Vec::new();
    let mut current: Option<(String, Option<String>, Vec<serde_json::Value>)> = None;
    for (feature_key, feature_name, category, dep_name, evidence) in rows {
        match &mut current {
            Some((key, _, items)) if *key == feature_key => {
                items.push(serde_json::json!({
                    "category": category, "name": dep_name, "evidence": evidence,
                }));
            }
            _ => {
                if let Some((key, name, items)) = current.take() {
                    features.push(serde_json::json!({
                        "key": key, "name": name, "dependencies": items,
                    }));
                }
                current = Some((
                    feature_key,
                    feature_name,
                    vec![serde_json::json!({
                        "category": category, "name": dep_name, "evidence": evidence,
                    })],
                ));
            }
        }
    }
    if let Some((key, name, items)) = current.take() {
        features.push(serde_json::json!({
            "key": key, "name": name, "dependencies": items,
        }));
    }

    let body = serde_json::json!({
        "analysisId": id,
        "repository": format!("{owner}/{name}"),
        "branch": branch,
        "categories": dependencies::CATEGORIES,
        "features": features,
    });
    let filename = format!("dependencies-{id}.json");
    Ok((
        [(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )],
        Json(body),
    ))
}

/// 이번 재분석이 무엇을 바꿨는가.
///
/// 비교 대상은 사용자가 고르지 않는다 — 같은 타깃의 직전 분석이고, `(created_at,
/// rowid)` 로 정렬한다(같은 초에 만들어진 형제 분석이 있으면 `created_at` 만으로는
/// 진짜 선행을 놓친다).
///
/// 지난번에 있었는데 이번에 없는 feature 는 싣지 않는다 — 아직 후보를 확정하지
/// 않았다는 사실이 "기능이 지워졌다"로 읽히면 안 된다.
async fn analysis_diff(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<AnalysisDiffView>, AppError> {
    owned_analysis(&state, &user.id, &id).await?;

    let previous: Option<(String, i64)> = sqlx::query_as(
        "SELECT prev.id, prev.created_at \
           FROM analyses cur \
           JOIN analyses prev \
             ON prev.user_id = cur.user_id AND prev.repo_owner = cur.repo_owner \
            AND prev.repo_name = cur.repo_name AND prev.branch = cur.branch \
            AND (prev.created_at < cur.created_at \
                 OR (prev.created_at = cur.created_at AND prev.rowid < cur.rowid)) \
          WHERE cur.id = ? AND cur.user_id = ? \
          ORDER BY prev.created_at DESC, prev.rowid DESC LIMIT 1",
    )
    .bind(&id)
    .bind(&user.id)
    .fetch_optional(&state.db)
    .await?;

    let Some((previous_id, previous_created_at)) = previous else {
        return Ok(Json(AnalysisDiffView {
            compared_to: None,
            compared_to_created_at: None,
            changed_line_count: 0,
            features: Vec::new(),
        }));
    };

    let current_doc = acceptance_document(&state, &id).await?;
    let previous_doc = acceptance_document(&state, &previous_id).await?;
    let locations = candidate_locations(&state, &id).await?;

    let mut features = Vec::new();
    for key in diff::feature_keys(&current_doc) {
        let name = diff::feature_name(&current_doc, &key).unwrap_or_else(|| key.clone());
        let location = locations
            .iter()
            .find(|(candidate, _)| *candidate == key)
            .map(|(_, location)| location.clone());
        let entry = diff::feature_diff(
            &key,
            &name,
            location,
            &diff::scenarios_of(&previous_doc, &key),
            &diff::scenarios_of(&current_doc, &key),
            traced_dependencies(&state, &previous_id, &key).await?.as_deref(),
            traced_dependencies(&state, &id, &key).await?.as_deref(),
        );
        if let Some(entry) = entry {
            features.push(entry);
        }
    }

    Ok(Json(AnalysisDiffView {
        compared_to: Some(previous_id),
        compared_to_created_at: Some(previous_created_at),
        changed_line_count: features.iter().map(diff::FeatureDiff::lines).sum(),
        features,
    }))
}

/// 문서가 없는 것은 오류가 아니라 "그 시점에 아직 쓰인 시나리오가 없다"이므로 빈
/// 문서로 읽는다.
async fn acceptance_document(
    state: &AppState,
    analysis_id: &str,
) -> Result<serde_json::Value, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(analysis_id)
    .bind(pipeline::ACCEPTANCE_DEPENDENCIES)
    .fetch_optional(&state.db)
    .await?;
    Ok(row
        .and_then(|(content,)| serde_json::from_str(&content).ok())
        .unwrap_or_else(|| serde_json::json!({})))
}

async fn candidate_locations(
    state: &AppState,
    analysis_id: &str,
) -> Result<Vec<(String, String)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT key, location FROM feature_candidates WHERE analysis_id = ? ORDER BY seq, rowid",
    )
    .bind(analysis_id)
    .fetch_all(&state.db)
    .await?)
}

/// 요청이 없거나 아직 끝나지 않았으면 `None` — 묻지 않은 것과 물었더니 없던 것은
/// 다르다(0008 이 요청과 데이터를 두 테이블로 나눈 것과 같은 구분).
async fn traced_dependencies(
    state: &AppState,
    analysis_id: &str,
    feature_key: &str,
) -> Result<Option<Vec<diff::Dependency>>, AppError> {
    let request: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM feature_dependency_requests \
          WHERE analysis_id = ? AND feature_key = ?",
    )
    .bind(analysis_id)
    .bind(feature_key)
    .fetch_optional(&state.db)
    .await?;
    if request.map(|(status,)| status)
        != Some(dependencies::request_status::SUCCEEDED.to_string())
    {
        return Ok(None);
    }

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT category, name FROM feature_dependencies \
          WHERE analysis_id = ? AND feature_key = ? ORDER BY seq, rowid",
    )
    .bind(analysis_id)
    .bind(feature_key)
    .fetch_all(&state.db)
    .await?;
    Ok(Some(
        rows.into_iter()
            .map(|(category, name)| diff::Dependency { category, name })
            .collect(),
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisDiffView {
    /// 첫 분석이면 `None` — 「바뀐 게 없다」와 구분되어야 한다.
    compared_to: Option<String>,
    compared_to_created_at: Option<i64>,
    changed_line_count: usize,
    features: Vec<diff::FeatureDiff>,
}

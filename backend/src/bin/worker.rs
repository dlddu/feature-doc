//! The analysis worker (AC4.5) — a *separate workload* from the API.
//!
//! It owns no database and no volume: it polls the API's `/internal/*` queue
//! routes for a job, runs the stages that are implemented, reports each transition
//! back, and moves on. That is what lets it be scaled to zero (the API keeps
//! serving; jobs simply accumulate in the queue) and scaled out (SQLite serialises
//! the claim, so N workers take disjoint jobs) — test/04 scenarios 7 and 8.
//!
//! All five stages are implemented: `fetch` (measure the repository tree),
//! `cross_cutting` (AC1.2 — extract the repository's cross-cutting concerns with
//! the owner's LLM key), `discovery_strategy` (AC1.3 — propose where features can
//! be found), `feature_candidates` (AC1.4 — extract the candidates the approved
//! strategy points at) and `acceptance_dependencies` (AC2.1~AC2.3 — write the
//! acceptance scenarios for the features the reviewer confirmed). A job still lands
//! in `awaiting_pipeline` rather than `succeeded` whenever a stage behind a human
//! gate has not had its turn yet, because calling a half-run analysis complete would
//! be a lie the user reads as progress.
//!
//! Stages 3 and 4 only *propose*. AC1.3 requires that the user review, edit and
//! approve the strategy before it feeds the next stage, and AC2.1 is about a
//! **confirmed** feature — so both approvals are user actions on the API side and
//! the worker is never the thing that decides they are final. Each approval
//! re-queues the job, which is how the next stage gets its turn: this worker sees a
//! second claim of the same analysis whose `executableStages` no longer contain the
//! stages that already succeeded. It runs what it is offered and nothing else —
//! re-running stage 2 or 3 there would spend the owner's LLM budget again and
//! replace the very proposal the reviewer approved.

use std::time::Duration;

use featuredoc::acceptance;
use featuredoc::config::Mode;
use featuredoc::cross_cutting;
use featuredoc::discovery_strategy;
use featuredoc::feature_candidates;
use featuredoc::llm;
use featuredoc::pipeline;
use featuredoc::repo_scan;
use serde::Deserialize;
use serde_json::json;

/// Wait between polls when the queue is empty.
const IDLE_POLL: Duration = Duration::from_secs(2);
/// Back-off when the API is unreachable, so a restarting API is not hammered.
const ERROR_BACKOFF: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claim {
    id: String,
    repo_owner: String,
    repo_name: String,
    branch: String,
    executable_stages: Vec<String>,
    /// Stage 4's input: the patterns the reviewer approved (AC1.3). Empty until
    /// they have, which is also when stage 4 is not offered.
    #[serde(default)]
    approved_patterns: Vec<String>,
    /// Stage 5's input: the candidates the reviewer approved (AC1.4). Empty until
    /// they have, which is also when stage 5 is not offered.
    #[serde(default)]
    approved_candidates: Vec<ApprovedCandidate>,
    installation_token: Option<String>,
    llm_provider: Option<String>,
    llm_api_key: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApprovedCandidate {
    key: String,
    name: String,
    location: String,
    symbol: Option<String>,
}

struct Worker {
    http: reqwest::Client,
    api_base: String,
    token: String,
    worker_id: String,
    mode: Mode,
    github_api_base: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    featuredoc::init_tracing();

    let token = env_or("FEATUREDOC_WORKER_TOKEN", "");
    if token.is_empty() {
        anyhow::bail!("FEATUREDOC_WORKER_TOKEN is required — the worker cannot claim without it");
    }

    let worker = Worker {
        http: reqwest::Client::builder()
            .user_agent("featuredoc-worker/0.1")
            .timeout(Duration::from_secs(60))
            .build()?,
        api_base: trim_slash(&env_or("FEATUREDOC_API_BASE", "http://featuredoc:8080")),
        token,
        // k8s projects the pod name here; falls back to the hostname.
        worker_id: std::env::var("WORKER_ID")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                std::fs::read_to_string("/etc/hostname")
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "worker".to_string())
            }),
        mode: match env_or("FEATUREDOC_MODE", "real").to_ascii_lowercase().as_str() {
            "stub" => Mode::Stub,
            _ => Mode::Real,
        },
        github_api_base: trim_slash(&env_or("GITHUB_API_BASE", "https://api.github.com")),
    };

    tracing::info!(
        worker_id = %worker.worker_id,
        api_base = %worker.api_base,
        "featuredoc worker started"
    );

    // A plain loop, not a scheduler: the API is the queue, and the lease in
    // `worker_api` is what makes an abrupt SIGKILL recoverable.
    loop {
        match worker.claim().await {
            Ok(Some(job)) => {
                if let Err(e) = worker.run(&job).await {
                    tracing::warn!(analysis_id = %job.id, "job failed: {e}");
                }
            }
            Ok(None) => tokio::time::sleep(IDLE_POLL).await,
            Err(e) => {
                tracing::warn!("claim failed: {e}");
                tokio::time::sleep(ERROR_BACKOFF).await;
            }
        }
    }
}

impl Worker {
    async fn claim(&self) -> anyhow::Result<Option<Claim>> {
        let resp = self
            .post("/internal/analyses/claim", json!({ "workerId": self.worker_id }))
            .await?;
        if resp.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }
        if !resp.status().is_success() {
            anyhow::bail!("claim rejected ({})", resp.status().as_u16());
        }
        Ok(Some(resp.json::<Claim>().await?))
    }

    /// Runs the implemented stages of one claimed job.
    async fn run(&self, job: &Claim) -> anyhow::Result<()> {
        tracing::info!(
            analysis_id = %job.id,
            worker_id = %self.worker_id,
            repo = %format!("{}/{}", job.repo_owner, job.repo_name),
            "claimed analysis"
        );

        if !job.executable_stages.iter().any(|s| s == pipeline::FETCH) {
            // Nothing this build knows how to run; hand the job back rather than
            // hold a lease on it.
            self.finish(&job.id, "failed", Some("no executable stage in this build"))
                .await?;
            return Ok(());
        }

        self.stage(&job.id, pipeline::FETCH, "running", None, None).await?;
        // Extend the lease before the one network call of this stage, so a slow
        // GitHub response cannot let a second worker reclaim the job mid-flight.
        self.heartbeat(&job.id).await?;

        let scanned = repo_scan::scan(
            &self.http,
            self.mode,
            &self.github_api_base,
            &job.repo_owner,
            &job.repo_name,
            &job.branch,
            job.installation_token.as_deref(),
        )
        .await;

        let result = match scanned {
            Ok(result) => {
                self.stage(
                    &job.id,
                    pipeline::FETCH,
                    "succeeded",
                    Some(&result.detail()),
                    None,
                )
                .await?;
                tracing::info!(
                    analysis_id = %job.id,
                    files = result.files,
                    bytes = result.bytes,
                    "fetch stage complete"
                );
                result
            }
            Err(reason) => {
                self.stage(&job.id, pipeline::FETCH, "failed", None, Some(&reason))
                    .await?;
                self.finish(&job.id, "failed", Some(&reason)).await?;
                return Ok(());
            }
        };

        // Stage 2 runs only if this build knows it *and* the queue offered it, so an
        // older worker against a newer API (or the reverse) degrades to stopping
        // early rather than reporting a stage it cannot run.
        let mut cross_cutting_doc: Option<serde_json::Value> = None;
        if job
            .executable_stages
            .iter()
            .any(|s| s == pipeline::CROSS_CUTTING)
        {
            match self.run_cross_cutting(&job, &result.paths).await {
                Ok(doc) => cross_cutting_doc = Some(doc),
                Err(reason) => {
                    // The failure is this stage's, not the job's: `fetch` keeps its
                    // measured detail, and AC1.5's per-stage retry can re-run just
                    // this one once the cause (usually a missing key) is fixed.
                    self.stage(
                        &job.id,
                        pipeline::CROSS_CUTTING,
                        "failed",
                        None,
                        Some(&reason),
                    )
                    .await?;
                    self.finish(&job.id, "failed", Some(&reason)).await?;
                    return Ok(());
                }
            }
        }

        // Stage 3 (AC1.3) needs stage 2's document as its input, so it runs only
        // when stage 2 actually produced one in this pass.
        if job
            .executable_stages
            .iter()
            .any(|s| s == pipeline::DISCOVERY_STRATEGY)
        {
            if let Some(landscape) = cross_cutting_doc.as_ref() {
                if let Err(reason) = self
                    .run_discovery_strategy(&job, &result.paths, landscape)
                    .await
                {
                    self.stage(
                        &job.id,
                        pipeline::DISCOVERY_STRATEGY,
                        "failed",
                        None,
                        Some(&reason),
                    )
                    .await?;
                    self.finish(&job.id, "failed", Some(&reason)).await?;
                    return Ok(());
                }
            }
        }

        // Stage 4 (AC1.4). Offered only when the reviewer approved a strategy *and*
        // the stage has not already succeeded, so this arm runs on the claim that
        // follows approval. Its inputs are this pass's path list and the approved
        // patterns the claim carried — never an in-pass product of stages 2-3,
        // which is what lets those stages stay untouched here.
        if job
            .executable_stages
            .iter()
            .any(|s| s == pipeline::FEATURE_CANDIDATES)
        {
            if let Err(reason) = self.run_feature_candidates(job, &result.paths).await {
                self.stage(
                    &job.id,
                    pipeline::FEATURE_CANDIDATES,
                    "failed",
                    None,
                    Some(&reason),
                )
                .await?;
                self.finish(&job.id, "failed", Some(&reason)).await?;
                return Ok(());
            }
        }

        // Stage 5 (AC2.1~AC2.3). Offered only when the reviewer approved at least one
        // feature candidate *and* the stage has not already succeeded, so this arm
        // runs on the claim that follows that approval — the same shape as stage 4,
        // one gate further along.
        if job
            .executable_stages
            .iter()
            .any(|s| s == pipeline::ACCEPTANCE_DEPENDENCIES)
        {
            if let Err(reason) = self.run_acceptance(job, &result.paths).await {
                self.stage(
                    &job.id,
                    pipeline::ACCEPTANCE_DEPENDENCIES,
                    "failed",
                    None,
                    Some(&reason),
                )
                .await?;
                self.finish(&job.id, "failed", Some(&reason)).await?;
                return Ok(());
            }
        }

        // Not `succeeded`: a stage behind a human gate may still be waiting — stage 4
        // on the strategy approval, stage 5 on a confirmed feature. The API turns
        // this into `queued` when an approval arrived while this pass was running.
        self.finish(&job.id, "awaiting_pipeline", None).await?;
        Ok(())
    }

    /// Stage 5 (AC2.1~AC2.3). Same shape as stages 2-4: the caller owns the
    /// reporting so the "which stage failed" decision stays in one place.
    async fn run_acceptance(&self, job: &Claim, paths: &[String]) -> Result<(), String> {
        self.stage(&job.id, pipeline::ACCEPTANCE_DEPENDENCIES, "running", None, None)
            .await
            .map_err(|e| format!("could not report stage start: {e}"))?;
        self.heartbeat(&job.id)
            .await
            .map_err(|e| format!("could not renew lease: {e}"))?;

        let subjects: Vec<acceptance::Subject> = job
            .approved_candidates
            .iter()
            .map(|c| acceptance::Subject {
                key: c.key.clone(),
                name: c.name.clone(),
                location: c.location.clone(),
                symbol: c.symbol.clone(),
            })
            .collect();

        let answer = acceptance::derive(
            &self.http,
            self.mode,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            &job.repo_owner,
            &job.repo_name,
            &job.branch,
            paths,
            &subjects,
        )
        .await?;

        self.submit_document(&job.id, pipeline::ACCEPTANCE_DEPENDENCIES, &answer)
            .await
            .map_err(|e| format!("could not store the document: {e}"))?;
        self.stage(
            &job.id,
            pipeline::ACCEPTANCE_DEPENDENCIES,
            "succeeded",
            Some(&acceptance::detail(&answer.content)),
            None,
        )
        .await
        .map_err(|e| format!("could not report stage completion: {e}"))?;

        tracing::info!(
            analysis_id = %job.id,
            features = acceptance::features(&answer.content).len(),
            "acceptance stage complete"
        );
        Ok(())
    }

    /// Stage 4 (AC1.4). Same shape as stages 2 and 3: the caller owns the reporting
    /// so the "which stage failed" decision stays in one place.
    async fn run_feature_candidates(&self, job: &Claim, paths: &[String]) -> Result<(), String> {
        self.stage(&job.id, pipeline::FEATURE_CANDIDATES, "running", None, None)
            .await
            .map_err(|e| format!("could not report stage start: {e}"))?;
        self.heartbeat(&job.id)
            .await
            .map_err(|e| format!("could not renew lease: {e}"))?;

        let answer = feature_candidates::extract(
            &self.http,
            self.mode,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            &job.repo_owner,
            &job.repo_name,
            &job.branch,
            paths,
            &job.approved_patterns,
        )
        .await?;

        self.submit_document(&job.id, pipeline::FEATURE_CANDIDATES, &answer)
            .await
            .map_err(|e| format!("could not store the document: {e}"))?;
        self.stage(
            &job.id,
            pipeline::FEATURE_CANDIDATES,
            "succeeded",
            Some(&feature_candidates::detail(&answer.content)),
            None,
        )
        .await
        .map_err(|e| format!("could not report stage completion: {e}"))?;

        tracing::info!(
            analysis_id = %job.id,
            candidates = feature_candidates::candidates(&answer.content).len(),
            "feature candidates stage complete"
        );
        Ok(())
    }

    /// Stage 3 (AC1.3). Same shape as stage 2: the caller owns the reporting so the
    /// "which stage failed" decision stays in one place.
    async fn run_discovery_strategy(
        &self,
        job: &Claim,
        paths: &[String],
        landscape: &serde_json::Value,
    ) -> Result<(), String> {
        self.stage(&job.id, pipeline::DISCOVERY_STRATEGY, "running", None, None)
            .await
            .map_err(|e| format!("could not report stage start: {e}"))?;
        self.heartbeat(&job.id)
            .await
            .map_err(|e| format!("could not renew lease: {e}"))?;

        let answer = discovery_strategy::propose(
            &self.http,
            self.mode,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            &job.repo_owner,
            &job.repo_name,
            &job.branch,
            paths,
            landscape,
        )
        .await?;

        self.submit_document(&job.id, pipeline::DISCOVERY_STRATEGY, &answer)
            .await
            .map_err(|e| format!("could not store the document: {e}"))?;
        self.stage(
            &job.id,
            pipeline::DISCOVERY_STRATEGY,
            "succeeded",
            Some(&discovery_strategy::detail(&answer.content)),
            None,
        )
        .await
        .map_err(|e| format!("could not report stage completion: {e}"))?;

        tracing::info!(
            analysis_id = %job.id,
            entries = discovery_strategy::patterns(&answer.content).len(),
            "discovery strategy stage complete"
        );
        Ok(())
    }

    /// Which provider this job's key belongs to. Shared by every LLM-backed stage so
    /// they cannot disagree about it mid-job.
    fn provider_for(&self, job: &Claim) -> Result<llm::Provider, String> {
        match (self.mode, job.llm_provider.as_deref()) {
            // Stub mode never reaches a provider, so an absent key is not a failure.
            (Mode::Stub, other) => Ok(other
                .and_then(llm::Provider::parse)
                .unwrap_or(llm::DEFAULT_PROVIDER)),
            (Mode::Real, Some(p)) => llm::Provider::parse(p)
                .ok_or_else(|| format!("unsupported LLM provider registered: {p}")),
            (Mode::Real, None) => {
                Err("no active LLM key for this user; register one to analyze".to_string())
            }
        }
    }

    /// Stage 2 (AC1.2). Returns the extracted document (stage 3's input) on success
    /// and the reason string on failure, so the caller owns the reporting and the
    /// "which stage failed" decision stays in one place.
    async fn run_cross_cutting(
        &self,
        job: &Claim,
        paths: &[String],
    ) -> Result<serde_json::Value, String> {
        self.stage(&job.id, pipeline::CROSS_CUTTING, "running", None, None)
            .await
            .map_err(|e| format!("could not report stage start: {e}"))?;
        // The model call is the long one in this job; renew before it as `fetch` does.
        self.heartbeat(&job.id)
            .await
            .map_err(|e| format!("could not renew lease: {e}"))?;

        let answer = cross_cutting::extract(
            &self.http,
            self.mode,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            &job.repo_owner,
            &job.repo_name,
            &job.branch,
            paths,
        )
        .await?;

        self.submit_document(&job.id, pipeline::CROSS_CUTTING, &answer)
            .await
            .map_err(|e| format!("could not store the document: {e}"))?;
        self.stage(
            &job.id,
            pipeline::CROSS_CUTTING,
            "succeeded",
            Some(&cross_cutting::detail(&answer.content)),
            None,
        )
        .await
        .map_err(|e| format!("could not report stage completion: {e}"))?;

        tracing::info!(
            analysis_id = %job.id,
            model = %answer.model,
            "cross-cutting stage complete"
        );
        Ok(answer.content)
    }

    async fn submit_document(
        &self,
        id: &str,
        kind: &str,
        answer: &llm::Answer,
    ) -> anyhow::Result<()> {
        let body = json!({
            "workerId": self.worker_id,
            "content": answer.content,
            "model": answer.model,
            "inputTokens": answer.input_tokens,
            "outputTokens": answer.output_tokens,
        });
        self.expect_ok(
            self.post(&format!("/internal/analyses/{id}/documents/{kind}"), body)
                .await?,
            "document submit",
        )
    }

    async fn stage(
        &self,
        id: &str,
        key: &str,
        status: &str,
        detail: Option<&str>,
        error: Option<&str>,
    ) -> anyhow::Result<()> {
        let body = json!({
            "workerId": self.worker_id,
            "status": status,
            "detail": detail,
            "error": error,
        });
        self.expect_ok(
            self.post(&format!("/internal/analyses/{id}/stages/{key}"), body)
                .await?,
            "stage report",
        )
    }

    async fn heartbeat(&self, id: &str) -> anyhow::Result<()> {
        self.expect_ok(
            self.post(
                &format!("/internal/analyses/{id}/heartbeat"),
                json!({ "workerId": self.worker_id }),
            )
            .await?,
            "heartbeat",
        )
    }

    async fn finish(&self, id: &str, status: &str, error: Option<&str>) -> anyhow::Result<()> {
        let body = json!({ "workerId": self.worker_id, "status": status, "error": error });
        self.expect_ok(
            self.post(&format!("/internal/analyses/{id}/finish"), body)
                .await?,
            "finish",
        )
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> anyhow::Result<reqwest::Response> {
        Ok(self
            .http
            .post(format!("{}{path}", self.api_base))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?)
    }

    fn expect_ok(&self, resp: reqwest::Response, what: &str) -> anyhow::Result<()> {
        if resp.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("{what} rejected ({})", resp.status().as_u16())
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn trim_slash(s: &str) -> String {
    s.trim_end_matches('/').to_string()
}

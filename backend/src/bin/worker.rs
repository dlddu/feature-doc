//! The analysis worker — a *separate workload* from the API.
//!
//! It owns no database and no volume, which is what lets it be scaled to zero (the
//! API keeps serving; jobs simply accumulate in the queue) and scaled out: SQLite
//! serialises the claim, so N workers take disjoint jobs.
//!
//! Every approval re-queues the job, and this worker then sees a second claim of the
//! same analysis whose `executableStages` no longer contain the stages that already
//! succeeded. It runs what it is offered and nothing else — re-running an earlier
//! stage there would spend the owner's LLM budget again and replace the very
//! proposal the reviewer approved.

use std::time::Duration;

use featuredoc::acceptance;
use featuredoc::config::Mode;
use featuredoc::cross_cutting;
use featuredoc::dependencies;
use featuredoc::discovery_strategy;
use featuredoc::feature_candidates;
use featuredoc::llm;
use featuredoc::pipeline;
use featuredoc::repo_scan;
use serde::Deserialize;
use serde_json::json;
use tokio::signal::unix::{signal, SignalKind};

/// Default pause after an empty claim. `FEATUREDOC_WORKER_IDLE_POLL_MS` overrides
/// it — the e2e overlay lowers it because every queue hand-off in a spec otherwise
/// waits out up to this long.
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
    /// Stage 2's stored document, present when stage 3 is re-run on its own.
    #[serde(default)]
    cross_cutting_document: Option<serde_json::Value>,
    /// Empty until the reviewer approves — which is also when stage 4 is not offered.
    #[serde(default)]
    approved_patterns: Vec<String>,
    /// Empty until the reviewer approves — which is also when stage 5 is not offered.
    #[serde(default)]
    approved_candidates: Vec<ApprovedCandidate>,
    /// Empty when nobody asked — the request row is the gate.
    #[serde(default)]
    dependency_requests: Vec<ApprovedCandidate>,
    installation_token: Option<String>,
    llm_provider: Option<String>,
    llm_api_key: Option<String>,
    #[serde(default)]
    llm_language: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApprovedCandidate {
    key: String,
    name: String,
    location: String,
    symbol: Option<String>,
}

/// Which external boundaries this worker answers with a test double.
///
/// Selected here rather than shared with `config::Doubles`, which is what keeps one
/// process from being able to enable a double it does not itself run.
struct WorkerDoubles {
    repo_scan: Mode,
    llm: Mode,
}

struct Worker {
    http: reqwest::Client,
    api_base: String,
    token: String,
    worker_id: String,
    doubles: WorkerDoubles,
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
        worker_id: std::env::var("WORKER_ID")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                std::fs::read_to_string("/etc/hostname")
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "worker".to_string())
            }),
        doubles: WorkerDoubles {
            repo_scan: Mode::from_env("FEATUREDOC_DOUBLE_REPO_SCAN"),
            llm: Mode::from_env("FEATUREDOC_DOUBLE_LLM"),
        },
        github_api_base: trim_slash(&env_or("GITHUB_API_BASE", "https://api.github.com")),
    };

    tracing::info!(
        worker_id = %worker.worker_id,
        api_base = %worker.api_base,
        "featuredoc worker started"
    );

    // The worker is its container's PID 1, and the kernel drops any signal sent to
    // a PID-namespace init that has no handler for it. Without these a scale-down's
    // SIGTERM is ignored and the kubelet waits out the whole grace period (30 s by
    // default) before its SIGKILL. Installed before the first claim so that a stop
    // landing during it is still seen.
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    let stop = async move {
        tokio::select! {
            _ = sigterm.recv() => "SIGTERM",
            _ = sigint.recv() => "SIGINT",
        }
    };
    tokio::pin!(stop);

    // A plain loop, not a scheduler: the API is the queue, and the lease in
    // `worker_api` is what makes an abrupt SIGKILL recoverable. A stop is honoured
    // only between claims: cancelling a claim in flight could leave a job leased to
    // a worker that never saw it, and abandoning a job strands its lease for
    // LEASE_SECONDS. So the current job runs on — to its end, or to the grace
    // period's SIGKILL, which the lease still covers — and nothing new is claimed.
    let idle_poll = idle_poll();
    loop {
        let pause = match worker.claim().await {
            Ok(Some(job)) => {
                if let Err(e) = worker.run(&job).await {
                    tracing::warn!(analysis_id = %job.id, "job failed: {e}");
                }
                Duration::ZERO
            }
            Ok(None) => idle_poll,
            Err(e) => {
                tracing::warn!("claim failed: {e}");
                ERROR_BACKOFF
            }
        };
        tokio::select! {
            // A stop that arrived during a job must win over the zero pause after it.
            biased;
            sig = &mut stop => {
                tracing::info!(
                    worker_id = %worker.worker_id,
                    signal = sig,
                    "stop requested, exiting between claims"
                );
                return Ok(());
            }
            _ = tokio::time::sleep(pause) => {}
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
            self.doubles.repo_scan,
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
        let mut cross_cutting_doc: Option<serde_json::Value> = job.cross_cutting_document.clone();
        if job
            .executable_stages
            .iter()
            .any(|s| s == pipeline::CROSS_CUTTING)
        {
            match self.run_cross_cutting(&job, &result.paths).await {
                Ok(doc) => cross_cutting_doc = Some(doc),
                Err(reason) => {
                    // The failure is this stage's, not the job's: `fetch` keeps its
                    // measured detail, so a per-stage retry can re-run just this one
                    // once the cause (usually a missing key) is fixed.
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

        // The inputs here are this pass's path list and the approved patterns the
        // claim carried — never an in-pass product of the earlier stages, which is
        // what lets those stay untouched on the claim that follows an approval.
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

        // One feature's failure does not kill the job: its request row carries the
        // reason and the other features' results are not held hostage.
        for feature in &job.dependency_requests {
            if let Err(reason) = self.run_dependencies(job, &result.paths, feature).await {
                tracing::warn!(
                    analysis_id = %job.id,
                    feature = %feature.key,
                    "dependency trace failed: {reason}"
                );
                self.report_dependencies(
                    &job.id,
                    &feature.key,
                    dependencies::request_status::FAILED,
                    Some(&reason),
                    None,
                )
                .await?;
            }
        }

        // Not `succeeded`: a stage behind a human gate may still be waiting — stage 4
        // on the strategy approval, stage 5 on a confirmed feature. The API turns
        // this into `queued` when an approval arrived while this pass was running.
        self.finish(&job.id, "awaiting_pipeline", None).await?;
        Ok(())
    }

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
            self.doubles.llm,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            self.language_for(job)?,
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

    /// One feature's end-to-end dependencies — reported through its own route, so
    /// there is no stage row to move and no document kind to write.
    async fn run_dependencies(
        &self,
        job: &Claim,
        paths: &[String],
        feature: &ApprovedCandidate,
    ) -> Result<(), String> {
        self.heartbeat(&job.id)
            .await
            .map_err(|e| format!("could not renew lease: {e}"))?;

        let subject = acceptance::Subject {
            key: feature.key.clone(),
            name: feature.name.clone(),
            location: feature.location.clone(),
            symbol: feature.symbol.clone(),
        };
        let answer = dependencies::derive(
            &self.http,
            self.doubles.llm,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            self.language_for(job)?,
            &job.repo_owner,
            &job.repo_name,
            &job.branch,
            paths,
            &subject,
        )
        .await?;

        let detail = dependencies::detail(&answer.content);
        self.report_dependencies(
            &job.id,
            &feature.key,
            dependencies::request_status::SUCCEEDED,
            None,
            Some(&answer),
        )
        .await
        .map_err(|e| format!("could not store the dependencies: {e}"))?;

        tracing::info!(
            analysis_id = %job.id,
            feature = %feature.key,
            detail = %detail,
            "dependency trace complete"
        );
        Ok(())
    }

    async fn report_dependencies(
        &self,
        id: &str,
        feature_key: &str,
        status: &str,
        error: Option<&str>,
        answer: Option<&llm::Answer>,
    ) -> anyhow::Result<()> {
        let body = json!({
            "workerId": self.worker_id,
            "featureKey": feature_key,
            "status": status,
            "error": error,
            "content": answer
                .map(|a| a.content.clone())
                .unwrap_or_else(|| json!({ "items": [] })),
            "model": answer.map(|a| a.model.clone()),
            "inputTokens": answer.map_or(0, |a| a.input_tokens),
            "outputTokens": answer.map_or(0, |a| a.output_tokens),
        });
        self.expect_ok(
            self.post(&format!("/internal/analyses/{id}/dependencies"), body)
                .await?,
            "dependency submit",
        )
    }

    async fn run_feature_candidates(&self, job: &Claim, paths: &[String]) -> Result<(), String> {
        self.stage(&job.id, pipeline::FEATURE_CANDIDATES, "running", None, None)
            .await
            .map_err(|e| format!("could not report stage start: {e}"))?;
        self.heartbeat(&job.id)
            .await
            .map_err(|e| format!("could not renew lease: {e}"))?;

        let answer = feature_candidates::extract(
            &self.http,
            self.doubles.llm,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            self.language_for(job)?,
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
            self.doubles.llm,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            self.language_for(job)?,
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
    ///
    /// An active key is the product's entry condition for an analysis, and it stays
    /// the entry condition when the LLM double is on — refusing here is what keeps
    /// the stubbed path from being more permissive than the real one.
    fn provider_for(&self, job: &Claim) -> Result<llm::Provider, String> {
        match job.llm_provider.as_deref() {
            Some(p) => llm::Provider::parse(p)
                .ok_or_else(|| format!("unsupported LLM provider registered: {p}")),
            None => Err("no active LLM key for this user; register one to analyze".to_string()),
        }
    }

    fn language_for(&self, job: &Claim) -> Result<Option<llm::Language>, String> {
        match job.llm_language.as_deref() {
            None => Ok(None),
            Some(l) => llm::Language::parse(l)
                .map(Some)
                .ok_or_else(|| format!("unsupported output language on this analysis: {l}")),
        }
    }

    /// Like every stage runner here, it leaves failure reporting to the caller so the
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

        let excerpts = repo_scan::read_files(
            &self.http,
            self.doubles.repo_scan,
            &self.github_api_base,
            &job.repo_owner,
            &job.repo_name,
            &job.branch,
            job.installation_token.as_deref(),
            &cross_cutting::key_files(paths),
            cross_cutting::MAX_EXCERPT_BYTES,
        )
        .await?;
        // Reading a handful of files is several round trips of its own.
        self.heartbeat(&job.id)
            .await
            .map_err(|e| format!("could not renew lease: {e}"))?;

        let answer = cross_cutting::extract(
            &self.http,
            self.doubles.llm,
            self.provider_for(job)?,
            job.llm_api_key.as_deref(),
            self.language_for(job)?,
            &job.repo_owner,
            &job.repo_name,
            &job.branch,
            paths,
            &excerpts,
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
            excerpts = excerpts.len(),
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

fn idle_poll() -> Duration {
    const KEY: &str = "FEATUREDOC_WORKER_IDLE_POLL_MS";
    match std::env::var(KEY) {
        Err(_) => IDLE_POLL,
        Ok(raw) => match raw.parse::<u64>() {
            Ok(ms) if ms > 0 => Duration::from_millis(ms),
            _ => {
                tracing::warn!("{KEY}={raw:?} is not a positive integer; using the default");
                IDLE_POLL
            }
        },
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn trim_slash(s: &str) -> String {
    s.trim_end_matches('/').to_string()
}

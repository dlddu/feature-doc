//! The analysis worker (AC4.5) — a *separate workload* from the API.
//!
//! It owns no database and no volume: it polls the API's `/internal/*` queue
//! routes for a job, runs the stages that are implemented, reports each transition
//! back, and moves on. That is what lets it be scaled to zero (the API keeps
//! serving; jobs simply accumulate in the queue) and scaled out (SQLite serialises
//! the claim, so N workers take disjoint jobs) — test/04 scenarios 7 and 8.
//!
//! Today exactly one stage is implemented: `fetch` (measure the repository tree).
//! Stages 2-5 stay `pending` and the job lands in `awaiting_pipeline` rather than
//! `succeeded`, because pretending an unimplemented stage ran would be a lie the
//! user reads as progress.

use std::time::Duration;

use featuredoc::config::Mode;
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
    installation_token: Option<String>,
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

        match scanned {
            Ok(result) => {
                self.stage(
                    &job.id,
                    pipeline::FETCH,
                    "succeeded",
                    Some(&result.detail()),
                    None,
                )
                .await?;
                // Not `succeeded`: stages 2-5 are still unimplemented (AC1.2~1.4).
                self.finish(&job.id, "awaiting_pipeline", None).await?;
                tracing::info!(
                    analysis_id = %job.id,
                    files = result.files,
                    bytes = result.bytes,
                    "fetch stage complete"
                );
            }
            Err(reason) => {
                self.stage(&job.id, pipeline::FETCH, "failed", None, Some(&reason))
                    .await?;
                self.finish(&job.id, "failed", Some(&reason)).await?;
            }
        }
        Ok(())
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

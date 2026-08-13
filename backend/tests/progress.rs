//! Async progress and partial retry (AC1.5).
//!
//! The two things S04 needs from the API: read the persisted progress of one
//! analysis, and re-run a single failed stage. Both run against the router
//! in-process, so the contract is gated by `cargo test`; the screen that renders it
//! is asserted by `e2e/tests/ac1-5-async-progress-and-partial-retry.spec.ts`.
//!
//! Stage transitions here are driven through the worker's own `/internal` routes
//! rather than by writing rows directly — a fixture that hand-wrote
//! `analysis_stages` could drift from what a worker actually produces.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

const WORKER: &str = "w-progress";

async fn login_installed(state: &AppState, github_id: i64, login: &str) -> String {
    let gh = GithubUser {
        id: github_id,
        login: login.into(),
        name: None,
        avatar_url: None,
    };
    let user = users::upsert(&state.db, &gh).await.unwrap();
    installations::upsert(
        &state.db,
        &user.id,
        &installations::NewInstallation {
            installation_id: 4242,
            account_login: Some("stub-account"),
            account_type: Some("User"),
            repository_selection: Some("selected"),
        },
    )
    .await
    .unwrap();
    session::create(&state.db, &user.id).await.unwrap()
}

fn get(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .body(Body::empty())
        .unwrap()
}

fn user_post(uri: &str, token: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn worker_post(uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {WORKER_TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn enqueue(state: &AppState, session: &str, repo: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            "/api/analyses",
            session,
            serde_json::json!({ "repoUrl": format!("stub-account/{repo}") }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    json_body(resp).await["id"].as_str().unwrap().to_string()
}

async fn detail(state: &AppState, session: &str, id: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

/// The stage entry with the given key, from a detail payload.
fn stage<'a>(detail: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    detail["stages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["key"] == key)
        .unwrap_or_else(|| panic!("no `{key}` stage in {detail}"))
}

/// Takes the job off the queue as a worker would (the only way to hold a lease).
async fn claim(state: &AppState) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            serde_json::json!({ "workerId": WORKER }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "queue should hold a job");
    json_body(resp).await
}

async fn report(state: &AppState, id: &str, key: &str, body: serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/{key}"),
            body,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

async fn finish(state: &AppState, id: &str, status: &str, error: Option<&str>) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/finish"),
            serde_json::json!({ "workerId": WORKER, "status": status, "error": error }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

/// Drives one analysis to "stage 1 succeeded, stage 2 failed, job failed" — the
/// state test/01 시나리오 6 starts from (earlier stages done, one stage failed).
async fn run_until_second_stage_fails(state: &AppState, id: &str) {
    let claimed = claim(state).await;
    assert_eq!(claimed["id"], id);
    report(
        state,
        id,
        "fetch",
        serde_json::json!({ "workerId": WORKER, "status": "succeeded", "detail": "766 files · 2.2 MB" }),
    )
    .await;
    report(
        state,
        id,
        "cross_cutting",
        serde_json::json!({ "workerId": WORKER, "status": "failed", "error": "llm call limit exceeded" }),
    )
    .await;
    finish(state, id, "failed", Some("llm call limit exceeded")).await;
}

// ── read: S04's progress ──────────────────────────────────────────────────────

#[tokio::test]
async fn detail_reports_every_pipeline_stage_of_the_owner_s_analysis() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &token, "payments-api").await;

    let body = detail(&state, &token, &id).await;
    assert_eq!(body["status"], "queued");
    assert_eq!(body["stagesTotal"], 5);
    assert_eq!(body["stagesDone"], 0);

    let stages = body["stages"].as_array().unwrap();
    assert_eq!(stages.len(), 5, "S04 renders one card per pipeline stage");
    assert_eq!(stages[0]["seq"], 1);
    assert_eq!(stages[0]["key"], "fetch");
    assert_eq!(stages[0]["title"], "Fetch repository");
    assert!(
        stages.iter().all(|s| s["status"] == "pending"),
        "nothing has run before a worker claims the job"
    );
    let _ = std::fs::remove_file(&path);
}

/// Another user's analysis is `404`, not `403` — the API does not confirm that an
/// id it will not serve exists (AC4.7).
#[tokio::test]
async fn detail_is_scoped_to_the_owner() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "alice").await;
    let other = login_installed(&state, 2, "bob").await;
    let id = enqueue(&state, &owner, "payments-api").await;

    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}"), &other))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let _ = std::fs::remove_file(&path);
}

/// test/01 시나리오 5 — the app is closed mid-analysis and reopened. Progress is
/// server state, so a second read returns exactly what the first one did, including
/// the work a worker did in between.
#[tokio::test]
async fn progress_is_persisted_so_a_later_read_sees_the_same_run() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &token, "payments-api").await;

    let before = detail(&state, &token, &id).await;
    assert_eq!(before["stagesDone"], 0);

    let claimed = claim(&state).await;
    assert_eq!(claimed["id"], id);
    report(
        &state,
        &id,
        "fetch",
        serde_json::json!({ "workerId": WORKER, "status": "succeeded", "detail": "766 files · 2.2 MB" }),
    )
    .await;
    finish(&state, &id, "awaiting_pipeline", None).await;

    let reopened = detail(&state, &token, &id).await;
    assert_eq!(reopened["status"], "awaiting_pipeline");
    assert_eq!(reopened["stagesDone"], 1);
    assert_eq!(stage(&reopened, "fetch")["status"], "succeeded");
    assert_eq!(stage(&reopened, "fetch")["detail"], "766 files · 2.2 MB");
    assert!(stage(&reopened, "fetch")["finishedAt"].is_number());

    // Reading again changes nothing — the screen has no state of its own to lose.
    let again = detail(&state, &token, &id).await;
    assert_eq!(again, reopened, "a re-entry must show the same progress");

    // And the home list carries the same fraction, so S02 can say "1 of 5".
    let list = json_body(
        build_router(state.clone())
            .oneshot(get("/api/analyses", &token))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(list[0]["stagesDone"], 1);
    assert_eq!(list[0]["stagesTotal"], 5);
    let _ = std::fs::remove_file(&path);
}

// ── partial retry ─────────────────────────────────────────────────────────────

/// test/01 시나리오 6 — retrying the failed stage leaves the finished ones alone and
/// puts the job back where a worker will pick it up again.
#[tokio::test]
async fn retry_resets_only_the_failed_stage_and_requeues_the_job() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &token, "payments-api").await;
    run_until_second_stage_fails(&state, &id).await;

    let failed = detail(&state, &token, &id).await;
    assert_eq!(failed["status"], "failed");
    assert_eq!(stage(&failed, "cross_cutting")["error"], "llm call limit exceeded");
    let fetch_before = stage(&failed, "fetch").clone();

    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/stages/cross_cutting/retry"),
            &token,
            serde_json::json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let after = json_body(resp).await;

    // The retried stage is clean again…
    let retried = stage(&after, "cross_cutting");
    assert_eq!(retried["status"], "pending");
    assert!(retried["error"].is_null());
    assert!(retried["startedAt"].is_null());
    assert!(retried["finishedAt"].is_null());

    // …the stage that already succeeded is untouched, measurement and all…
    assert_eq!(stage(&after, "fetch"), &fetch_before);
    assert_eq!(after["stagesDone"], 1);

    // …the stages that never ran are still waiting…
    for key in ["discovery_strategy", "feature_candidates", "acceptance_dependencies"] {
        assert_eq!(stage(&after, key)["status"], "pending");
    }

    // …and the job is queued again, so the existing worker path re-runs it.
    assert_eq!(after["status"], "queued");
    assert!(after["error"].is_null());
    let reclaimed = claim(&state).await;
    assert_eq!(reclaimed["id"], id, "a re-queued job is claimable again");
    let _ = std::fs::remove_file(&path);
}

/// "이 단계만 다시 시도" is an offer the failed stage makes; a stage that is pending
/// or succeeded has nothing to retry, and re-queueing on its behalf would silently
/// re-run work the user did not ask for.
#[tokio::test]
async fn retry_is_refused_for_a_stage_that_did_not_fail() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &token, "payments-api").await;
    run_until_second_stage_fails(&state, &id).await;

    for key in ["fetch", "discovery_strategy"] {
        let resp = build_router(state.clone())
            .oneshot(user_post(
                &format!("/api/analyses/{id}/stages/{key}/retry"),
                &token,
                serde_json::json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT, "{key} did not fail");
    }

    // The refusals changed nothing.
    let body = detail(&state, &token, &id).await;
    assert_eq!(body["status"], "failed");
    assert_eq!(stage(&body, "fetch")["status"], "succeeded");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn retry_rejects_an_unknown_stage_key() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &token, "payments-api").await;

    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/stages/not-a-stage/retry"),
            &token,
            serde_json::json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn retry_is_scoped_to_the_owner() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "alice").await;
    let other = login_installed(&state, 2, "bob").await;
    let id = enqueue(&state, &owner, "payments-api").await;
    run_until_second_stage_fails(&state, &id).await;

    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/stages/cross_cutting/retry"),
            &other,
            serde_json::json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // The owner's job is untouched by the refused retry.
    let body = detail(&state, &owner, &id).await;
    assert_eq!(body["status"], "failed");
    assert_eq!(stage(&body, "cross_cutting")["status"], "failed");
    let _ = std::fs::remove_file(&path);
}

/// A worker still inside its lease owns the run. Re-queueing under it would let the
/// holder keep reporting into a run the user just reset.
#[tokio::test]
async fn retry_is_refused_while_a_worker_still_holds_the_job() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &token, "payments-api").await;

    let claimed = claim(&state).await;
    assert_eq!(claimed["id"], id);
    report(
        &state,
        &id,
        "fetch",
        serde_json::json!({ "workerId": WORKER, "status": "failed", "error": "github tree rejected (404)" }),
    )
    .await;
    // Deliberately no `finish`: the job is still `running` under an unexpired lease.

    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/stages/fetch/retry"),
            &token,
            serde_json::json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    let body = detail(&state, &token, &id).await;
    assert_eq!(body["status"], "running");
    assert_eq!(stage(&body, "fetch")["status"], "failed");
    let _ = std::fs::remove_file(&path);
}

//! The worker queue protocol (AC4.5): authentication, atomic claim, lease
//! reclaim, and stage reporting.
//!
//! These run against the router in-process, so the safety properties that make
//! horizontal scaling correct are gated by `cargo test` — no cluster required.
//! The cluster-level half (worker pods actually scaled to 0 and to 2) is asserted
//! by `e2e/tests/ac4-5-worker-workload-separation.spec.ts`.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

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

fn user_post(uri: &str, token: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn worker_post(uri: &str, bearer: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {bearer}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Enqueues one analysis for `repo` and returns its id.
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

async fn claim(state: &AppState, worker_id: &str) -> axum::response::Response {
    build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            WORKER_TOKEN,
            serde_json::json!({ "workerId": worker_id }),
        ))
        .await
        .unwrap()
}

// ── authentication ────────────────────────────────────────────────────────────

#[tokio::test]
async fn internal_routes_reject_without_or_with_a_wrong_token() {
    let (state, _p) = stub_state().await;

    let no_auth = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/analyses/claim")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"workerId":"w1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(no_auth.status(), StatusCode::UNAUTHORIZED);

    let wrong = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            "not-the-token",
            serde_json::json!({ "workerId": "w1" }),
        ))
        .await
        .unwrap();
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
}

/// A deployment with no worker secret must not expose the queue at all — an unset
/// token is "closed", never "open".
#[tokio::test]
async fn internal_routes_are_closed_when_no_worker_token_is_configured() {
    let (mut state, _p) = stub_state().await;
    let mut config = (*state.config).clone();
    config.worker_token = String::new();
    state.config = std::sync::Arc::new(config);

    let resp = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            "",
            serde_json::json!({ "workerId": "w1" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── enqueue seeds the pipeline ────────────────────────────────────────────────

#[tokio::test]
async fn enqueue_seeds_one_row_per_pipeline_stage() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &s, "payments-api").await;

    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT seq, key, status FROM analysis_stages WHERE analysis_id = ? ORDER BY seq",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap();

    assert_eq!(rows.len(), featuredoc::pipeline::STAGES.len());
    for (row, stage) in rows.iter().zip(featuredoc::pipeline::STAGES.iter()) {
        assert_eq!(row.0, stage.seq);
        assert_eq!(row.1, stage.key);
        assert_eq!(row.2, "pending");
    }
}

// ── claim ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn claim_returns_no_content_when_the_queue_is_empty() {
    let (state, _p) = stub_state().await;
    assert_eq!(claim(&state, "w1").await.status(), StatusCode::NO_CONTENT);
}

/// The property horizontal scaling rests on: two workers racing for one job
/// produce exactly one winner, and the loser is told the queue is empty.
#[tokio::test]
async fn two_workers_racing_for_one_job_produce_exactly_one_winner() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &s, "payments-api").await;

    let (a, b) = tokio::join!(claim(&state, "w1"), claim(&state, "w2"));
    let mut statuses = [a.status(), b.status()];
    statuses.sort_by_key(|s| s.as_u16());
    assert_eq!(
        statuses,
        [StatusCode::OK, StatusCode::NO_CONTENT],
        "exactly one worker must win the claim"
    );

    let claimed_by: Vec<String> = sqlx::query_scalar("SELECT claimed_by FROM analyses WHERE id = ?")
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .unwrap();
    assert_eq!(claimed_by.len(), 1);
    assert!(claimed_by[0] == "w1" || claimed_by[0] == "w2");
}

/// Two queued jobs and two workers: each takes a different one, none is taken twice.
#[tokio::test]
async fn concurrent_workers_take_disjoint_jobs() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let first = enqueue(&state, &s, "payments-api").await;
    let second = enqueue(&state, &s, "checkout-web").await;

    let (a, b) = tokio::join!(claim(&state, "w1"), claim(&state, "w2"));
    assert_eq!(a.status(), StatusCode::OK);
    assert_eq!(b.status(), StatusCode::OK);

    let mut ids = vec![
        json_body(a).await["id"].as_str().unwrap().to_string(),
        json_body(b).await["id"].as_str().unwrap().to_string(),
    ];
    ids.sort();
    let mut expected = vec![first, second];
    expected.sort();
    assert_eq!(ids, expected, "each queued job is claimed exactly once");
}

/// The same property under real thread contention rather than cooperative
/// interleaving: six workers on four runtime threads racing over six jobs must
/// still produce six distinct claims and zero duplicates.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn many_workers_racing_never_claim_the_same_job_twice() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    for _ in 0..6 {
        enqueue(&state, &s, "payments-api").await;
    }

    let mut handles = Vec::new();
    for w in 0..6 {
        let st = state.clone();
        handles.push(tokio::spawn(async move {
            let resp = claim(&st, &format!("w{w}")).await;
            assert_eq!(resp.status(), StatusCode::OK);
            json_body(resp).await["id"].as_str().unwrap().to_string()
        }));
    }

    let mut ids = Vec::new();
    for h in handles {
        ids.push(h.await.unwrap());
    }
    let distinct: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(distinct.len(), 6, "claimed ids must all differ: {ids:?}");

    let still_queued: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM analyses WHERE status = 'queued'")
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(still_queued, 0);
}

#[tokio::test]
async fn claim_hands_over_the_target_and_the_executable_stage() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    enqueue(&state, &s, "payments-api").await;

    let body = json_body(claim(&state, "w1").await).await;
    assert_eq!(body["repoOwner"], "stub-account");
    assert_eq!(body["repoName"], "payments-api");
    assert_eq!(body["branch"], "main");
    // Widened again by slice 4b-1: stage 3 (`discovery_strategy`, AC1.3) is now
    // implemented, so the queue offers it alongside `fetch` and `cross_cutting`.
    // Stage 4 stays unoffered — it is both unimplemented *and* gated on the user
    // approving the strategy (AC1.3), which `tests/strategy.rs` asserts separately.
    assert_eq!(
        body["executableStages"],
        serde_json::json!(["fetch", "cross_cutting", "discovery_strategy"])
    );
    // Stub mode still mints a (synthetic) short-lived installation token — the
    // worker needs one shaped like the real thing.
    assert!(body["installationToken"].is_string());
}

/// A worker that dies mid-job must not strand it: once the lease lapses, the next
/// claim takes it back (test/04 scenario 7's "워커 복구 후 처리").
#[tokio::test]
async fn an_expired_lease_returns_the_job_to_the_queue() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &s, "payments-api").await;

    assert_eq!(claim(&state, "dead-worker").await.status(), StatusCode::OK);
    // Nothing else is claimable while the lease holds.
    assert_eq!(claim(&state, "w2").await.status(), StatusCode::NO_CONTENT);

    sqlx::query("UPDATE analyses SET lease_expires_at = ? WHERE id = ?")
        .bind(1_i64)
        .bind(&id)
        .execute(&state.db)
        .await
        .unwrap();

    let retaken = claim(&state, "w2").await;
    assert_eq!(retaken.status(), StatusCode::OK);
    assert_eq!(json_body(retaken).await["id"], id);
}

// ── progress reporting ────────────────────────────────────────────────────────

#[tokio::test]
async fn stage_reports_persist_and_finish_closes_the_job() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &s, "payments-api").await;
    assert_eq!(claim(&state, "w1").await.status(), StatusCode::OK);

    let running = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/fetch"),
            WORKER_TOKEN,
            serde_json::json!({ "workerId": "w1", "status": "running" }),
        ))
        .await
        .unwrap();
    assert_eq!(running.status(), StatusCode::NO_CONTENT);

    let done = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/fetch"),
            WORKER_TOKEN,
            serde_json::json!({
                "workerId": "w1",
                "status": "succeeded",
                "detail": "766 files · 2.2 MB"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(done.status(), StatusCode::NO_CONTENT);

    let finish = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/finish"),
            WORKER_TOKEN,
            serde_json::json!({ "workerId": "w1", "status": "awaiting_pipeline" }),
        ))
        .await
        .unwrap();
    assert_eq!(finish.status(), StatusCode::NO_CONTENT);

    let (status, detail): (String, Option<String>) =
        sqlx::query_as("SELECT status, detail FROM analysis_stages WHERE analysis_id = ? AND key = 'fetch'")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(status, "succeeded");
    assert_eq!(detail.as_deref(), Some("766 files · 2.2 MB"));

    // Only the executed stage moved; the unimplemented ones stay pending rather
    // than being marked done (AC1.2~AC1.4 are not in this slice).
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM analysis_stages WHERE analysis_id = ? AND status = 'pending'",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(pending, (featuredoc::pipeline::STAGES.len() - 1) as i64);

    let (analysis_status, lease): (String, Option<i64>) =
        sqlx::query_as("SELECT status, lease_expires_at FROM analyses WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(analysis_status, "awaiting_pipeline");
    assert_eq!(lease, None, "finishing releases the lease");
}

/// A worker whose job was reclaimed must not be able to overwrite its successor.
#[tokio::test]
async fn a_worker_without_the_lease_cannot_report() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &s, "payments-api").await;
    assert_eq!(claim(&state, "w1").await.status(), StatusCode::OK);

    let intruder = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/fetch"),
            WORKER_TOKEN,
            serde_json::json!({ "workerId": "w2", "status": "succeeded" }),
        ))
        .await
        .unwrap();
    assert_eq!(intruder.status(), StatusCode::CONFLICT);

    let heartbeat = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/heartbeat"),
            WORKER_TOKEN,
            serde_json::json!({ "workerId": "w2" }),
        ))
        .await
        .unwrap();
    assert_eq!(heartbeat.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn unknown_stage_or_status_is_rejected() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &s, "payments-api").await;
    assert_eq!(claim(&state, "w1").await.status(), StatusCode::OK);

    let bad_stage = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/not-a-stage"),
            WORKER_TOKEN,
            serde_json::json!({ "workerId": "w1", "status": "succeeded" }),
        ))
        .await
        .unwrap();
    assert_eq!(bad_stage.status(), StatusCode::BAD_REQUEST);

    let bad_status = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/fetch"),
            WORKER_TOKEN,
            serde_json::json!({ "workerId": "w1", "status": "teleported" }),
        ))
        .await
        .unwrap();
    assert_eq!(bad_status.status(), StatusCode::BAD_REQUEST);

    let bad_terminal = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/finish"),
            WORKER_TOKEN,
            serde_json::json!({ "workerId": "w1", "status": "succeeded" }),
        ))
        .await
        .unwrap();
    assert_eq!(
        bad_terminal.status(),
        StatusCode::BAD_REQUEST,
        "`succeeded` would claim the unimplemented stages ran"
    );
}

/// AC4.5's headline: the API keeps answering with no worker in sight, and the
/// job simply waits. (The cluster-level version of this is e2e scenario 7.)
#[tokio::test]
async fn the_api_serves_and_the_queue_holds_while_no_worker_claims() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    enqueue(&state, &s, "payments-api").await;

    let listed = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/analyses")
                .header(header::COOKIE, format!("fd_session={s}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let body = json_body(listed).await;
    assert_eq!(body[0]["status"], "queued");
}

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
    login_installed_with(state, github_id, login, 4242).await
}

async fn login_installed_with(
    state: &AppState,
    github_id: i64,
    login: &str,
    installation_id: i64,
) -> String {
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
            installation_id,
            account_login: Some("stub-account"),
            account_type: Some("User"),
            repository_selection: Some("selected"),
        },
    )
    .await
    .unwrap();
    session::create(&state.db, &user.id).await.unwrap()
}

async fn register_key(state: &AppState, session: &str, provider: &str, key: &str) {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            "/api/llm-keys",
            session,
            serde_json::json!({ "provider": provider, "key": key }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED, "register {provider} key");
}

async fn set_language(state: &AppState, session: &str, language: &str) {
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/settings")
                .header(header::COOKIE, format!("fd_session={session}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({ "llmLanguage": language }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "set language {language}");
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

#[tokio::test]
async fn claim_returns_no_content_when_the_queue_is_empty() {
    let (state, _p) = stub_state().await;
    assert_eq!(claim(&state, "w1").await.status(), StatusCode::NO_CONTENT);
}

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

/// Plain `#[tokio::test]` interleaves cooperatively on one thread, so the race
/// above does not by itself prove the claim is safe under real thread contention.
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
    // Stage 4's absence has two independent causes — unimplemented, *and* gated on
    // the user approving the strategy — so implementing it alone will not offer it.
    assert_eq!(
        body["executableStages"],
        serde_json::json!(["fetch", "cross_cutting", "discovery_strategy"])
    );
    // Stub mode still mints a (synthetic) short-lived installation token — the
    // worker needs one shaped like the real thing.
    assert!(body["installationToken"].is_string());
}

/// Takes the App's access away without touching a process-wide stub switch that
/// sibling tests in this binary would see.
async fn uninstall(state: &AppState) {
    sqlx::query("DELETE FROM installations")
        .execute(&state.db)
        .await
        .unwrap();
}

async fn analysis_row(state: &AppState, id: &str) -> (String, Option<String>, Option<i64>) {
    sqlx::query_as("SELECT status, error, lease_expires_at FROM analyses WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .unwrap()
}

/// AC4.1 「해제」 — a queued job whose access is gone is closed by the policy instead
/// of being handed out, and the queue does not wedge behind it.
#[tokio::test]
async fn a_job_whose_access_was_revoked_is_stopped_rather_than_claimed() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &s, "payments-api").await;

    uninstall(&state).await;

    assert_eq!(
        claim(&state, "w1").await.status(),
        StatusCode::NO_CONTENT,
        "a revoked job is never handed to a worker"
    );

    let (status, error, lease) = analysis_row(&state, &id).await;
    assert_eq!(status, "failed");
    assert_eq!(error.as_deref(), Some(featuredoc::analysis::ACCESS_REVOKED));
    assert_eq!(lease, None, "stopping releases the lease");

    assert_eq!(claim(&state, "w1").await.status(), StatusCode::NO_CONTENT);
}

/// AC4.1 「해제」 — access taken away *while the job runs*. The renewal is refused at
/// the lease boundary, which is what makes "현재 호출까지만 마무리" true: the call the
/// worker is in the middle of is never pre-empted, and the stage it was running is
/// closed with the same reason the user reads.
#[tokio::test]
async fn revoking_access_mid_run_stops_the_job_at_the_next_lease_boundary() {
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

    uninstall(&state).await;

    let heartbeat = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/heartbeat"),
            WORKER_TOKEN,
            serde_json::json!({ "workerId": "w1" }),
        ))
        .await
        .unwrap();
    assert_eq!(
        heartbeat.status(),
        StatusCode::CONFLICT,
        "the worker stops through the path it already had"
    );

    let (status, error, lease) = analysis_row(&state, &id).await;
    assert_eq!(status, "failed");
    assert_eq!(error.as_deref(), Some(featuredoc::analysis::ACCESS_REVOKED));
    assert_eq!(lease, None);

    let (stage_status, stage_error): (String, Option<String>) = sqlx::query_as(
        "SELECT status, error FROM analysis_stages WHERE analysis_id = ? AND key = 'fetch'",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(stage_status, "failed");
    assert_eq!(
        stage_error.as_deref(),
        Some(featuredoc::analysis::ACCESS_REVOKED),
        "one story on the screen, not two"
    );

    let detail = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/api/analyses/{id}"))
                .header(header::COOKIE, format!("fd_session={s}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let body = json_body(detail).await;
    assert_eq!(body["accessRevoked"], true);
    assert_eq!(body["error"], featuredoc::analysis::ACCESS_REVOKED);
}

#[tokio::test]
async fn an_expired_lease_returns_the_job_to_the_queue() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &s, "payments-api").await;

    assert_eq!(claim(&state, "dead-worker").await.status(), StatusCode::OK);
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
#[tokio::test]
async fn one_worker_claiming_two_users_jobs_never_mixes_their_context() {
    const ALICE_KEY: &str = "sk-ant-api03-aaaaaaaaaaaaaaaaaaaa";
    const BOB_KEY: &str = "sk-proj-bbbbbbbbbbbbbbbbbbbb";

    let (state, _p) = stub_state().await;
    let alice = login_installed_with(&state, 1, "alice", 11_001).await;
    let bob = login_installed_with(&state, 2, "bob", 11_002).await;

    register_key(&state, &alice, "anthropic", ALICE_KEY).await;
    register_key(&state, &bob, "openai", BOB_KEY).await;
    set_language(&state, &bob, "en").await;

    let alice_job = enqueue(&state, &alice, "payments-api").await;
    let bob_job = enqueue(&state, &bob, "checkout-web").await;

    let first = json_body(claim(&state, "w1").await).await;
    let second = json_body(claim(&state, "w1").await).await;
    let mut by_id = std::collections::HashMap::new();
    for c in [&first, &second] {
        by_id.insert(c["id"].as_str().unwrap().to_string(), c);
    }
    assert_eq!(by_id.len(), 2, "한 워커가 서로 다른 두 job 을 집었다");
    let a = by_id.get(&alice_job).expect("alice 의 job 이 건네졌다");
    let b = by_id.get(&bob_job).expect("bob 의 job 이 건네졌다");

    assert_eq!(a["repoName"], "payments-api");
    assert_eq!(b["repoName"], "checkout-web");

    assert_eq!(a["llmApiKey"], ALICE_KEY, "alice 의 job 에 alice 의 키");
    assert_eq!(a["llmProvider"], "anthropic");
    assert_eq!(b["llmApiKey"], BOB_KEY, "bob 의 job 에 bob 의 키");
    assert_eq!(b["llmProvider"], "openai");

    // 값 비교만으로는 "둘 다 실렸다"를 못 잡는다 — 상대 것의 부재를 따로 센다.
    assert!(
        !a.to_string().contains(BOB_KEY),
        "alice 의 claim 이 bob 의 키를 흘렸다: {a}"
    );
    assert!(
        !b.to_string().contains(ALICE_KEY),
        "bob 의 claim 이 alice 의 키를 흘렸다: {b}"
    );

    let a_token = a["installationToken"].as_str().unwrap();
    let b_token = b["installationToken"].as_str().unwrap();
    assert!(
        a_token.starts_with("ghs_stub_11001_"),
        "alice 의 설치로 발급되지 않았다: {a_token}"
    );
    assert!(
        b_token.starts_with("ghs_stub_11002_"),
        "bob 의 설치로 발급되지 않았다: {b_token}"
    );

    assert_eq!(b["llmLanguage"], "en", "bob 이 고른 언어");
    assert_ne!(a["llmLanguage"], "en", "alice 는 고른 적이 없다");
}

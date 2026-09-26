//! Measured LLM usage: what `/api/usage` and the analysis detail add up, and
//! whose rows they are allowed to add (AC4.6).
//!
//! The figures are written through the worker's own `/internal` routes rather
//! than by inserting rows, for the reason `documents.rs` gives: a hand-built row
//! can drift from what a worker actually submits, and the contract under test is
//! exactly that submission.
mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

const WORKER: &str = "w-usage";

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
            json!({ "repoUrl": format!("stub-account/{repo}") }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    json_body(resp).await["id"].as_str().unwrap().to_string()
}

async fn claim(state: &AppState) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            json!({ "workerId": WORKER }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "queue should hold a job");
}

async fn submit_doc(state: &AppState, id: &str, kind: &str, calls: i64, input: i64, output: i64) {
    let mut body = json!({
        "workerId": WORKER,
        "content": { "categories": [] },
        "model": "stub-model",
        "inputTokens": input,
        "outputTokens": output,
    });
    if calls >= 0 {
        body["calls"] = json!(calls);
    }
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/documents/{kind}"),
            body,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "submit {kind}");
}

async fn usage(state: &AppState, session: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get("/api/usage", session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

async fn detail(state: &AppState, session: &str, id: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

#[tokio::test]
async fn per_analysis_and_total_spend_add_up() {
    let (state, _path) = stub_state().await;
    let token = login_installed(&state, 9001, "usage-user").await;

    let first = enqueue(&state, &token, "payments-api").await;
    claim(&state).await;
    submit_doc(&state, &first, "cross_cutting", 1, 1_000_000, 0).await;

    let second = enqueue(&state, &token, "checkout-web").await;
    claim(&state).await;
    submit_doc(&state, &second, "cross_cutting", 1, 0, 1_000_000).await;

    let seen = usage(&state, &token).await;
    let rows = seen["analyses"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "작업별 행은 사용자의 분석마다 하나다");

    let total = &seen["total"];
    assert_eq!(total["llmCalls"], 2);
    assert_eq!(total["inputTokens"], 1_000_000);
    assert_eq!(total["outputTokens"], 1_000_000);

    let summed_calls: i64 = rows.iter().map(|r| r["llmCalls"].as_i64().unwrap()).sum();
    let summed_in: i64 = rows.iter().map(|r| r["inputTokens"].as_i64().unwrap()).sum();
    let summed_out: i64 = rows.iter().map(|r| r["outputTokens"].as_i64().unwrap()).sum();
    assert_eq!(summed_calls, total["llmCalls"].as_i64().unwrap());
    assert_eq!(summed_in, total["inputTokens"].as_i64().unwrap());
    assert_eq!(summed_out, total["outputTokens"].as_i64().unwrap());

    // 출력 토큰이 더 비싸다 — 두 분석은 토큰 수가 같고 방향만 다르다.
    let by_id = |id: &str| -> i64 {
        rows.iter()
            .find(|r| r["analysisId"] == id)
            .unwrap()["costCents"]
            .as_i64()
            .unwrap()
    };
    assert!(by_id(&second) > by_id(&first));
}

#[tokio::test]
async fn a_merged_stage_row_counts_both_of_its_calls() {
    let (state, _path) = stub_state().await;
    let token = login_installed(&state, 9002, "merged-user").await;

    let id = enqueue(&state, &token, "payments-api").await;
    claim(&state).await;
    submit_doc(&state, &id, "acceptance_dependencies", 2, 10, 20).await;

    assert_eq!(detail(&state, &token, &id).await["spend"]["llmCalls"], 2);

    // 음성 프로브: 같은 행이 1회라고 말하면 합계도 1이어야 한다 — 값이 실제로
    // 읽히고 있다는 뜻이고, 2 가 상수로 박힌 것이 아니라는 뜻이다.
    submit_doc(&state, &id, "acceptance_dependencies", 1, 10, 20).await;
    assert_eq!(detail(&state, &token, &id).await["spend"]["llmCalls"], 1);
}

#[tokio::test]
async fn a_report_without_a_call_count_still_counts_one_call() {
    let (state, _path) = stub_state().await;
    let token = login_installed(&state, 9003, "legacy-worker-user").await;

    let id = enqueue(&state, &token, "payments-api").await;
    claim(&state).await;
    submit_doc(&state, &id, "cross_cutting", -1, 7, 9).await;

    let spend = &detail(&state, &token, &id).await["spend"];
    assert_eq!(spend["llmCalls"], 1);
    assert_eq!(spend["inputTokens"], 7);
}

#[tokio::test]
async fn one_users_spend_never_appears_in_anothers() {
    let (state, _path) = stub_state().await;
    let mine = login_installed(&state, 9004, "mine").await;
    let theirs = login_installed(&state, 9005, "theirs").await;

    let id = enqueue(&state, &mine, "payments-api").await;
    claim(&state).await;
    submit_doc(&state, &id, "cross_cutting", 1, 500, 500).await;

    let other = usage(&state, &theirs).await;
    assert_eq!(other["analyses"].as_array().unwrap().len(), 0);
    assert_eq!(other["total"]["llmCalls"], 0);
    assert_eq!(other["total"]["costCents"], 0);

    assert_eq!(usage(&state, &mine).await["total"]["llmCalls"], 1);
}

#[tokio::test]
async fn an_analysis_that_called_no_one_spends_nothing() {
    let (state, _path) = stub_state().await;
    let token = login_installed(&state, 9006, "quiet-user").await;

    let id = enqueue(&state, &token, "payments-api").await;

    let spend = &detail(&state, &token, &id).await["spend"];
    assert_eq!(spend["llmCalls"], 0);
    assert_eq!(spend["costCents"], 0);
}

#[tokio::test]
async fn usage_is_closed_to_anonymous_requests() {
    let (state, _path) = stub_state().await;
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/usage")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

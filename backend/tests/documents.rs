//! Pipeline document storage and the reproducibility verdict (AC1.2).
//!
//! What Cross-cutting Concerns needs from the API: read the cross-cutting document one analysis
//! produced, and see whether re-analyzing the same target reproduced it. Both run
//! against the router in-process, so the contract is gated by `cargo test`; the
//! screen that renders it is asserted by
//! `e2e/tests/ac1-2-cross-cutting-concerns.spec.ts`.
//!
//! Documents are written through the worker's own `/internal` route rather than by
//! inserting rows directly — a hand-written fixture could drift from what a worker
//! actually submits, which is the whole point of gating the contract here.

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

const WORKER: &str = "w-documents";

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

async fn claim(state: &AppState) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            json!({ "workerId": WORKER }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "queue should hold a job");
    json_body(resp).await
}

/// Submits a document the way the worker does, and asserts it was accepted.
async fn submit(state: &AppState, id: &str, content: serde_json::Value) -> StatusCode {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/documents/cross_cutting"),
            json!({
                "workerId": WORKER,
                "content": content,
                "model": "stub-model",
                "inputTokens": 120,
                "outputTokens": 256,
            }),
        ))
        .await
        .unwrap();
    resp.status()
}

async fn read_document(state: &AppState, session: &str, id: &str) -> axum::response::Response {
    build_router(state.clone())
        .oneshot(get(
            &format!("/api/analyses/{id}/documents/cross-cutting"),
            session,
        ))
        .await
        .unwrap()
}

fn doc(n: u8) -> serde_json::Value {
    json!({ "categories": [
        { "axis": "infrastructure", "items": [
            { "name": format!("Kubernetes v{n}"), "evidence": ["deploy/base/deployment.yaml"] },
        ]},
    ]})
}

/// The queue must hand the worker the key it needs for stage 2, and must name the
/// stage as executable — otherwise the worker stops after `fetch`.
#[tokio::test]
async fn claim_offers_the_cross_cutting_stage() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 8101, "claimer").await;
    enqueue(&state, &s, "payments-api").await;

    let job = claim(&state).await;
    let stages: Vec<&str> = job["executableStages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(stages.contains(&"fetch"));
    assert!(
        stages.contains(&"cross_cutting"),
        "stage 2 must be offered: {stages:?}"
    );
    // No key registered for this user, so the queue reports none rather than
    // inventing one — the stage then fails with a clear reason.
    assert!(job["llmApiKey"].is_null());
}

/// Lease-guarded like every other worker write: a caller without the lease cannot
/// store a document.
#[tokio::test]
async fn a_worker_without_the_lease_cannot_store_a_document() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 8102, "unleased").await;
    let id = enqueue(&state, &s, "payments-api").await;

    // Never claimed — nobody holds a lease on this job.
    assert_eq!(submit(&state, &id, doc(1)).await, StatusCode::CONFLICT);
}

#[tokio::test]
async fn an_unknown_stage_is_rejected() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 8103, "unknown-stage").await;
    let id = enqueue(&state, &s, "payments-api").await;
    claim(&state).await;

    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/documents/not-a-stage"),
            json!({ "workerId": WORKER, "content": doc(1), "model": "m" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

/// The first analysis of a target has nothing to compare against, and must say so
/// rather than claiming the result reproduced.
#[tokio::test]
async fn the_first_analysis_reports_no_comparison() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 8104, "firstrun").await;
    let id = enqueue(&state, &s, "payments-api").await;
    claim(&state).await;
    assert_eq!(submit(&state, &id, doc(1)).await, StatusCode::NO_CONTENT);

    let body = json_body(read_document(&state, &s, &id).await).await;
    assert_eq!(body["kind"], "cross_cutting");
    assert_eq!(body["reproducibility"]["verdict"], "first");
    assert!(body["reproducibility"]["comparedTo"].is_null());
    assert_eq!(body["content"]["categories"][0]["axis"], "infrastructure");
}

/// AC1.2's determinism clause, both halves: re-analyzing the same target reports
/// `unchanged` when the result reproduced and `changed` when it did not.
#[tokio::test]
async fn a_reanalysis_reports_whether_the_result_reproduced() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 8105, "rerun").await;

    let first = enqueue(&state, &s, "payments-api").await;
    claim(&state).await;
    submit(&state, &first, doc(1)).await;

    // Same target again, identical result.
    let second = enqueue(&state, &s, "payments-api").await;
    claim(&state).await;
    submit(&state, &second, doc(1)).await;
    let body = json_body(read_document(&state, &s, &second).await).await;
    assert_eq!(body["reproducibility"]["verdict"], "unchanged");
    assert_eq!(body["reproducibility"]["comparedTo"], first);

    // Same target once more, different result.
    let third = enqueue(&state, &s, "payments-api").await;
    claim(&state).await;
    submit(&state, &third, doc(2)).await;
    let body = json_body(read_document(&state, &s, &third).await).await;
    assert_eq!(body["reproducibility"]["verdict"], "changed");
    assert_eq!(body["reproducibility"]["comparedTo"], second);
}

/// A different repository is a different target: its document must not be compared
/// against this one, or every first analysis of a new repo would read as "changed".
#[tokio::test]
async fn a_different_repository_is_not_treated_as_a_rerun() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 8106, "othertarget").await;

    let first = enqueue(&state, &s, "payments-api").await;
    claim(&state).await;
    submit(&state, &first, doc(1)).await;

    let other = enqueue(&state, &s, "checkout-web").await;
    claim(&state).await;
    submit(&state, &other, doc(2)).await;

    let body = json_body(read_document(&state, &s, &other).await).await;
    assert_eq!(body["reproducibility"]["verdict"], "first");
}

/// A re-run of the stage (AC1.5's partial retry) replaces this analysis's document
/// rather than adding a second one.
#[tokio::test]
async fn resubmitting_replaces_the_analysis_own_document() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 8107, "resubmit").await;
    let id = enqueue(&state, &s, "payments-api").await;
    claim(&state).await;

    submit(&state, &id, doc(1)).await;
    submit(&state, &id, doc(2)).await;

    let body = json_body(read_document(&state, &s, &id).await).await;
    assert_eq!(body["content"]["categories"][0]["items"][0]["name"], "Kubernetes v2");

    let rows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM analysis_documents WHERE analysis_id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(rows.0, 1, "one row per (analysis, kind)");
}

/// Owner-scoped like every other analysis read (AC4.7).
#[tokio::test]
async fn another_users_document_is_not_readable() {
    let (state, _path) = stub_state().await;
    let owner = login_installed(&state, 8108, "owner").await;
    let stranger = login_installed(&state, 8109, "stranger").await;

    let id = enqueue(&state, &owner, "payments-api").await;
    claim(&state).await;
    submit(&state, &id, doc(1)).await;

    assert_eq!(
        read_document(&state, &stranger, &id).await.status(),
        StatusCode::NOT_FOUND
    );
}

/// A stage that has not produced its document yet is a 404, not an empty document —
/// Cross-cutting Concerns must be able to tell "not run" from "ran and found nothing".
#[tokio::test]
async fn a_document_that_was_never_produced_is_not_found() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 8110, "notyet").await;
    let id = enqueue(&state, &s, "payments-api").await;

    assert_eq!(
        read_document(&state, &s, &id).await.status(),
        StatusCode::NOT_FOUND
    );
}

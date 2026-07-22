//! Analysis enqueue surface (AC1.1): list accessible repos, pre-flight estimate,
//! trigger an analysis (queued), and reject out-of-scope targets without queuing.
//! Runs in stub mode, so the GitHub boundary is answered by in-process doubles.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::stub_state;
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

/// A logged-in user whose (stub) App installation grants the stub repository set
/// (stub-account/{payments-api,checkout-web,notif-worker}).
async fn login_installed(state: &AppState) -> String {
    let gh = GithubUser {
        id: 1,
        login: "alice".into(),
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

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn get(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .body(Body::empty())
        .unwrap()
}

fn post_json(uri: &str, token: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn lists_accessible_repositories() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state).await;

    let resp = build_router(state)
        .oneshot(get("/api/repositories", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    let repos = body.as_array().unwrap();
    assert_eq!(repos.len(), 3);
    assert!(repos
        .iter()
        .any(|r| r["fullName"] == "stub-account/payments-api"));
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn preflight_reports_access_and_positive_estimate() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state).await;

    let resp = build_router(state)
        .oneshot(post_json(
            "/api/analyses/preflight",
            &token,
            serde_json::json!({ "repoUrl": "https://github.com/stub-account/payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["hasAccess"], true);
    assert_eq!(body["branch"], "main");
    assert!(body["estLlmCalls"].as_i64().unwrap() > 0);
    assert!(body["filesToScan"].as_i64().unwrap() > 0);
    assert!(body["estCostCents"].as_i64().unwrap() > 0);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn preflight_out_of_scope_reports_no_access() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state).await;

    let resp = build_router(state)
        .oneshot(post_json(
            "/api/analyses/preflight",
            &token,
            serde_json::json!({ "repoUrl": "stub-account/not-granted" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["hasAccess"], false);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn trigger_enqueues_a_queued_analysis() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state).await;

    let resp = build_router(state.clone())
        .oneshot(post_json(
            "/api/analyses",
            &token,
            serde_json::json!({ "repoUrl": "github.com/stub-account/payments-api", "branch": "main" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = json_body(resp).await;
    assert_eq!(body["status"], "queued");
    assert_eq!(body["repoName"], "payments-api");
    assert_eq!(body["branch"], "main");

    // The queued job is visible on the home list.
    let resp = build_router(state)
        .oneshot(get("/api/analyses", &token))
        .await
        .unwrap();
    let list = json_body(resp).await;
    let jobs = list.as_array().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["repoName"], "payments-api");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn out_of_scope_target_is_rejected_and_nothing_queued() {
    let (state, path) = stub_state().await;
    let token = login_installed(&state).await;

    let resp = build_router(state.clone())
        .oneshot(post_json(
            "/api/analyses",
            &token,
            serde_json::json!({ "repoUrl": "https://github.com/stub-account/secret-repo" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Nothing was queued.
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM analyses")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count.0, 0, "an out-of-scope target must not enqueue a job");
    let _ = std::fs::remove_file(&path);
}

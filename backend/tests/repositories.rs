//! S02 repositories surface: empty when no installation, connect within the
//! installation's access scope, reject out-of-scope repos (AC1.1), and per-user
//! isolation (AC4.7).

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::stub_state;
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

async fn login_user(state: &AppState, login: &str, id: i64) -> (String, String) {
    let user = users::upsert(
        &state.db,
        &GithubUser {
            id,
            login: login.into(),
            name: Some(login.into()),
            avatar_url: None,
        },
    )
    .await
    .unwrap();
    let token = session::create(&state.db, &user.id).await.unwrap();
    (user.id, token)
}

async fn link_installation(state: &AppState, user_id: &str, installation_id: i64) {
    installations::upsert(
        &state.db,
        user_id,
        &installations::NewInstallation {
            installation_id,
            account_login: Some("stub-account"),
            account_type: Some("User"),
            repository_selection: Some("selected"),
        },
    )
    .await
    .unwrap();
}

fn get(cookie: &str) -> Request<Body> {
    Request::builder()
        .uri("/api/repositories")
        .header(header::COOKIE, format!("fd_session={cookie}"))
        .body(Body::empty())
        .unwrap()
}

fn connect(cookie: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/repositories")
        .header(header::COOKIE, format!("fd_session={cookie}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn list_is_empty_without_a_connected_repository() {
    let (state, path) = stub_state().await;
    let (_id, token) = login_user(&state, "alice", 1).await;

    let resp = build_router(state).oneshot(get(&token)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn connect_links_an_in_scope_repository_with_reserved_analysis_fields() {
    let (state, path) = stub_state().await;
    let (uid, token) = login_user(&state, "alice", 1).await;
    link_installation(&state, &uid, 777).await;

    // Connect a repo the stub installation can access.
    let resp = build_router(state.clone())
        .oneshot(connect(
            &token,
            serde_json::json!({ "owner": "stub-account", "name": "payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let repo = json_body(resp).await;
    assert_eq!(repo["owner"], "stub-account");
    assert_eq!(repo["name"], "payments-api");
    assert_eq!(repo["branch"], "main");
    // Analysis-derived fields are reserved (null) until the pipeline lands.
    assert_eq!(repo["status"], "not_analyzed");
    assert!(repo["featureCount"].is_null());
    assert!(repo["conflictCount"].is_null());
    assert!(repo["spendCents"].is_null());
    assert!(repo["progress"].is_null());
    assert!(repo["step"].is_null());
    assert!(repo["lastAnalyzedAt"].is_null());

    // It now shows up in the list.
    let resp = build_router(state).oneshot(get(&token)).await.unwrap();
    let body = json_body(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["name"], "payments-api");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn connect_rejects_a_repository_outside_the_installation_scope() {
    let (state, path) = stub_state().await;
    let (uid, token) = login_user(&state, "alice", 1).await;
    link_installation(&state, &uid, 777).await;

    let resp = build_router(state.clone())
        .oneshot(connect(
            &token,
            serde_json::json!({ "owner": "stub-account", "name": "not-granted" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Nothing was linked.
    let resp = build_router(state).oneshot(get(&token)).await.unwrap();
    let body = json_body(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn connect_without_an_installation_is_forbidden() {
    let (state, path) = stub_state().await;
    let (_uid, token) = login_user(&state, "alice", 1).await;

    let resp = build_router(state)
        .oneshot(connect(
            &token,
            serde_json::json!({ "owner": "stub-account", "name": "payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn one_users_repositories_are_invisible_to_another() {
    let (state, path) = stub_state().await;
    let (alice_id, alice) = login_user(&state, "alice", 1).await;
    let (_bob_id, bob) = login_user(&state, "bob", 2).await;
    link_installation(&state, &alice_id, 777).await;

    let resp = build_router(state.clone())
        .oneshot(connect(
            &alice,
            serde_json::json!({ "owner": "stub-account", "name": "payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Bob sees none of Alice's repositories.
    let resp = build_router(state.clone()).oneshot(get(&bob)).await.unwrap();
    assert_eq!(json_body(resp).await.as_array().unwrap().len(), 0);

    // Alice still sees her own.
    let resp = build_router(state).oneshot(get(&alice)).await.unwrap();
    assert_eq!(json_body(resp).await.as_array().unwrap().len(), 1);
    let _ = std::fs::remove_file(&path);
}

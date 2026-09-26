//! AC4.10 — asking the owner to share a repository you cannot see, without the
//! asking itself telling you whether there is anything there to share.
//!
//! The load-bearing assertion is `indistinguishable_…`: it puts a real stranger's
//! analysis id and an id that has never existed through the same call and compares
//! the two answers as whole values. Comparing them field by field would let a new
//! field slip in unchecked, which is exactly how this property erodes.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::stub_state;
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

/// An id no analysis will ever have — the control the scenario asks for
/// ("대비군으로 실재하지 않는 임의 id").
const ABSENT_ID: &str = "00000000-0000-4000-8000-000000000000";

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

fn post(uri: &str, token: Option<&str>) -> Request<Body> {
    let b = Request::builder().method("POST").uri(uri);
    let b = match token {
        Some(t) => b.header(header::COOKIE, format!("fd_session={t}")),
        None => b,
    };
    b.header(header::CONTENT_TYPE, "application/json")
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

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body().collect().await.unwrap().to_bytes().to_vec()
}

/// Sends the request and returns exactly what a client could observe.
async fn ask(state: &AppState, token: &str, id: &str) -> (StatusCode, Vec<u8>) {
    let resp = build_router(state.clone())
        .oneshot(post(&format!("/api/analyses/{id}/access-request"), Some(token)))
        .await
        .unwrap();
    let status = resp.status();
    (status, body_bytes(resp).await)
}

async fn create_analysis(state: &AppState, token: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(post_json(
            "/api/analyses",
            token,
            serde_json::json!({ "repoUrl": "stub-account/payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = serde_json::from_slice(&body_bytes(resp).await).unwrap();
    body["id"].as_str().unwrap().to_string()
}

async fn rows(state: &AppState, analysis_id: &str) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM access_requests WHERE analysis_id = ?")
        .bind(analysis_id)
        .fetch_one(&state.db)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_request_for_a_strangers_analysis_is_indistinguishable_from_one_for_nothing() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "sc0415a").await;
    let stranger = login_installed(&state, 2, "sc0415b").await;
    let hidden = create_analysis(&state, &owner).await;

    // The stranger cannot even see that the analysis is there...
    let read = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/api/analyses/{hidden}"))
                .header(header::COOKIE, format!("fd_session={stranger}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(read.status(), StatusCode::NOT_FOUND, "AC4.7 이 세운 비확인");

    // ...and asking for it answers exactly as asking for nothing does.
    let real = ask(&state, &stranger, &hidden).await;
    let absent = ask(&state, &stranger, ABSENT_ID).await;
    assert_eq!(real.0, StatusCode::ACCEPTED);
    assert_eq!(
        real, absent,
        "실재하는 남의 대상과 실재하지 않는 대상의 응답이 구분된다"
    );

    // Neither answer names the owner, the repository, or the target.
    let seen = String::from_utf8(real.1.clone()).unwrap();
    for leak in ["sc0415a", "payments-api", "stub-account", hidden.as_str()] {
        assert!(!seen.contains(leak), "응답이 {leak} 를 흘린다: {seen}");
    }

    // Both were recorded — the screen's "요청을 보냈어요" is true on both paths.
    assert_eq!(rows(&state, &hidden).await, 1);
    assert_eq!(rows(&state, ABSENT_ID).await, 1);

    state.db.close().await;
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn asking_twice_leaves_one_request_and_the_same_answer() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "sc0415a").await;
    let stranger = login_installed(&state, 2, "sc0415b").await;
    let hidden = create_analysis(&state, &owner).await;

    let first = ask(&state, &stranger, &hidden).await;
    let second = ask(&state, &stranger, &hidden).await;
    assert_eq!(first, second, "재요청의 응답이 첫 요청과 다르다");
    assert_eq!(rows(&state, &hidden).await, 1, "재요청이 행을 늘렸다");

    // A different person asking for the same link is a different request.
    let third = login_installed(&state, 3, "sc0415c").await;
    assert_eq!(ask(&state, &third, &hidden).await, first);
    assert_eq!(rows(&state, &hidden).await, 2);

    state.db.close().await;
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn an_unauthenticated_caller_cannot_ask() {
    let (state, path) = stub_state().await;

    let resp = build_router(state.clone())
        .oneshot(post(&format!("/api/analyses/{ABSENT_ID}/access-request"), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(rows(&state, ABSENT_ID).await, 0);

    state.db.close().await;
    let _ = std::fs::remove_file(&path);
}

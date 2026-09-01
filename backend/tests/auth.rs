//! Auth surface: 401 without a session, 200 with one, idempotent upsert, stub login redirect.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use featuredoc::config::{Config, GithubConfig, Mode};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, db, session, users};

// Path allocation is shared with the other suites rather than copied: this file
// held a second copy that keyed only on (pid, nanos), and on a host whose clock
// advances in coarser steps than a nanosecond two tests in this binary draw the
// same path and race each other's migrations ("table users already exists").
// `common::temp_db_url` carries the per-process counter that makes the path
// unique by construction.
use common::temp_db_url;

async fn stub_state() -> (AppState, PathBuf) {
    stub_state_for_preview(None).await
}

/// A stub-mode state that optionally believes it is the preview for a pull
/// request, which is what makes `login` prefix the OAuth state.
async fn stub_state_for_preview(preview_id: Option<&str>) -> (AppState, PathBuf) {
    let (url, path) = temp_db_url();
    let pool = db::connect(&url).await.expect("connect");
    let config = Arc::new(Config {
        database_url: url,
        base_url: "http://localhost:8080".into(),
        oauth_redirect_base_url: "http://localhost:8080".into(),
        preview_id: preview_id.map(String::from),
        static_dir: "dist".into(),
        kek: [7u8; 32],
        mode: Mode::Stub,
        github: GithubConfig {
            app_private_key: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            app_slug: "featuredoc".into(),
            api_base: "https://api.github.com".into(),
            web_base: "https://github.com".into(),
        },
        cookie_secure: false,
        // These suites never call /internal; an empty token keeps it closed.
        worker_token: String::new(),
    });
    (
        AppState {
            db: pool,
            config,
            http: reqwest::Client::new(),
        },
        path,
    )
}

fn gh(login: &str, id: i64) -> GithubUser {
    GithubUser {
        id,
        login: login.into(),
        name: Some(login.into()),
        avatar_url: None,
    }
}

#[tokio::test]
async fn me_requires_authentication() {
    let (state, path) = stub_state().await;
    let resp = build_router(state)
        .oneshot(Request::builder().uri("/api/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn me_returns_user_with_valid_session() {
    let (state, path) = stub_state().await;
    let user = users::upsert(&state.db, &gh("alice", 1001)).await.unwrap();
    let token = session::create(&state.db, &user.id).await.unwrap();

    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .header(header::COOKIE, format!("fd_session={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["login"], "alice");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn upsert_is_idempotent_by_github_id() {
    let (state, path) = stub_state().await;
    let first = users::upsert(&state.db, &gh("bob", 2002)).await.unwrap();
    let second = users::upsert(&state.db, &gh("bob-renamed", 2002)).await.unwrap();
    assert_eq!(first.id, second.id, "same github_id must map to the same user row");
    assert_eq!(second.login, "bob-renamed", "profile is refreshed on re-login");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count.0, 1);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn stub_login_redirects_to_callback_and_sets_state_cookie() {
    let (state, path) = stub_state().await;
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let location = resp.headers().get(header::LOCATION).unwrap().to_str().unwrap();
    assert!(location.starts_with("/api/auth/callback?code=stub&state="), "got {location}");
    let set_cookie = resp.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(set_cookie.contains("fd_oauth_state="), "got {set_cookie}");
    let _ = std::fs::remove_file(&path);
}

/// A preview deployment cannot register its own callback URL with the GitHub App,
/// so it tags the OAuth state with its pull-request number and lets the redirect
/// proxy on the registered host route the callback back. The cookie must hold the
/// same tagged value, or the callback's CSRF check would reject every preview
/// login.
#[tokio::test]
async fn preview_login_tags_state_with_pull_request_number() {
    let (state, path) = stub_state_for_preview(Some("42")).await;
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let location = resp.headers().get(header::LOCATION).unwrap().to_str().unwrap();
    let sent = location.split("&state=").nth(1).expect("state in redirect");
    assert!(sent.starts_with("pr-42~"), "state not tagged: {location}");
    assert_eq!(sent.len(), "pr-42~".len() + 64, "nonce is still 32 random bytes");

    let set_cookie = resp.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    let stored = common::cookie_value(set_cookie, "fd_oauth_state").expect("state cookie");
    assert_eq!(stored, sent, "cookie must hold the tagged state verbatim");
    let _ = std::fs::remove_file(&path);
}

/// The tag is routing metadata, not a credential: a tagged state still has to
/// match the cookie to open a session, and still fails the check when it does not.
#[tokio::test]
async fn preview_callback_round_trips_the_tagged_state() {
    let (state, path) = stub_state_for_preview(Some("42")).await;
    let router = build_router(state);

    let login = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let location = login.headers().get(header::LOCATION).unwrap().to_str().unwrap().to_string();
    let set_cookie = login.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    let issued = common::cookie_value(set_cookie, "fd_oauth_state").unwrap();

    let ok = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(location.as_str())
                .header(header::COOKIE, format!("fd_oauth_state={issued}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::SEE_OTHER, "tagged state should be accepted");
    let session = ok.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(session.contains("fd_session="), "no session opened: {session}");

    // Same tag, different nonce: the prefix must not buy anything on its own.
    let forged = format!("/api/auth/callback?code=stub&state=pr-42~{}", "0".repeat(64));
    let rejected = router
        .oneshot(
            Request::builder()
                .uri(forged.as_str())
                .header(header::COOKIE, format!("fd_oauth_state={issued}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    let _ = std::fs::remove_file(&path);
}

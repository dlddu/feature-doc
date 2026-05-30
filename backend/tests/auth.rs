//! Auth surface: 401 without a session, 200 with one, idempotent upsert, stub login redirect.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use featuredoc::config::{Config, GithubConfig, Mode};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, db, session, users};

fn temp_db_url() -> (String, PathBuf) {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("featuredoc-auth-{}-{}.db", std::process::id(), nanos));
    (format!("sqlite://{}?mode=rwc", path.display()), path)
}

async fn stub_state() -> (AppState, PathBuf) {
    let (url, path) = temp_db_url();
    let pool = db::connect(&url).await.expect("connect");
    let config = Arc::new(Config {
        database_url: url,
        base_url: "http://localhost:8080".into(),
        static_dir: "dist".into(),
        kek: [7u8; 32],
        mode: Mode::Stub,
        github: GithubConfig {
            app_id: String::new(),
            app_private_key: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            app_slug: "featuredoc".into(),
            api_base: "https://api.github.com".into(),
            web_base: "https://github.com".into(),
        },
        cookie_secure: false,
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

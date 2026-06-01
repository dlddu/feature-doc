//! Auth surface: 401 without a session, 200 with one, idempotent upsert, the login
//! redirect to GitHub, and a full OAuth round-trip against the mock GitHub server.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use url::{Position, Url};

use common::{cookie_value, real_state, stub_state};
use featuredoc::github_api::GithubUser;
use featuredoc::{build_router, session, users};

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
async fn login_redirects_to_github_authorize_and_sets_state_cookie() {
    let (state, mock, path) = real_state().await;
    let expected_prefix = format!("{}/login/oauth/authorize", mock.base_url);

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
    assert!(location.starts_with(&expected_prefix), "got {location}");
    assert!(location.contains("client_id=test-client-id"), "got {location}");
    assert!(location.contains("state="), "got {location}");
    let set_cookie = resp.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(set_cookie.contains("fd_oauth_state="), "got {set_cookie}");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn login_callback_round_trip_via_mock_creates_session() {
    let (state, _mock, path) = real_state().await;
    let router = build_router(state);

    // 1) /login → redirect to the mock authorize page + the CSRF state cookie.
    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let authorize_url = resp
        .headers()
        .get(header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let state_cookie = resp
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|v| cookie_value(v.to_str().ok()?, "fd_oauth_state"))
        .expect("oauth state cookie");

    // 2) the browser hits the mock authorize page, which bounces back to our
    //    callback with a synthetic code. Resolve that one hop with reqwest.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let bounce = client.get(&authorize_url).send().await.unwrap();
    let callback_url = bounce
        .headers()
        .get(reqwest::header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let parsed = Url::parse(&callback_url).unwrap();
    let callback_path_query = parsed[Position::BeforePath..].to_string();

    // 3) drive the callback with the state cookie → session created, redirect home.
    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(&callback_path_query)
                .header(header::COOKIE, format!("fd_oauth_state={state_cookie}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(resp.headers().get(header::LOCATION).unwrap(), "/");
    let session_cookie = resp
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|v| cookie_value(v.to_str().ok()?, "fd_session"))
        .expect("session cookie");

    // 4) /me with the session returns the mock's default user.
    let resp = router
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .header(header::COOKIE, format!("fd_session={session_cookie}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["login"], "stub");
    let _ = std::fs::remove_file(&path);
}

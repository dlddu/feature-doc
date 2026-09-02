//! GitHub App installation surface: connection state, full stub setup round-trip,
//! and short-lived installation tokens that are never persisted.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::{cookie_value, real_state, stub_state};
use featuredoc::github_api::GithubUser;
use featuredoc::{build_router, github_app, github_tokens, session, users};

async fn login_user(state: &featuredoc::state::AppState, login: &str, id: i64) -> String {
    let gh = GithubUser {
        id,
        login: login.into(),
        name: Some(login.into()),
        avatar_url: None,
    };
    let user = users::upsert(&state.db, &gh).await.unwrap();
    session::create(&state.db, &user.id).await.unwrap()
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn connection_when_not_installed_still_lists_requested_permissions() {
    let (state, path) = stub_state().await;
    let token = login_user(&state, "alice", 1).await;

    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/api/github/connection")
                .header(header::COOKIE, format!("fd_session={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["installed"], false);
    assert_eq!(body["permissions"][0], "contents:read");
    assert_eq!(body["permissions"][1], "metadata:read");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn install_url_returns_stub_target_and_sets_setup_cookie() {
    let (state, path) = stub_state().await;
    let token = login_user(&state, "alice", 1).await;

    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/api/github/install-url")
                .header(header::COOKIE, format!("fd_session={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(set_cookie.contains("fd_setup_state="));
    let body = json_body(resp).await;
    let url = body["url"].as_str().unwrap();
    assert!(url.starts_with("/api/github/setup?installation_id="), "got {url}");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn setup_round_trip_marks_connection_installed() {
    let (state, path) = stub_state().await;
    let token = login_user(&state, "alice", 1).await;

    // 1) ask for the install URL + capture the setup-state cookie.
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/github/install-url")
                .header(header::COOKIE, format!("fd_session={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let setup_state = cookie_value(&set_cookie, "fd_setup_state").unwrap();
    let url = json_body(resp).await["url"].as_str().unwrap().to_string();

    // 2) follow the setup callback with both cookies.
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri(&url)
                .header(
                    header::COOKIE,
                    format!("fd_session={token}; fd_setup_state={setup_state}"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);

    // 3) connection now reports installed + the mockup's repo count / account.
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/api/github/connection")
                .header(header::COOKIE, format!("fd_session={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = json_body(resp).await;
    assert_eq!(body["installed"], true);
    assert_eq!(body["repositoryCount"], 3);
    assert_eq!(body["account"]["login"], "stub-account");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn installation_token_is_short_lived_and_not_persisted() {
    let (state, path) = stub_state().await;

    let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM installations")
        .fetch_one(&state.db)
        .await
        .unwrap();

    let token = github_app::mint_installation_token(&state, 4242).await.unwrap();
    assert!(!token.token.is_empty());
    assert!(
        token.expires_at > featuredoc::util::now_unix(),
        "token must expire in the future"
    );

    let after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM installations")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(before.0, after.0, "minting a token must not write any rows");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn github_token_store_load_roundtrips_encrypted() {
    let (state, path) = stub_state().await;
    let user = users::upsert(
        &state.db,
        &GithubUser {
            id: 1,
            login: "alice".into(),
            name: None,
            avatar_url: None,
        },
    )
    .await
    .unwrap();

    featuredoc::github_tokens::store(&state.db, &state.config.kek, &user.id, "gho_secret_token_value")
        .await
        .unwrap();

    let loaded = featuredoc::github_tokens::load(&state.db, &state.config.kek, &user.id)
        .await
        .unwrap();
    assert_eq!(loaded.as_deref(), Some("gho_secret_token_value"));

    // Unknown user -> None.
    assert!(featuredoc::github_tokens::load(&state.db, &state.config.kek, "nobody")
        .await
        .unwrap()
        .is_none());

    // The token must be encrypted at rest, not stored as plaintext.
    let row: (Vec<u8>,) = sqlx::query_as("SELECT ciphertext FROM github_tokens WHERE user_id = ?")
        .bind(&user.id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert!(!row.0.windows(9).any(|w| w == b"gho_secre"));

    let _ = std::fs::remove_file(&path);
}

/// A throwaway stand-in for GitHub's API: answers `GET /user/installations` with
/// `body` and nothing else. Returns the origin to point `api_base` at.
async fn fake_github(body: serde_json::Value) -> String {
    let app = axum::Router::new().route(
        "/user/installations",
        axum::routing::get(move || {
            let body = body.clone();
            async move { axum::Json(body) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// The App is installed on GitHub but nothing links it locally — the state a user
/// lands in when the Setup URL callback never completed. The screen must report
/// the installation it already has rather than offering to make a second one.
#[tokio::test]
async fn connection_adopts_an_installation_github_already_reports() {
    let api = fake_github(serde_json::json!({
        "total_count": 1,
        "installations": [{
            "id": 77,
            "account": { "login": "dlddu", "type": "User" },
            "repository_selection": "selected"
        }]
    }))
    .await;
    let (state, path) = real_state(&api).await;

    let gh = GithubUser {
        id: 1,
        login: "alice".into(),
        name: None,
        avatar_url: None,
    };
    let user = users::upsert(&state.db, &gh).await.unwrap();
    let session = session::create(&state.db, &user.id).await.unwrap();
    github_tokens::store(&state.db, &state.config.kek, &user.id, "gho_test")
        .await
        .unwrap();

    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/github/connection")
                .header(header::COOKIE, format!("fd_session={session}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["installed"], true);
    assert_eq!(body["account"]["login"], "dlddu");
    assert_eq!(body["repositorySelection"], "selected");

    // Adopted for good, not re-fetched on every render.
    let row: (i64,) = sqlx::query_as("SELECT installation_id FROM installations WHERE user_id = ?")
        .bind(&user.id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(row.0, 77);
    let _ = std::fs::remove_file(&path);
}

/// Adoption is best-effort: with no usable OAuth token the lookup fails, and the
/// screen must fall back to "not installed" rather than erroring out.
#[tokio::test]
async fn connection_without_a_usable_token_reports_not_installed() {
    let api = fake_github(serde_json::json!({ "total_count": 0, "installations": [] })).await;
    let (state, path) = real_state(&api).await;
    let token = login_user(&state, "alice", 1).await;

    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/api/github/connection")
                .header(header::COOKIE, format!("fd_session={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["installed"], false);
    let _ = std::fs::remove_file(&path);
}

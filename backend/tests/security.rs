//! Cross-cutting credential safety: secret redaction (AC4.3), per-user isolation
//! across resources (AC4.7 / test#10), and audit recording without secret leakage.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::stub_state;
use featuredoc::config::{Config, GithubConfig, Mode};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

const VALID_KEY: &str = "sk-ant-api03-aaaaaaaaaaaaaaaaaaaa";

async fn login_user(state: &AppState, login: &str, id: i64) -> String {
    let user = users::upsert(
        &state.db,
        &GithubUser {
            id,
            login: login.into(),
            name: None,
            avatar_url: None,
        },
    )
    .await
    .unwrap();
    session::create(&state.db, &user.id).await.unwrap()
}

fn get(uri: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={cookie}"))
        .body(Body::empty())
        .unwrap()
}

fn post_key(cookie: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/llm-keys")
        .header(header::COOKIE, format!("fd_session={cookie}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "provider": "anthropic", "key": VALID_KEY }).to_string(),
        ))
        .unwrap()
}

async fn json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn config_debug_redacts_secrets() {
    let config = Config {
        database_url: "sqlite://x.db".into(),
        base_url: "http://localhost:8080".into(),
        static_dir: "dist".into(),
        kek: [0xAB; 32],
        mode: Mode::Real,
        github: GithubConfig {
            app_id: "12345".into(),
            app_private_key: "-----BEGIN RSA PRIVATE KEY-----secretpem".into(),
            client_id: "Iv1.public".into(),
            client_secret: "shhh-super-secret-value".into(),
            app_slug: "featuredoc".into(),
            api_base: "https://api.github.com".into(),
            web_base: "https://github.com".into(),
        },
        cookie_secure: true,
    };
    let dump = format!("{config:?}");
    assert!(dump.contains("[REDACTED]"));
    assert!(!dump.contains("super-secret-value"), "client secret leaked: {dump}");
    assert!(!dump.contains("BEGIN RSA PRIVATE KEY"), "private key leaked");
    assert!(!dump.contains("ababab"), "kek bytes leaked");
    // Public identifiers are still shown.
    assert!(dump.contains("12345") && dump.contains("Iv1.public"));
}

#[tokio::test]
async fn one_users_credentials_are_invisible_to_another() {
    let (state, path) = stub_state().await;
    let alice = login_user(&state, "alice", 1).await;
    let bob = login_user(&state, "bob", 2).await;

    // Alice connects an installation and registers a key.
    let alice_user: (String,) = sqlx::query_as("SELECT id FROM users WHERE login = 'alice'")
        .fetch_one(&state.db)
        .await
        .unwrap();
    installations::upsert(
        &state.db,
        &alice_user.0,
        &installations::NewInstallation {
            installation_id: 777,
            account_login: Some("alice"),
            account_type: Some("User"),
            repository_selection: Some("selected"),
        },
    )
    .await
    .unwrap();
    let resp = build_router(state.clone()).oneshot(post_key(&alice)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Bob's view of every credential resource is empty / not-installed.
    let conn = json(
        build_router(state.clone())
            .oneshot(get("/api/github/connection", &bob))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(conn["installed"], false);

    let keys = json(
        build_router(state.clone())
            .oneshot(get("/api/llm-keys", &bob))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(keys.as_array().unwrap().len(), 0);

    // Alice still sees her own installation.
    let conn = json(
        build_router(state)
            .oneshot(get("/api/github/connection", &alice))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(conn["installed"], true);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn register_and_revoke_are_audited_without_leaking_the_key() {
    let (state, path) = stub_state().await;
    let token = login_user(&state, "alice", 1).await;

    let created = json(build_router(state.clone()).oneshot(post_key(&token)).await.unwrap()).await;
    let id = created["id"].as_str().unwrap().to_string();

    build_router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/llm-keys/{id}"))
                .header(header::COOKIE, format!("fd_session={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let audit = json(
        build_router(state)
            .oneshot(get("/api/audit", &token))
            .await
            .unwrap(),
    )
    .await;

    let actions: Vec<&str> = audit
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert!(actions.contains(&"llm_key.register"));
    assert!(actions.contains(&"llm_key.revoke"));

    // The whole audit payload must not contain the plaintext key anywhere.
    assert!(!audit.to_string().contains(VALID_KEY), "audit leaked the key");
    let _ = std::fs::remove_file(&path);
}

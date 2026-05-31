//! LLM key lifecycle: register (identifiers only, ciphertext at rest), reject
//! invalid keys, revoke -> blocked, and per-user isolation.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::stub_state;
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, session, users};

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

fn post_json(uri: &str, cookie: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={cookie}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get(uri: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={cookie}"))
        .body(Body::empty())
        .unwrap()
}

async fn json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn register_stores_only_ciphertext_and_lists_identifiers() {
    let (state, path) = stub_state().await;
    let token = login_user(&state, "alice", 1).await;

    let resp = build_router(state.clone())
        .oneshot(post_json(
            "/api/llm-keys",
            &token,
            serde_json::json!({ "provider": "anthropic", "key": VALID_KEY }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = json(resp).await;
    assert_eq!(body["provider"], "anthropic");
    assert!(body["fingerprint"].as_str().unwrap().len() == 16);
    assert!(body["masked"].as_str().unwrap().contains('\u{2022}'));
    assert!(body.get("key").is_none(), "response must not echo the key");

    // At rest: the stored ciphertext must not contain the plaintext key bytes.
    let uid: (String,) = sqlx::query_as("SELECT id FROM users WHERE login = ?")
        .bind("alice")
        .fetch_one(&state.db)
        .await
        .unwrap();
    let row: (Vec<u8>,) = sqlx::query_as("SELECT ciphertext FROM llm_keys WHERE user_id = ?")
        .bind(&uid.0)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert!(
        !contains(&row.0, VALID_KEY.as_bytes()),
        "plaintext key must never be stored"
    );

    // Listing exposes identifiers only.
    let resp = build_router(state).oneshot(get("/api/llm-keys", &token)).await.unwrap();
    let list = json(resp).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert!(list[0].get("ciphertext").is_none());
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn invalid_key_is_rejected() {
    let (state, path) = stub_state().await;
    let token = login_user(&state, "alice", 1).await;
    let resp = build_router(state)
        .oneshot(post_json(
            "/api/llm-keys",
            &token,
            serde_json::json!({ "provider": "anthropic", "key": "bad" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn revoke_blocks_subsequent_use() {
    let (state, path) = stub_state().await;
    let token = login_user(&state, "alice", 1).await;

    let created = json(
        build_router(state.clone())
            .oneshot(post_json(
                "/api/llm-keys",
                &token,
                serde_json::json!({ "provider": "anthropic", "key": VALID_KEY }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();

    // Usable before revocation.
    let resp = build_router(state.clone())
        .oneshot(get("/api/llm-keys/preflight", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Revoke.
    let resp = build_router(state.clone())
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
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Blocked afterwards with the specified message.
    let resp = build_router(state)
        .oneshot(get("/api/llm-keys/preflight", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = json(resp).await;
    assert_eq!(body["error"], "키가 없거나 폐기되었습니다");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn keys_are_isolated_per_user() {
    let (state, path) = stub_state().await;
    let alice = login_user(&state, "alice", 1).await;
    let bob = login_user(&state, "bob", 2).await;

    let created = json(
        build_router(state.clone())
            .oneshot(post_json(
                "/api/llm-keys",
                &alice,
                serde_json::json!({ "provider": "anthropic", "key": VALID_KEY }),
            ))
            .await
            .unwrap(),
    )
    .await;
    let alice_key_id = created["id"].as_str().unwrap().to_string();

    // Bob sees none of Alice's keys.
    let resp = build_router(state.clone()).oneshot(get("/api/llm-keys", &bob)).await.unwrap();
    assert_eq!(json(resp).await.as_array().unwrap().len(), 0);

    // Bob cannot revoke Alice's key.
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/llm-keys/{alice_key_id}"))
                .header(header::COOKIE, format!("fd_session={bob}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Alice's key is still usable.
    let resp = build_router(state)
        .oneshot(get("/api/llm-keys/preflight", &alice))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let _ = std::fs::remove_file(&path);
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

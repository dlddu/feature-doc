//! Integration tests for the S01 credential-setup API. Everything runs in
//! mock mode against an in-memory SQLite database — no network — exercising
//! the AC4.1 / 4.2 / 4.3 / 4.7 contract end to end.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use featuredoc::config::Config;
use featuredoc::state::AppState;
use featuredoc::{db, github, llm, router};

async fn test_router() -> (Router, AppState) {
    let config = Config::for_test();
    let kek = featuredoc::crypto::Kek::from_bytes([3u8; 32]);
    let pool = db::connect_memory().await;
    let http = reqwest::Client::new();
    let gh = github::build(&config.github, http.clone());
    let validator = llm::build(config.llm_mode, http);
    let state = AppState::new(pool, kek, config, gh, validator);
    (router(state.clone()), state)
}

/// Minimal cookie jar: maps name -> value across requests.
#[derive(Default, Clone)]
struct Jar(std::collections::BTreeMap<String, String>);

impl Jar {
    fn header(&self) -> String {
        self.0
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ")
    }
    fn absorb(&mut self, resp: &axum::http::Response<Body>) {
        for hv in resp.headers().get_all(header::SET_COOKIE) {
            if let Ok(s) = hv.to_str() {
                let pair = s.split(';').next().unwrap_or("");
                if let Some((k, v)) = pair.split_once('=') {
                    if v.is_empty() {
                        self.0.remove(k);
                    } else {
                        self.0.insert(k.to_string(), v.to_string());
                    }
                }
            }
        }
    }
}

struct Resp {
    status: StatusCode,
    location: Option<String>,
    body: Value,
}

async fn send(
    app: &Router,
    jar: &mut Jar,
    method: &str,
    uri: &str,
    json_body: Option<Value>,
) -> Resp {
    let mut builder = Request::builder().method(method).uri(uri);
    if !jar.header().is_empty() {
        builder = builder.header(header::COOKIE, jar.header());
    }
    let req = match json_body {
        Some(b) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(b.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
    let resp = app.clone().oneshot(req).await.unwrap();
    jar.absorb(&resp);
    let status = resp.status();
    let location = resp
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Resp {
        status,
        location,
        body,
    }
}

/// Drive the full mock OAuth flow, leaving a valid session in `jar`.
async fn login(app: &Router, jar: &mut Jar) {
    let r = send(app, jar, "GET", "/api/auth/login", None).await;
    assert_eq!(r.status, StatusCode::SEE_OTHER);
    let loc = r.location.expect("login redirect");
    // loc is .../api/auth/callback?code=...&state=...
    let qs = loc.split_once('?').unwrap().1;
    let r = send(app, jar, "GET", &format!("/api/auth/callback?{qs}"), None).await;
    assert_eq!(r.status, StatusCode::SEE_OTHER, "callback should redirect");
}

#[tokio::test]
async fn unauthenticated_requests_are_rejected() {
    let (app, _state) = test_router().await;
    let mut jar = Jar::default();
    for uri in ["/api/me", "/api/llm-keys", "/api/github/connection"] {
        let r = send(&app, &mut jar, "GET", uri, None).await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{uri} must require auth");
    }
}

#[tokio::test]
async fn login_establishes_a_session() {
    let (app, _state) = test_router().await;
    let mut jar = Jar::default();
    login(&app, &mut jar).await;
    let r = send(&app, &mut jar, "GET", "/api/me", None).await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(r.body["login"].as_str().unwrap().starts_with("octo-"));
}

#[tokio::test]
async fn github_install_flow_attributes_installation_to_user() {
    let (app, _state) = test_router().await;
    let mut jar = Jar::default();
    login(&app, &mut jar).await;

    // Before install: not installed, but the requested permissions are shown.
    let r = send(&app, &mut jar, "GET", "/api/github/connection", None).await;
    assert_eq!(r.body["installed"], Value::Bool(false));
    assert!(r.body["permissions"]
        .as_array()
        .unwrap()
        .contains(&Value::String("contents:read".into())));

    // The install URL loops back to setup in mock mode.
    let r = send(&app, &mut jar, "GET", "/api/github/install-url", None).await;
    let url = r.body["url"].as_str().unwrap();
    let qs = url.split_once('?').unwrap().1;
    let r = send(&app, &mut jar, "GET", &format!("/api/github/setup?{qs}"), None).await;
    assert_eq!(r.status, StatusCode::SEE_OTHER);

    // After install: connection reflects the mockup's values.
    let r = send(&app, &mut jar, "GET", "/api/github/connection", None).await;
    assert_eq!(r.body["installed"], Value::Bool(true));
    assert_eq!(r.body["repo_count"], Value::from(3));
    assert_eq!(r.body["account"], Value::String("octo-org".into()));
}

#[tokio::test]
async fn llm_key_is_validated_stored_encrypted_and_listed_masked() {
    let (app, state) = test_router().await;
    let mut jar = Jar::default();
    login(&app, &mut jar).await;

    let plaintext = "sk-ant-supersecretkey-abcd1234";
    let r = send(
        &app,
        &mut jar,
        "POST",
        "/api/llm-keys",
        Some(serde_json::json!({ "provider": "anthropic", "key": plaintext })),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "valid key should register");
    assert_eq!(r.body["provider"], Value::String("anthropic".into()));
    let masked = r.body["masked"].as_str().unwrap();
    assert!(masked.contains('•'));
    assert!(!masked.contains("supersecret"));
    // The response carries no plaintext key field.
    assert!(r.body.get("key").is_none());

    // The persisted row must not contain the plaintext anywhere.
    let rows: Vec<(Vec<u8>, Vec<u8>, String, String)> = sqlx::query_as(
        "SELECT ciphertext, dek_wrapped, masked, fingerprint FROM llm_keys",
    )
    .fetch_all(&state.db)
    .await
    .unwrap();
    assert_eq!(rows.len(), 1);
    let needle = b"supersecret";
    for (ct, dek, masked, fp) in &rows {
        assert!(!ct.windows(needle.len()).any(|w| w == needle), "ciphertext leaks plaintext");
        assert!(!dek.windows(needle.len()).any(|w| w == needle), "wrapped DEK leaks plaintext");
        assert!(!masked.contains("supersecret"));
        assert!(!fp.contains("supersecret"));
    }

    // Listing returns identifiers only.
    let r = send(&app, &mut jar, "GET", "/api/llm-keys", None).await;
    let list = r.body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].get("key").is_none());
    assert!(list[0].get("ciphertext").is_none());
    assert_eq!(list[0]["status"], Value::String("active".into()));
}

#[tokio::test]
async fn invalid_key_is_rejected_and_not_stored() {
    let (app, state) = test_router().await;
    let mut jar = Jar::default();
    login(&app, &mut jar).await;

    let r = send(
        &app,
        &mut jar,
        "POST",
        "/api/llm-keys",
        Some(serde_json::json!({ "provider": "anthropic", "key": "sk-ant-invalid-key" })),
    )
    .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM llm_keys")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(count.0, 0, "rejected key must not be persisted");
}

#[tokio::test]
async fn revoked_key_blocks_subsequent_use() {
    let (app, state) = test_router().await;
    let mut jar = Jar::default();
    login(&app, &mut jar).await;

    let r = send(
        &app,
        &mut jar,
        "POST",
        "/api/llm-keys",
        Some(serde_json::json!({ "provider": "anthropic", "key": "sk-ant-valid-key-abcd1234" })),
    )
    .await;
    let id = r.body["id"].as_str().unwrap().to_string();

    // active_key_for resolves while active...
    let user_id: (String,) = sqlx::query_as("SELECT user_id FROM llm_keys WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert!(featuredoc::llmkey::active_key_for(
        &state,
        &user_id.0,
        featuredoc::llm::Provider::Anthropic
    )
    .await
    .is_ok());

    // Revoke it.
    let r = send(&app, &mut jar, "DELETE", &format!("/api/llm-keys/{id}"), None).await;
    assert_eq!(r.status, StatusCode::OK);

    // ...and is blocked afterwards (AC4.2).
    let blocked = featuredoc::llmkey::active_key_for(
        &state,
        &user_id.0,
        featuredoc::llm::Provider::Anthropic,
    )
    .await;
    assert!(blocked.is_err());

    // Revoking again is a not-found.
    let r = send(&app, &mut jar, "DELETE", &format!("/api/llm-keys/{id}"), None).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn users_cannot_touch_each_others_resources() {
    let (app, _state) = test_router().await;

    // User A logs in and registers a key + installs the App.
    let mut jar_a = Jar::default();
    login(&app, &mut jar_a).await;
    let r = send(
        &app,
        &mut jar_a,
        "POST",
        "/api/llm-keys",
        Some(serde_json::json!({ "provider": "anthropic", "key": "sk-ant-akey-abcd12345678" })),
    )
    .await;
    let a_key_id = r.body["id"].as_str().unwrap().to_string();
    let r = send(&app, &mut jar_a, "GET", "/api/github/install-url", None).await;
    let qs = r.body["url"].as_str().unwrap().split_once('?').unwrap().1.to_string();
    send(&app, &mut jar_a, "GET", &format!("/api/github/setup?{qs}"), None).await;

    // User B logs in separately (distinct mock identity).
    let mut jar_b = Jar::default();
    login(&app, &mut jar_b).await;

    // B sees an empty key list and no installation — isolation of reads.
    let r = send(&app, &mut jar_b, "GET", "/api/llm-keys", None).await;
    assert_eq!(r.body.as_array().unwrap().len(), 0);
    let r = send(&app, &mut jar_b, "GET", "/api/github/connection", None).await;
    assert_eq!(r.body["installed"], Value::Bool(false));

    // B cannot revoke A's key — scoped query yields not-found (AC4.7).
    let r = send(&app, &mut jar_b, "DELETE", &format!("/api/llm-keys/{a_key_id}"), None).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);

    // A's key is still active.
    let r = send(&app, &mut jar_a, "GET", "/api/llm-keys", None).await;
    assert_eq!(r.body[0]["status"], Value::String("active".into()));
}

//! An in-process mock of the GitHub endpoints FeatureDoc calls.
//!
//! This replaces the old `Mode::Stub` in-code doubles. The application now always
//! runs the real HTTP path (`github_api` / `github_app`); to stay hermetic, tests
//! and the kind e2e simply point the GitHub base URLs at this server instead of at
//! `github.com` / `api.github.com`.
//!
//! The mock is intentionally permissive: it does not verify the App JWT, the OAuth
//! client secret, or any signature. It only needs to return GitHub-shaped,
//! internally-consistent, deterministic data. Identity is carried end-to-end by
//! the OAuth `code` (which is just the login handle) → access token
//! (`gho_mock_<login>`) → `/user`, so distinct logins yield distinct users and the
//! isolation tests can log in as several identities at will.

use axum::extract::{Form, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Configuration for the mock.
#[derive(Clone)]
pub struct MockConfig {
    /// The FeatureDoc app's own public origin. Browser-facing redirects (the OAuth
    /// authorize page and the App installation page) send the browser back here.
    pub app_base_url: String,
    /// Login handle used when a browser hits the authorize / install pages without
    /// an explicit `?login=` hint (the real app never sends one).
    pub default_login: String,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            app_base_url: "http://localhost:8080".to_string(),
            default_login: "stub".to_string(),
        }
    }
}

/// Deterministic GitHub user id for a login handle (positive, SQLite-friendly).
pub fn user_id_for(login: &str) -> i64 {
    let digest = Sha256::digest(login.as_bytes());
    let mut head = [0u8; 8];
    head.copy_from_slice(&digest[..8]);
    // Shift to guarantee a positive i64.
    (u64::from_be_bytes(head) >> 1) as i64
}

/// Deterministic installation id for a login handle (distinct logins → distinct ids).
pub fn installation_id_for(login: &str) -> i64 {
    10_000 + user_id_for(login).rem_euclid(90_000)
}

/// The OAuth access token the mock issues for a login. It encodes the login so the
/// later `/user` and `/user/installations` calls can recover the identity.
pub fn access_token_for(login: &str) -> String {
    format!("gho_mock_{login}")
}

/// Recovers the login from a `gho_mock_<login>` bearer token, if present.
fn login_from_auth(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("authorization")?.to_str().ok()?;
    let tok = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("token "))
        .unwrap_or(raw);
    tok.strip_prefix("gho_mock_").map(|s| s.to_string())
}

/// Builds the mock GitHub router. Mount it on any listener and point the app's
/// `GITHUB_API_BASE` / `GITHUB_OAUTH_BASE` / `GITHUB_WEB_BASE` at it.
pub fn router(config: MockConfig) -> Router {
    Router::new()
        // Health probe (used by the e2e readiness check).
        .route("/healthz", get(|| async { "ok" }))
        // User-authorization (login) flow.
        .route("/login/oauth/authorize", get(authorize))
        .route("/login/oauth/access_token", post(access_token))
        .route("/user", get(get_user))
        .route("/user/installations", get(user_installations))
        // App installation flow.
        .route("/apps/{slug}/installations/new", get(install_new))
        .route(
            "/app/installations/{id}/access_tokens",
            post(installation_token),
        )
        .route("/app/installations/{id}", get(installation))
        .route("/installation/repositories", get(installation_repositories))
        .with_state(config)
}

#[derive(Deserialize)]
struct AuthorizeParams {
    redirect_uri: Option<String>,
    state: Option<String>,
    /// Test/e2e affordance to pick the synthetic identity; the real app never sends it.
    login: Option<String>,
}

/// OAuth authorize page. Real GitHub would show a consent screen; the mock bounces
/// the browser straight back to `redirect_uri` with a synthetic `code` (the login).
async fn authorize(State(cfg): State<MockConfig>, Query(p): Query<AuthorizeParams>) -> Redirect {
    let login = p.login.unwrap_or(cfg.default_login);
    let redirect_uri = p
        .redirect_uri
        .unwrap_or_else(|| format!("{}/api/auth/callback", cfg.app_base_url));
    let state = p.state.unwrap_or_default();

    let mut url = url::Url::parse(&redirect_uri).unwrap_or_else(|_| {
        url::Url::parse(&format!("{}/api/auth/callback", cfg.app_base_url)).unwrap()
    });
    url.query_pairs_mut()
        .append_pair("code", &login)
        .append_pair("state", &state);
    Redirect::to(url.as_str())
}

#[derive(Deserialize)]
struct TokenForm {
    code: Option<String>,
}

/// OAuth token exchange. Turns the `code` (a login handle) into an opaque token.
async fn access_token(Form(f): Form<TokenForm>) -> Json<Value> {
    let login = f.code.unwrap_or_default();
    Json(json!({
        "access_token": access_token_for(&login),
        "token_type": "bearer",
        "scope": "",
    }))
}

/// The authenticated user for an OAuth token.
async fn get_user(headers: HeaderMap) -> impl IntoResponse {
    match login_from_auth(&headers) {
        Some(login) => Json(json!({
            "id": user_id_for(&login),
            "login": login,
            "name": format!("Stub {login}"),
            "avatar_url": Value::Null,
        }))
        .into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

/// Installations visible to the OAuth user — one, derived from their login. Used by
/// the app to confirm a user actually owns the installation id from the Setup URL.
async fn user_installations(headers: HeaderMap) -> impl IntoResponse {
    match login_from_auth(&headers) {
        Some(login) => Json(json!({
            "installations": [ { "id": installation_id_for(&login) } ],
        }))
        .into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

/// App installation page. Real GitHub would let the user pick repositories; the mock
/// redirects the browser to the app's Setup URL with a deterministic installation id.
async fn install_new(
    State(cfg): State<MockConfig>,
    Path(_slug): Path<String>,
    Query(p): Query<AuthorizeParams>,
) -> Redirect {
    let login = p.login.unwrap_or(cfg.default_login);
    let state = p.state.unwrap_or_default();
    let iid = installation_id_for(&login);
    let target = format!(
        "{}/api/github/setup?installation_id={iid}&setup_action=install&state={state}",
        cfg.app_base_url
    );
    Redirect::to(&target)
}

/// A short-lived installation access token. The app never persists it.
async fn installation_token(Path(id): Path<i64>) -> Json<Value> {
    let expires_at = (time::OffsetDateTime::now_utc() + time::Duration::hours(1))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "2099-01-01T00:00:00Z".to_string());
    Json(json!({
        "token": format!("ghs_mock_{id}_{}", crate::util::random_token()),
        "expires_at": expires_at,
    }))
}

/// Installation metadata (canned, matching the values the suite asserts on).
async fn installation(Path(_id): Path<i64>) -> Json<Value> {
    Json(json!({
        "account": { "login": "stub-account", "type": "User" },
        "repository_selection": "selected",
    }))
}

/// Repository count for the installation (best-effort display value).
async fn installation_repositories() -> Json<Value> {
    Json(json!({ "total_count": 3, "repositories": [] }))
}

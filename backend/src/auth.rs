//! "Sign in with GitHub" using the FeatureDoc App's user-authorization flow,
//! plus session issuance and the `CurrentUser` extractor that gates every
//! protected route. No long-lived user secret is stored: the OAuth token is
//! used once to identify the user, then dropped.

use axum::{
    extract::{FromRequestParts, Query, State},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::config::Mode;
use crate::db::now;
use crate::error::AppError;
use crate::state::AppState;

const SESSION_COOKIE: &str = "fd_session";
const STATE_COOKIE: &str = "fd_oauth_state";
const SESSION_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// An authenticated user, resolved from the session cookie. Extracting this
/// is what enforces authentication — handlers that take it cannot run
/// unauthenticated.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: String,
    pub login: String,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app = AppState::from_ref(state);
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get(SESSION_COOKIE).map(|c| c.value().to_string());
        let Some(token) = token else {
            return Err(AppError::Unauthorized);
        };
        let hash = hash_token(&token);
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT u.id, u.login FROM sessions s \
             JOIN users u ON u.id = s.user_id \
             WHERE s.token_hash = ? AND s.expires_at > ?",
        )
        .bind(hash)
        .bind(now())
        .fetch_optional(&app.db)
        .await?;
        match row {
            Some((id, login)) => Ok(CurrentUser { id, login }),
            None => Err(AppError::Unauthorized),
        }
    }
}

// Re-export so the extractor bound reads cleanly.
use axum::extract::FromRef;

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn random_token() -> String {
    let mut b = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut b);
    hex::encode(b)
}

fn cookie<'a>(name: &'a str, value: String, secure: bool, max_age_secs: i64) -> Cookie<'a> {
    let mut c = Cookie::new(name.to_string(), value);
    c.set_http_only(true);
    c.set_same_site(SameSite::Lax);
    c.set_secure(secure);
    c.set_path("/");
    c.set_max_age(time::Duration::seconds(max_age_secs));
    c
}

/// GET /api/auth/login — start the OAuth dance.
pub async fn login(State(app): State<AppState>) -> Response {
    let mut nonce = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let state = hex::encode(nonce);

    let jar = CookieJar::new().add(cookie(
        STATE_COOKIE,
        state.clone(),
        app.config.cookie_secure,
        600,
    ));

    let redirect_uri = format!("{}/api/auth/callback", app.config.base_url);
    let location = match app.config.github.mode {
        // No external GitHub in mock mode: loop straight back to our callback.
        Mode::Mock => format!(
            "{}/api/auth/callback?code=mockcode-{}&state={}",
            app.config.base_url, state, state
        ),
        Mode::Real => format!(
            "{}/login/oauth/authorize?client_id={}&redirect_uri={}&state={}",
            app.config.github.web_base,
            app.config.github.client_id,
            urlencoding(&redirect_uri),
            state
        ),
    };
    (jar, Redirect::to(&location)).into_response()
}

fn urlencoding(s: &str) -> String {
    s.replace(':', "%3A").replace('/', "%2F")
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

/// GET /api/auth/callback — finish OAuth, upsert the user, open a session.
pub async fn callback(
    State(app): State<AppState>,
    jar: CookieJar,
    Query(q): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    let expected = jar.get(STATE_COOKIE).map(|c| c.value().to_string());
    if expected.as_deref() != Some(q.state.as_str()) {
        return Err(AppError::BadRequest("invalid oauth state".into()));
    }

    // Identify the user. The token is used here and then dropped.
    let token = app.github.exchange_code(&q.code).await?;
    let gh = app.github.get_user(&token).await?;
    drop(token);

    let user_id = upsert_user(&app, &gh).await?;
    let session_token = random_token();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, token_hash, user_id, created_at, expires_at) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(hash_token(&session_token))
    .bind(&user_id)
    .bind(now())
    .bind(now() + SESSION_TTL_SECS)
    .execute(&app.db)
    .await?;

    crate::audit::record(&app.db, Some(&user_id), "auth.login", &gh.login).await;

    let jar = jar
        .remove(Cookie::from(STATE_COOKIE))
        .add(cookie(
            SESSION_COOKIE,
            session_token,
            app.config.cookie_secure,
            SESSION_TTL_SECS,
        ));
    Ok((jar, Redirect::to(&format!("{}/", app.config.base_url))).into_response())
}

async fn upsert_user(app: &AppState, gh: &crate::github::GitHubUser) -> Result<String, AppError> {
    if let Some((id,)) = sqlx::query_as::<_, (String,)>("SELECT id FROM users WHERE github_id = ?")
        .bind(gh.github_id)
        .fetch_optional(&app.db)
        .await?
    {
        sqlx::query("UPDATE users SET login = ?, avatar_url = ? WHERE id = ?")
            .bind(&gh.login)
            .bind(&gh.avatar_url)
            .bind(&id)
            .execute(&app.db)
            .await?;
        return Ok(id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, github_id, login, avatar_url, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(gh.github_id)
    .bind(&gh.login)
    .bind(&gh.avatar_url)
    .bind(now())
    .execute(&app.db)
    .await?;
    Ok(id)
}

/// POST /api/auth/logout — drop the session.
pub async fn logout(State(app): State<AppState>, jar: CookieJar) -> Result<Response, AppError> {
    if let Some(c) = jar.get(SESSION_COOKIE) {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(hash_token(c.value()))
            .execute(&app.db)
            .await?;
    }
    let jar = jar.remove(Cookie::from(SESSION_COOKIE));
    Ok((jar, Json(json!({ "ok": true }))).into_response())
}

/// GET /api/me — who am I?
pub async fn me(user: CurrentUser) -> Json<serde_json::Value> {
    Json(json!({ "id": user.id, "login": user.login }))
}

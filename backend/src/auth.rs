//! Login, logout, session cookie, and the authenticated-user extractor.

use axum::extract::{FromRequestParts, Query, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::config::Mode;
use crate::error::AppError;
use crate::models::User;
use crate::state::AppState;
use crate::{cookies, github_api, session, users, util};

const STATE_COOKIE: &str = "fd_oauth_state";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", get(login))
        .route("/api/auth/callback", get(callback))
        .route("/api/auth/logout", post(logout))
        .route("/api/me", get(me))
}

#[derive(Deserialize, Default)]
struct LoginParams {
    /// Stub-mode only: pick which synthetic identity to log in as.
    #[serde(rename = "as")]
    as_user: Option<String>,
}

/// Begins login. Real mode redirects to GitHub's App user-authorization page;
/// stub mode bounces straight back to our callback with a synthetic code.
async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<LoginParams>,
) -> Result<(CookieJar, Redirect), AppError> {
    let nonce = util::random_token();
    let jar = jar.add(cookies::make(&state, STATE_COOKIE, nonce.clone()));

    let location = match state.config.mode {
        Mode::Stub => {
            let code = sanitize_handle(params.as_user.as_deref().unwrap_or("stub"));
            format!("/api/auth/callback?code={code}&state={nonce}")
        }
        Mode::Real => {
            let redirect_uri = format!("{}/api/auth/callback", state.config.base_url);
            let mut url = url::Url::parse(&format!(
                "{}/login/oauth/authorize",
                state.config.github.web_base
            ))
            .map_err(|_| AppError::internal("invalid web_base"))?;
            url.query_pairs_mut()
                .append_pair("client_id", &state.config.github.client_id)
                .append_pair("redirect_uri", &redirect_uri)
                .append_pair("state", &nonce);
            url.to_string()
        }
    };

    Ok((jar, Redirect::to(&location)))
}

#[derive(Deserialize)]
struct CallbackParams {
    code: String,
    state: String,
}

/// Completes login: validates the CSRF state, resolves the GitHub user, upserts
/// it, opens a session, and redirects to the SPA.
async fn callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<CallbackParams>,
) -> Result<(CookieJar, Redirect), AppError> {
    let expected = jar.get(STATE_COOKIE).map(|c| c.value().to_string());
    if expected.as_deref() != Some(params.state.as_str()) {
        return Err(AppError::BadRequest("invalid oauth state".into()));
    }

    let gh = github_api::exchange_code_for_user(&state, &params.code).await?;
    let user = users::upsert(&state.db, &gh).await?;
    let token = session::create(&state.db, &user.id).await?;
    crate::audit::record(&state.db, Some(&user.id), "auth.login", None).await;

    let jar = jar
        .remove(cookies::removal(STATE_COOKIE))
        .add(cookies::make(&state, session::SESSION_COOKIE, token));

    Ok((jar, Redirect::to("/")))
}

/// Ends the session and clears the cookie.
async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AppError> {
    if let Some(c) = jar.get(session::SESSION_COOKIE) {
        session::delete(&state.db, c.value()).await?;
    }
    let jar = jar.remove(cookies::removal(session::SESSION_COOKIE));
    Ok((jar, StatusCode::NO_CONTENT))
}

/// Returns the authenticated user, or 401.
async fn me(CurrentUser(user): CurrentUser) -> Json<UserView> {
    Json(UserView::from(user))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserView {
    id: String,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

impl From<User> for UserView {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            login: u.login,
            name: u.name,
            avatar_url: u.avatar_url,
        }
    }
}

/// Extractor that resolves the session cookie to the current [`User`], or rejects
/// with 401. Protected handlers take this to require authentication.
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar
            .get(session::SESSION_COOKIE)
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthorized)?;
        match session::lookup_user(&state.db, &token).await? {
            Some(user) => Ok(CurrentUser(user)),
            None => Err(AppError::Unauthorized),
        }
    }
}

fn sanitize_handle(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(39)
        .collect();
    if cleaned.is_empty() {
        "stub".to_string()
    } else {
        cleaned.to_ascii_lowercase()
    }
}

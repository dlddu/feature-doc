//! Thin GitHub client for the user-authorization (login) flow.
//!
//! Always talks to whatever `GITHUB_OAUTH_BASE` / `GITHUB_API_BASE` point at — real
//! GitHub in production, or the mock server ([`crate::mock_github`]) in tests and
//! the kind e2e. Error messages here are deliberately generic — tokens and secrets
//! never appear in them (AC4.3).

use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

/// The subset of a GitHub user we persist.
pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Result of resolving a login: the user plus their OAuth token, kept to verify
/// installation ownership later.
pub struct AuthOutcome {
    pub user: GithubUser,
    pub token: Option<String>,
}

/// Exchanges an authorization `code` for the authenticated GitHub user.
pub async fn exchange_code_for_user(
    state: &AppState,
    code: &str,
) -> Result<AuthOutcome, AppError> {
    let token = exchange_code(state, code).await?;
    let user = fetch_user(state, &token).await?;
    Ok(AuthOutcome {
        user,
        token: Some(token),
    })
}

async fn exchange_code(state: &AppState, code: &str) -> Result<String, AppError> {
    let url = format!("{}/login/oauth/access_token", state.config.github.oauth_base);
    let redirect_uri = format!("{}/api/auth/callback", state.config.base_url);

    let resp = state
        .http
        .post(&url)
        .header("Accept", "application/json")
        .form(&[
            ("client_id", state.config.github.client_id.as_str()),
            ("client_secret", state.config.github.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|_| AppError::internal("github token exchange request failed"))?;

    #[derive(Deserialize)]
    struct TokenResp {
        access_token: Option<String>,
    }
    let body: TokenResp = resp
        .json()
        .await
        .map_err(|_| AppError::internal("github token exchange: malformed response"))?;

    body.access_token.ok_or(AppError::Unauthorized)
}

async fn fetch_user(state: &AppState, token: &str) -> Result<GithubUser, AppError> {
    let url = format!("{}/user", state.config.github.api_base);
    let resp = state
        .http
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "featuredoc/0.1")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|_| AppError::internal("github get-user request failed"))?;

    if !resp.status().is_success() {
        return Err(AppError::Unauthorized);
    }

    #[derive(Deserialize)]
    struct GhUser {
        id: i64,
        login: String,
        name: Option<String>,
        avatar_url: Option<String>,
    }
    let u: GhUser = resp
        .json()
        .await
        .map_err(|_| AppError::internal("github get-user: malformed response"))?;

    Ok(GithubUser {
        id: u.id,
        login: u.login,
        name: u.name,
        avatar_url: u.avatar_url,
    })
}

//! Thin GitHub client for the user-authorization (login) flow.
//!
//! Upstream failures are mapped to fixed strings rather than interpolated: the
//! client secret, the OAuth code, and the access token must not reach a log line.

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::config::Mode;
use crate::error::AppError;
use crate::state::AppState;

pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

pub struct AuthOutcome {
    pub user: GithubUser,
    pub token: Option<String>,
}

pub async fn exchange_code_for_user(
    state: &AppState,
    code: &str,
) -> Result<AuthOutcome, AppError> {
    match state.config.doubles.github_auth {
        // mock-exception: EXT-01 — 실제 GitHub OAuth 동의 화면·계정은 kind CI에서 왕복 불가
        Mode::Stub => Ok(AuthOutcome {
            user: stub_user_from_code(code),
            token: None,
        }),
        Mode::Real => {
            let token = exchange_code(state, code).await?;
            let user = fetch_user(state, &token).await?;
            Ok(AuthOutcome {
                user,
                token: Some(token),
            })
        }
    }
}

async fn exchange_code(state: &AppState, code: &str) -> Result<String, AppError> {
    let url = format!("{}/login/oauth/access_token", state.config.github.web_base);
    // Must match the redirect_uri sent to /login/oauth/authorize exactly, so this
    // is the registered origin rather than our own — they differ on previews.
    let redirect_uri = format!(
        "{}/api/auth/callback",
        state.config.oauth_redirect_base_url
    );

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

/// Distinct `code`s must yield distinct users — `?as=<handle>` spec isolation rests on it.
fn stub_user_from_code(code: &str) -> GithubUser {
    GithubUser {
        id: stable_id(code),
        login: code.to_string(),
        name: Some(format!("Stub {code}")),
        avatar_url: None,
    }
}

fn stable_id(code: &str) -> i64 {
    let digest = Sha256::digest(code.as_bytes());
    let mut head = [0u8; 8];
    head.copy_from_slice(&digest[..8]);
    // Shift to guarantee a positive, SQLite-friendly i64.
    (u64::from_be_bytes(head) >> 1) as i64
}

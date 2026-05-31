//! GitHub App credentials: short-lived installation tokens (minted on demand,
//! never persisted — AC4.1) and installation metadata.
//!
//! `Mode::Stub` answers everything in-process. Real mode signs an App JWT with the
//! App private key and calls the GitHub API. Error messages never include the JWT,
//! the token, or the key (AC4.3).

use serde::{Deserialize, Serialize};

use crate::config::Mode;
use crate::error::AppError;
use crate::state::AppState;
use crate::util::{now_unix, rfc3339_to_unix};

/// The minimal, read-only scopes the App requests — shown to the user before they
/// install (journey F1) and alongside the installed state (mockup).
pub const REQUESTED_PERMISSIONS: &[&str] = &["contents:read", "metadata:read"];

/// A short-lived installation access token. Returned to callers for immediate use;
/// it is never written to the database.
pub struct InstallationToken {
    pub token: String,
    pub expires_at: i64,
}

pub struct InstallationInfo {
    pub account_login: Option<String>,
    pub account_type: Option<String>,
    pub repository_selection: Option<String>,
}

/// Mints a fresh installation access token for `installation_id`.
pub async fn mint_installation_token(
    state: &AppState,
    installation_id: i64,
) -> Result<InstallationToken, AppError> {
    match state.config.mode {
        Mode::Stub => Ok(InstallationToken {
            token: format!("ghs_stub_{installation_id}_{}", crate::util::random_token()),
            expires_at: now_unix() + 3600,
        }),
        Mode::Real => {
            let jwt = app_jwt(state)?;
            let url = format!(
                "{}/app/installations/{}/access_tokens",
                state.config.github.api_base, installation_id
            );
            let resp = state
                .http
                .post(&url)
                .header("Authorization", format!("Bearer {jwt}"))
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "featuredoc/0.1")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await
                .map_err(|_| AppError::internal("github installation-token request failed"))?;
            if !resp.status().is_success() {
                return Err(AppError::internal("github installation-token rejected"));
            }
            #[derive(Deserialize)]
            struct R {
                token: String,
                expires_at: String,
            }
            let r: R = resp
                .json()
                .await
                .map_err(|_| AppError::internal("github installation-token: malformed response"))?;
            Ok(InstallationToken {
                token: r.token,
                expires_at: rfc3339_to_unix(&r.expires_at).unwrap_or_else(|| now_unix() + 3600),
            })
        }
    }
}

/// Looks up installation metadata (account + repository selection) via the App JWT.
pub async fn fetch_installation(
    state: &AppState,
    installation_id: i64,
) -> Result<InstallationInfo, AppError> {
    match state.config.mode {
        Mode::Stub => Ok(InstallationInfo {
            account_login: Some("stub-account".to_string()),
            account_type: Some("User".to_string()),
            repository_selection: Some("selected".to_string()),
        }),
        Mode::Real => {
            let jwt = app_jwt(state)?;
            let url = format!(
                "{}/app/installations/{}",
                state.config.github.api_base, installation_id
            );
            let resp = state
                .http
                .get(&url)
                .header("Authorization", format!("Bearer {jwt}"))
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "featuredoc/0.1")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await
                .map_err(|_| AppError::internal("github installation lookup failed"))?;
            if !resp.status().is_success() {
                return Err(AppError::internal("github installation lookup rejected"));
            }
            #[derive(Deserialize)]
            struct Account {
                login: Option<String>,
                #[serde(rename = "type")]
                kind: Option<String>,
            }
            #[derive(Deserialize)]
            struct R {
                account: Option<Account>,
                repository_selection: Option<String>,
            }
            let r: R = resp
                .json()
                .await
                .map_err(|_| AppError::internal("github installation lookup: malformed response"))?;
            Ok(InstallationInfo {
                account_login: r.account.as_ref().and_then(|a| a.login.clone()),
                account_type: r.account.and_then(|a| a.kind),
                repository_selection: r.repository_selection,
            })
        }
    }
}

/// Best-effort count of repositories the installation can access (for display).
pub async fn repository_count(state: &AppState, installation_id: i64) -> Option<i64> {
    match state.config.mode {
        Mode::Stub => Some(3),
        Mode::Real => {
            let token = mint_installation_token(state, installation_id).await.ok()?;
            let url = format!(
                "{}/installation/repositories?per_page=1",
                state.config.github.api_base
            );
            let resp = state
                .http
                .get(&url)
                .header("Authorization", format!("Bearer {}", token.token))
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "featuredoc/0.1")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await
                .ok()?;
            #[derive(Deserialize)]
            struct R {
                total_count: i64,
            }
            let r: R = resp.json().await.ok()?;
            Some(r.total_count)
        }
    }
}

/// A repository an installation can access — owner, name, and default branch.
pub struct RepoRef {
    pub owner: String,
    pub name: String,
    pub default_branch: String,
}

/// Lists the repositories the installation can access (the candidates a user may
/// connect — S02/S03). Stub mode returns a deterministic set whose size matches
/// [`repository_count`]'s stub (3). Real mode lists them with an installation token.
pub async fn list_repositories(
    state: &AppState,
    installation_id: i64,
) -> Result<Vec<RepoRef>, AppError> {
    match state.config.mode {
        Mode::Stub => Ok(vec![
            RepoRef {
                owner: "stub-account".to_string(),
                name: "payments-api".to_string(),
                default_branch: "main".to_string(),
            },
            RepoRef {
                owner: "stub-account".to_string(),
                name: "checkout-web".to_string(),
                default_branch: "main".to_string(),
            },
            RepoRef {
                owner: "stub-account".to_string(),
                name: "notif-worker".to_string(),
                default_branch: "main".to_string(),
            },
        ]),
        Mode::Real => {
            let token = mint_installation_token(state, installation_id).await?;
            let url = format!(
                "{}/installation/repositories?per_page=100",
                state.config.github.api_base
            );
            let resp = state
                .http
                .get(&url)
                .header("Authorization", format!("Bearer {}", token.token))
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "featuredoc/0.1")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await
                .map_err(|_| AppError::internal("github repositories lookup failed"))?;
            if !resp.status().is_success() {
                return Err(AppError::internal("github repositories lookup rejected"));
            }
            #[derive(Deserialize)]
            struct Owner {
                login: String,
            }
            #[derive(Deserialize)]
            struct Repo {
                name: String,
                owner: Owner,
                default_branch: Option<String>,
            }
            #[derive(Deserialize)]
            struct R {
                repositories: Vec<Repo>,
            }
            let r: R = resp
                .json()
                .await
                .map_err(|_| AppError::internal("github repositories: malformed response"))?;
            Ok(r
                .repositories
                .into_iter()
                .map(|repo| RepoRef {
                    owner: repo.owner.login,
                    name: repo.name,
                    default_branch: repo.default_branch.unwrap_or_else(|| "main".to_string()),
                })
                .collect())
        }
    }
}

/// Verifies the signed-in user actually has access to `installation_id` before we
/// link it. The Setup URL's installation_id is spoofable (GitHub docs), so in real
/// mode we list the user's installations with their OAuth token and require a
/// match. Stub mode trusts the synthetic id.
pub async fn verify_user_owns_installation(
    state: &AppState,
    user_id: &str,
    installation_id: i64,
) -> Result<(), AppError> {
    if state.config.mode == Mode::Stub {
        return Ok(());
    }

    let token = crate::github_tokens::load(&state.db, &state.config.kek, user_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("GitHub 재인증이 필요합니다".into()))?;

    let url = format!(
        "{}/user/installations?per_page=100",
        state.config.github.api_base
    );
    let resp = state
        .http
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "featuredoc/0.1")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|_| AppError::internal("github installations lookup failed"))?;
    if !resp.status().is_success() {
        return Err(AppError::BadRequest("GitHub 재인증이 필요합니다".into()));
    }

    #[derive(Deserialize)]
    struct Inst {
        id: i64,
    }
    #[derive(Deserialize)]
    struct R {
        installations: Vec<Inst>,
    }
    let body: R = resp
        .json()
        .await
        .map_err(|_| AppError::internal("github installations: malformed response"))?;

    if body.installations.iter().any(|i| i.id == installation_id) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Signs a short-lived (≈9 min) RS256 App JWT with the App private key, using the
/// client ID as the `iss` claim — GitHub's recommended identifier as of 2024-05
/// (the numeric App ID also works, but compatibility with future features relies
/// on the client ID).
fn app_jwt(state: &AppState) -> Result<String, AppError> {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

    #[derive(Serialize)]
    struct Claims {
        iat: i64,
        exp: i64,
        iss: String,
    }

    let now = now_unix();
    let claims = Claims {
        iat: now - 60,
        exp: now + 540,
        iss: state.config.github.client_id.clone(),
    };
    let key = EncodingKey::from_rsa_pem(state.config.github.app_private_key.as_bytes())
        .map_err(|_| AppError::internal("invalid GitHub App private key"))?;
    encode(&Header::new(Algorithm::RS256), &claims, &key)
        .map_err(|_| AppError::internal("failed to sign App JWT"))
}

//! GitHub App integration: OAuth login identity, installation linkage, and
//! short-lived installation access tokens (AC4.1). The single FeatureDoc App
//! provides both the user-authorization (login) and the installation token
//! (repo access).
//!
//! All outbound calls go through the `GitHubApi` trait so CI and the kind e2e
//! can run a deterministic in-process double (`MockGitHub`) while staging uses
//! `RealGitHub`. Long-lived user secrets are never stored: the installation
//! access token is minted on demand and discarded (AC4.1, AC4.3).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::GitHubConfig;
use crate::crypto::SecretString;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct GitHubUser {
    pub github_id: i64,
    pub login: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstallationInfo {
    pub account_login: String,
    pub repo_count: i64,
    /// Minimal permission set, surfaced to the user before/after install.
    pub permissions: Vec<String>,
}

/// A short-lived installation access token. Holds a `SecretString` so it can
/// never be logged, and is dropped at the end of the using scope.
pub struct InstallationToken {
    pub token: SecretString,
    pub expires_at: String,
}

#[async_trait]
pub trait GitHubApi: Send + Sync {
    /// Exchange an OAuth `code` for a user access token.
    async fn exchange_code(&self, code: &str) -> Result<SecretString, AppError>;
    /// Identify the user behind a user access token.
    async fn get_user(&self, user_token: &SecretString) -> Result<GitHubUser, AppError>;
    /// Describe an installation: account, repo count, permissions.
    async fn get_installation(&self, installation_id: i64) -> Result<InstallationInfo, AppError>;
    /// Mint a short-lived installation access token (on demand, not stored).
    async fn installation_token(
        &self,
        installation_id: i64,
    ) -> Result<InstallationToken, AppError>;
}

pub type SharedGitHub = Arc<dyn GitHubApi>;

/// The minimum permissions the App requests, shown to the user before install
/// (test scenario 1). Mirrors the mockup tags.
pub const REQUESTED_PERMISSIONS: [&str; 2] = ["contents:read", "metadata:read"];

// ----------------------------------------------------------------------------
// Mock implementation — deterministic, no network. Used by CI/e2e.
// ----------------------------------------------------------------------------

pub struct MockGitHub;

#[async_trait]
impl GitHubApi for MockGitHub {
    async fn exchange_code(&self, code: &str) -> Result<SecretString, AppError> {
        if code.is_empty() {
            return Err(AppError::BadRequest("missing code".into()));
        }
        // Encode the code into the fake token so different codes map to
        // different users deterministically.
        Ok(SecretString::new(format!("mock-user-token-{code}")))
    }

    async fn get_user(&self, user_token: &SecretString) -> Result<GitHubUser, AppError> {
        // Derive a stable identity from the token suffix so the same code
        // always resolves to the same user.
        let suffix = user_token
            .expose()
            .strip_prefix("mock-user-token-")
            .unwrap_or("anon");
        let github_id = stable_id(suffix);
        Ok(GitHubUser {
            github_id,
            login: format!("octo-{suffix}"),
            avatar_url: Some("https://avatars.githubusercontent.com/u/0".into()),
        })
    }

    async fn get_installation(&self, _installation_id: i64) -> Result<InstallationInfo, AppError> {
        Ok(InstallationInfo {
            account_login: "octo-org".into(),
            repo_count: 3,
            permissions: REQUESTED_PERMISSIONS.iter().map(|s| s.to_string()).collect(),
        })
    }

    async fn installation_token(
        &self,
        installation_id: i64,
    ) -> Result<InstallationToken, AppError> {
        Ok(InstallationToken {
            token: SecretString::new(format!("ghs_mock_{installation_id}")),
            expires_at: "2099-01-01T00:00:00Z".into(),
        })
    }
}

fn stable_id(s: &str) -> i64 {
    // Tiny deterministic hash → positive i64.
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    (h >> 1) as i64
}

// ----------------------------------------------------------------------------
// Real implementation — talks to api.github.com / github.com.
// ----------------------------------------------------------------------------

pub struct RealGitHub {
    cfg: GitHubConfig,
    http: reqwest::Client,
}

impl RealGitHub {
    pub fn new(cfg: GitHubConfig, http: reqwest::Client) -> Self {
        RealGitHub { cfg, http }
    }

    /// Sign a short-lived App JWT (RS256) for app-authenticated endpoints.
    fn app_jwt(&self) -> Result<String, AppError> {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
        #[derive(Serialize)]
        struct Claims {
            iat: i64,
            exp: i64,
            iss: String,
        }
        let now = crate::db::now();
        let claims = Claims {
            iat: now - 60,
            exp: now + 9 * 60,
            iss: self.cfg.app_id.clone(),
        };
        let key = EncodingKey::from_rsa_pem(self.cfg.private_key_pem.expose().as_bytes())
            .map_err(|e| {
                tracing::error!(error = %e, "invalid App private key");
                AppError::Internal
            })?;
        encode(&Header::new(Algorithm::RS256), &claims, &key).map_err(|e| {
            tracing::error!(error = %e, "failed to sign App JWT");
            AppError::Internal
        })
    }
}

#[async_trait]
impl GitHubApi for RealGitHub {
    async fn exchange_code(&self, code: &str) -> Result<SecretString, AppError> {
        #[derive(Deserialize)]
        struct TokenResp {
            access_token: Option<String>,
        }
        let resp = self
            .http
            .post(format!("{}/login/oauth/access_token", self.cfg.web_base))
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.cfg.client_id.as_str()),
                ("client_secret", self.cfg.client_secret.expose()),
                ("code", code),
            ])
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "github token exchange failed");
                AppError::ProviderRejected
            })?;
        let parsed: TokenResp = resp.json().await.map_err(|_| AppError::ProviderRejected)?;
        parsed
            .access_token
            .map(SecretString::new)
            .ok_or(AppError::ProviderRejected)
    }

    async fn get_user(&self, user_token: &SecretString) -> Result<GitHubUser, AppError> {
        #[derive(Deserialize)]
        struct UserResp {
            id: i64,
            login: String,
            avatar_url: Option<String>,
        }
        let resp = self
            .http
            .get(format!("{}/user", self.cfg.api_base))
            .header("Authorization", format!("Bearer {}", user_token.expose()))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "featuredoc")
            .send()
            .await
            .map_err(|_| AppError::ProviderRejected)?;
        let u: UserResp = resp.json().await.map_err(|_| AppError::ProviderRejected)?;
        Ok(GitHubUser {
            github_id: u.id,
            login: u.login,
            avatar_url: u.avatar_url,
        })
    }

    async fn get_installation(&self, installation_id: i64) -> Result<InstallationInfo, AppError> {
        #[derive(Deserialize)]
        struct Account {
            login: String,
        }
        #[derive(Deserialize)]
        struct InstResp {
            account: Account,
            #[serde(default)]
            repository_selection: String,
            #[serde(default)]
            permissions: std::collections::BTreeMap<String, String>,
        }
        let jwt = self.app_jwt()?;
        let resp = self
            .http
            .get(format!("{}/app/installations/{installation_id}", self.cfg.api_base))
            .header("Authorization", format!("Bearer {jwt}"))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "featuredoc")
            .send()
            .await
            .map_err(|_| AppError::ProviderRejected)?;
        let inst: InstResp = resp.json().await.map_err(|_| AppError::ProviderRejected)?;
        let perms = inst
            .permissions
            .into_iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect();
        // Repo count for "selected" installs requires a second call; the
        // connection view fetches it lazily. Default to 0 when unknown.
        let repo_count = if inst.repository_selection == "all" { -1 } else { 0 };
        Ok(InstallationInfo {
            account_login: inst.account.login,
            repo_count,
            permissions: perms,
        })
    }

    async fn installation_token(
        &self,
        installation_id: i64,
    ) -> Result<InstallationToken, AppError> {
        #[derive(Deserialize)]
        struct TokResp {
            token: String,
            expires_at: String,
        }
        let jwt = self.app_jwt()?;
        let resp = self
            .http
            .post(format!(
                "{}/app/installations/{installation_id}/access_tokens",
                self.cfg.api_base
            ))
            .header("Authorization", format!("Bearer {jwt}"))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "featuredoc")
            .send()
            .await
            .map_err(|_| AppError::ProviderRejected)?;
        let t: TokResp = resp.json().await.map_err(|_| AppError::ProviderRejected)?;
        Ok(InstallationToken {
            token: SecretString::new(t.token),
            expires_at: t.expires_at,
        })
    }
}

pub fn build(cfg: &GitHubConfig, http: reqwest::Client) -> SharedGitHub {
    match cfg.mode {
        crate::config::Mode::Mock => Arc::new(MockGitHub),
        crate::config::Mode::Real => Arc::new(RealGitHub::new(cfg.clone(), http)),
    }
}

// ----------------------------------------------------------------------------
// HTTP handlers (AC4.1). All scoped to the session user.
// ----------------------------------------------------------------------------

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde_json::json;

use crate::auth::CurrentUser;
use crate::config::Mode;
use crate::state::AppState;

/// GET /api/github/install-url — where to send the user to install the App,
/// along with the minimum permissions we will request (shown before install,
/// test scenario 1).
pub async fn install_url(
    State(app): State<AppState>,
    _user: CurrentUser,
) -> Json<serde_json::Value> {
    let url = match app.config.github.mode {
        // No real install page in mock mode: loop back to our setup callback.
        Mode::Mock => format!(
            "{}/api/github/setup?installation_id=42424242",
            app.config.base_url
        ),
        Mode::Real => format!(
            "{}/apps/{}/installations/new",
            app.config.github.web_base, app.config.github.app_slug
        ),
    };
    Json(json!({
        "url": url,
        "permissions": REQUESTED_PERMISSIONS,
    }))
}

#[derive(serde::Deserialize)]
pub struct SetupQuery {
    installation_id: i64,
}

/// GET /api/github/setup — GitHub's post-install redirect lands here. Bind the
/// installation to the session user, then return to the app.
pub async fn setup(
    State(app): State<AppState>,
    user: CurrentUser,
    Query(q): Query<SetupQuery>,
) -> Result<Response, AppError> {
    let info = app.github.get_installation(q.installation_id).await?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO installations \
         (id, user_id, installation_id, account_login, repo_count, permissions, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(user_id, installation_id) DO UPDATE SET \
           account_login = excluded.account_login, \
           repo_count = excluded.repo_count, \
           permissions = excluded.permissions",
    )
    .bind(id)
    .bind(&user.id)
    .bind(q.installation_id)
    .bind(&info.account_login)
    .bind(info.repo_count)
    .bind(info.permissions.join(","))
    .bind(crate::db::now())
    .execute(&app.db)
    .await?;

    crate::audit::record(
        &app.db,
        Some(&user.id),
        "github.install",
        &format!("installation_id={} account={}", q.installation_id, info.account_login),
    )
    .await;

    Ok(Redirect::to(&format!("{}/", app.config.base_url)).into_response())
}

/// GET /api/github/connection — current install status for the session user.
pub async fn connection(
    State(app): State<AppState>,
    user: CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let row: Option<(i64, String, i64, String)> = sqlx::query_as(
        "SELECT installation_id, account_login, repo_count, permissions \
         FROM installations WHERE user_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&user.id)
    .fetch_optional(&app.db)
    .await?;

    match row {
        Some((installation_id, account_login, repo_count, permissions)) => Ok(Json(json!({
            "installed": true,
            "installation_id": installation_id,
            "account": account_login,
            "repo_count": repo_count,
            "permissions": permissions.split(',').filter(|s| !s.is_empty()).collect::<Vec<_>>(),
        }))),
        None => Ok(Json(json!({
            "installed": false,
            "permissions": REQUESTED_PERMISSIONS,
        }))),
    }
}

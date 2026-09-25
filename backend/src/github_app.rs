//! GitHub App credentials and installation metadata.
//!
//! Upstream failures are mapped to fixed strings rather than interpolated: the App
//! JWT, the installation token, and the private key must never reach a log line.

use serde::{Deserialize, Serialize};

use crate::config::Mode;
use crate::error::AppError;
use crate::models::User;
use crate::state::AppState;
use crate::util::{now_unix, rfc3339_to_unix};

pub const REQUESTED_PERMISSIONS: &[&str] = &["contents:read", "metadata:read"];

pub struct InstallationToken {
    pub token: String,
    pub expires_at: i64,
}

pub struct InstallationInfo {
    pub account_login: Option<String>,
    pub account_type: Option<String>,
    pub repository_selection: Option<String>,
}

pub async fn mint_installation_token(
    state: &AppState,
    installation_id: i64,
) -> Result<InstallationToken, AppError> {
    match state.config.doubles.github_app {
        // mock-exception: EXT-02 — App JWT 서명·실제 설치 토큰 발급은 실제 App 개인키가 필요
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

pub async fn fetch_installation(
    state: &AppState,
    installation_id: i64,
) -> Result<InstallationInfo, AppError> {
    match state.config.doubles.github_app {
        // mock-exception: EXT-02 — 설치 메타 조회는 실제 App 설치를 요구
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

/// `size_kb` is GitHub's reported size: a pre-flight estimate, never an access decision.
pub struct RepoRef {
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub default_branch: String,
    pub size_kb: i64,
}

pub async fn list_repositories(
    state: &AppState,
    installation_id: i64,
) -> Result<Vec<RepoRef>, AppError> {
    match state.config.doubles.github_app {
        // mock-exception: EXT-02 — 접근 가능 저장소 목록은 실제 설치 토큰이 필요
        Mode::Stub => Ok(stub_repositories()),
        Mode::Real => {
            let token = mint_installation_token(state, installation_id).await?;
            let mut out: Vec<RepoRef> = Vec::new();
            let mut page = 1;
            loop {
                let url = format!(
                    "{}/installation/repositories?per_page=100&page={page}",
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
                    .map_err(|_| AppError::internal("github repositories request failed"))?;
                if !resp.status().is_success() {
                    return Err(AppError::internal("github repositories rejected"));
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
                    #[serde(default)]
                    size: i64,
                }
                #[derive(Deserialize)]
                struct R {
                    total_count: i64,
                    repositories: Vec<Repo>,
                }
                let r: R = resp
                    .json()
                    .await
                    .map_err(|_| AppError::internal("github repositories: malformed response"))?;
                let got = r.repositories.len();
                for repo in r.repositories {
                    let owner = repo.owner.login;
                    let full_name = format!("{owner}/{}", repo.name);
                    out.push(RepoRef {
                        default_branch: repo.default_branch.unwrap_or_else(|| "main".into()),
                        size_kb: repo.size,
                        owner,
                        name: repo.name,
                        full_name,
                    });
                }
                if got == 0 || out.len() as i64 >= r.total_count {
                    break;
                }
                page += 1;
            }
            Ok(out)
        }
    }
}

/// Narrows what the stub installation grants, so a test can take repository access
/// away the way a user does on GitHub (AC4.1's "해제·범위 축소").
///
/// Unset is the full stub installation, so a deployment that says nothing keeps the
/// three repositories every other spec relies on. The value is a comma-separated
/// list of repository names; an empty value grants none, which is what an
/// uninstall looks like from here.
const STUB_ACCESS: &str = "FEATUREDOC_STUB_REPO_ACCESS";

fn stub_granted_names() -> Option<Vec<String>> {
    let raw = std::env::var(STUB_ACCESS).ok()?;
    Some(
        raw.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

/// Three repos — the count must stay in step with [`repository_count`]'s stub arm.
/// Both read [`stub_granted_names`] for that reason: one narrowing, two answers
/// that cannot drift apart.
fn stub_repositories() -> Vec<RepoRef> {
    let granted = stub_granted_names();
    [
        ("payments-api", "main", 2300),
        ("checkout-web", "main", 5100),
        ("notif-worker", "main", 800),
    ]
    .into_iter()
    .filter(|(name, _, _)| match &granted {
        None => true,
        Some(names) => names.iter().any(|n| n == name),
    })
    .map(|(name, branch, size_kb)| RepoRef {
        owner: "stub-account".to_string(),
        name: name.to_string(),
        full_name: format!("stub-account/{name}"),
        default_branch: branch.to_string(),
        size_kb,
    })
    .collect()
}

pub async fn repository_count(state: &AppState, installation_id: i64) -> Option<i64> {
    match state.config.doubles.github_app {
        // mock-exception: EXT-02 — 저장소 개수 조회는 실제 설치 토큰이 필요
        Mode::Stub => Some(stub_repositories().len() as i64),
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

pub struct UserInstallation {
    pub installation_id: i64,
    pub account_login: Option<String>,
    pub account_type: Option<String>,
    pub repository_selection: Option<String>,
}

/// The endpoint is already scoped to the App that issued the token, so there is
/// nothing to filter by app id. Stub mode has no GitHub to ask and reports none —
/// the stub install flow writes its row through the Setup URL instead.
pub async fn list_user_installations(
    state: &AppState,
    user_id: &str,
) -> Result<Vec<UserInstallation>, AppError> {
    // mock-exception: EXT-02 — 사용자별 설치 목록은 실제 사용자 토큰으로 실 계정의 설치를 읽어야 한다
    if state.config.doubles.github_app == Mode::Stub {
        return Ok(Vec::new());
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
    struct Account {
        login: Option<String>,
        #[serde(rename = "type")]
        kind: Option<String>,
    }
    #[derive(Deserialize)]
    struct Inst {
        id: i64,
        account: Option<Account>,
        repository_selection: Option<String>,
    }
    #[derive(Deserialize)]
    struct R {
        installations: Vec<Inst>,
    }
    let body: R = resp
        .json()
        .await
        .map_err(|_| AppError::internal("github installations: malformed response"))?;

    Ok(body
        .installations
        .into_iter()
        .map(|i| UserInstallation {
            installation_id: i.id,
            account_login: i.account.as_ref().and_then(|a| a.login.clone()),
            account_type: i.account.and_then(|a| a.kind),
            repository_selection: i.repository_selection,
        })
        .collect())
}

/// The Setup URL's `installation_id` is attacker-controlled (GitHub does not sign
/// it), so it must be confirmed against the user's own installations. The stub has
/// no GitHub to ask, so it confirms against the only id its own install flow ever
/// hands out — [`stub_installation_id`] of this user.
pub async fn verify_user_owns_installation(
    state: &AppState,
    user: &User,
    installation_id: i64,
) -> Result<(), AppError> {
    // mock-exception: EXT-02 — 설치 소유 확인은 실제 사용자 토큰으로 실 계정의 설치 목록을 읽어야 한다
    if state.config.doubles.github_app == Mode::Stub {
        return if installation_id == stub_installation_id(user.github_id) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        };
    }

    let installations = list_user_installations(state, &user.id).await?;
    if installations
        .iter()
        .any(|i| i.installation_id == installation_id)
    {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// The single installation id the stub install flow hands out, derived from the
/// user so two stub users never collide. Lives here rather than beside the install
/// route because [`verify_user_owns_installation`] has to check against the very
/// value that flow issued.
pub fn stub_installation_id(github_id: i64) -> i64 {
    10_000 + github_id.rem_euclid(90_000)
}

/// `iss` is the client ID, not the numeric App ID: both authenticate today, but
/// GitHub ties forward compatibility to the client ID (guidance as of 2024-05).
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

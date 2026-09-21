//! Process configuration loaded from the environment.
//!
//! Secrets (KEK, GitHub App private key, client secret) live here but are never
//! derived into `Debug`/`Display` — see the deliberate absence of `#[derive(Debug)]`
//! on [`Config`] and [`GithubConfig`] (AC4.3: credentials must not reach logs).

use std::sync::Arc;

use sha2::{Digest, Sha256};

/// Selects a real external integration vs. its deterministic in-process test
/// double — for **one** boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Real,
    Stub,
}

/// Which external boundaries this process answers with a test double.
///
/// Selected *per boundary* rather than by one process-wide switch, so nothing can
/// turn on a double the running process does not itself own — the analysis worker
/// holds its own set. Each is opt-in; the default everywhere is the real
/// integration.
#[derive(Clone, Copy, Debug)]
pub struct Doubles {
    pub github_auth: Mode,
    pub github_app: Mode,
    pub llm_key: Mode,
    /// 모델 호출 자체. 워커도 같은 이름의 변수를 자기 프로세스에서 읽는다 — 값이
    /// 아니라 **이름**을 공유하는 것이고, 한쪽만 스텁인 배포가 가능하다.
    pub llm: Mode,
    /// 저장소 트리 스캔. 1단계는 워커의 것이지만, 사람이 feature 를 직접 더할 때의
    /// 근거 찾기는 이 프로세스가 같은 스캔을 한 번 더 한다 — `llm` 과 같은 방식으로
    /// 워커와 이름만 공유한다.
    pub repo_scan: Mode,
}

impl Mode {
    /// Reads one boundary's setting from its own environment variable.
    ///
    /// An unset variable is the real integration, so a deployment that says nothing
    /// gets nothing stubbed. Deliberately not one shared name for every boundary: a
    /// value read here can only ever reach the single boundary its caller names.
    pub fn from_env(key: &str) -> Self {
        if env_or(key, "").trim().eq_ignore_ascii_case("stub") {
            Self::Stub
        } else {
            Self::Real
        }
    }
}

impl Doubles {
    /// Every boundary at the same setting — for tests, which want one hermetic state
    /// rather than a per-boundary mix.
    pub fn all(mode: Mode) -> Self {
        Self {
            github_auth: mode,
            github_app: mode,
            llm_key: mode,
            llm: mode,
            repo_scan: mode,
        }
    }

    fn from_env() -> Self {
        Self {
            github_auth: Mode::from_env("FEATUREDOC_DOUBLE_GITHUB_AUTH"),
            github_app: Mode::from_env("FEATUREDOC_DOUBLE_GITHUB_APP"),
            llm_key: Mode::from_env("FEATUREDOC_DOUBLE_LLM_KEY"),
            llm: Mode::from_env("FEATUREDOC_DOUBLE_LLM"),
            repo_scan: Mode::from_env("FEATUREDOC_DOUBLE_REPO_SCAN"),
        }
    }
}

/// Single GitHub App used for both user-authorization (login) and installation
/// (repository access).
#[derive(Clone)]
pub struct GithubConfig {
    /// PEM-encoded RSA private key used to mint short-lived App JWTs. Secret.
    pub app_private_key: String,
    /// Public client ID — the OAuth authorize client_id and the App JWT `iss`.
    pub client_id: String,
    /// OAuth client secret for the user-authorization code exchange. Secret.
    pub client_secret: String,
    pub app_slug: String,
    pub api_base: String,
    pub web_base: String,
}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub base_url: String,
    /// Origin the GitHub App's callback URL is registered at. Normally the same
    /// as [`Self::base_url`].
    ///
    /// A GitHub App accepts no wildcard in its callback URL, so per-pull-request
    /// preview hosts cannot each register their own. Previews instead point this
    /// at the production origin and rely on a redirect proxy there to bounce the
    /// code back — see [`Self::preview_id`].
    pub oauth_redirect_base_url: String,
    /// Set only on preview deployments: the pull-request number this instance
    /// serves. `Some` means callbacks land on [`Self::oauth_redirect_base_url`]
    /// and are bounced here, so the OAuth `state` carries a `pr-<id>~` prefix for
    /// the proxy to route on. Digits only (see [`sanitize_preview_id`]).
    pub preview_id: Option<String>,
    pub static_dir: String,
    /// 32-byte key-encryption-key that wraps per-record DEKs. Secret.
    pub kek: [u8; 32],
    pub doubles: Doubles,
    pub github: GithubConfig,
    /// Emit the session cookie with `Secure` (true behind HTTPS). Off for local/e2e http.
    pub cookie_secure: bool,
    /// Shared secret the analysis worker presents on `/internal/*`. Secret.
    /// Empty means "this deployment has no worker" — the internal routes then
    /// reject every caller rather than falling open.
    pub worker_token: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Arc<Self>> {
        let doubles = Doubles::from_env();

        let kek_secret = std::env::var("FEATUREDOC_KEK").ok();
        if kek_secret.is_none() {
            tracing::warn!(
                "FEATUREDOC_KEK is not set — deriving an insecure development KEK; \
                 set FEATUREDOC_KEK in any real deployment"
            );
        }
        let kek = derive_kek(kek_secret.as_deref().unwrap_or("insecure-development-kek"));

        let github = GithubConfig {
            app_private_key: read_secret("GITHUB_APP_PRIVATE_KEY", "GITHUB_APP_PRIVATE_KEY_FILE"),
            client_id: env_or("GITHUB_CLIENT_ID", ""),
            client_secret: env_or("GITHUB_CLIENT_SECRET", ""),
            app_slug: env_or("GITHUB_APP_SLUG", "featuredoc"),
            api_base: trim_trailing_slash(&env_or("GITHUB_API_BASE", "https://api.github.com")),
            web_base: trim_trailing_slash(&env_or("GITHUB_WEB_BASE", "https://github.com")),
        };

        let base_url = trim_trailing_slash(&env_or("BASE_URL", "http://localhost:8080"));
        let oauth_redirect_base_url = match env_or("OAUTH_REDIRECT_BASE_URL", "") {
            v if v.trim().is_empty() => base_url.clone(),
            v => trim_trailing_slash(v.trim()),
        };

        let preview_id = std::env::var("FEATUREDOC_PREVIEW_ID")
            .ok()
            .and_then(|v| sanitize_preview_id(&v));

        Ok(Arc::new(Self {
            database_url: env_or("DATABASE_URL", "sqlite://featuredoc.db?mode=rwc"),
            base_url,
            oauth_redirect_base_url,
            preview_id,
            static_dir: env_or("STATIC_DIR", "dist"),
            kek,
            doubles,
            github,
            cookie_secure: env_or("COOKIE_SECURE", "").eq_ignore_ascii_case("true"),
            worker_token: env_or("FEATUREDOC_WORKER_TOKEN", ""),
        }))
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Reads a secret from `key`, falling back to the contents of the file named by
/// `file_key` (k8s commonly projects secrets as files). Empty if neither is set.
fn read_secret(key: &str, file_key: &str) -> String {
    if let Ok(v) = std::env::var(key) {
        return v;
    }
    if let Ok(path) = std::env::var(file_key) {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            return contents;
        }
    }
    String::new()
}

fn trim_trailing_slash(s: &str) -> String {
    s.trim_end_matches('/').to_string()
}

/// Accepts a preview id only if it is a short run of ASCII digits, and drops it
/// otherwise rather than escaping it.
///
/// The id is interpolated into the OAuth `state`, so a `~`, `&` or `#` here would
/// let it forge extra query parameters on the callback URL. Nothing is lost by
/// being strict: the redirect proxy rebuilds the preview host from this id with
/// the same `[0-9]{1,6}` shape, so a value this function would have to escape
/// could never have routed anywhere anyway.
fn sanitize_preview_id(raw: &str) -> Option<String> {
    let v = raw.trim();
    if v.is_empty() {
        return None;
    }
    if v.len() <= 6 && v.bytes().all(|b| b.is_ascii_digit()) {
        Some(v.to_string())
    } else {
        tracing::warn!(
            "FEATUREDOC_PREVIEW_ID is not a 1-6 digit pull-request number; \
             ignoring it and signing in as a non-preview deployment"
        );
        None
    }
}

impl std::fmt::Debug for GithubConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GithubConfig")
            .field("app_private_key", &"[REDACTED]")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("app_slug", &self.app_slug)
            .field("api_base", &self.api_base)
            .field("web_base", &self.web_base)
            .finish()
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &self.database_url)
            .field("base_url", &self.base_url)
            .field("oauth_redirect_base_url", &self.oauth_redirect_base_url)
            .field("preview_id", &self.preview_id)
            .field("static_dir", &self.static_dir)
            .field("kek", &"[REDACTED]")
            .field("doubles", &self.doubles)
            .field("github", &self.github)
            .field("cookie_secure", &self.cookie_secure)
            .field("worker_token", &"[REDACTED]")
            .finish()
    }
}

fn derive_kek(secret: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"featuredoc-kek-v1");
    h.update(secret.as_bytes());
    let out = h.finalize();
    let mut k = [0u8; 32];
    k.copy_from_slice(&out);
    k
}

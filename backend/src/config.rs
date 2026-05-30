//! Process configuration loaded from the environment.
//!
//! Secrets (KEK, GitHub App private key, client secret) live here but are never
//! derived into `Debug`/`Display` — see the deliberate absence of `#[derive(Debug)]`
//! on [`Config`] and [`GithubConfig`] (AC4.3: credentials must not reach logs).

use std::sync::Arc;

use sha2::{Digest, Sha256};

/// Selects real external integrations vs. deterministic in-process test doubles.
///
/// `real` (default) talks to GitHub and the LLM providers over the network.
/// `stub` short-circuits those boundaries with canned, deterministic behaviour so
/// the kind-based e2e and unit tests stay hermetic (plan: "테스트 더블로 모킹").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Real,
    Stub,
}

/// Single GitHub App used for both user-authorization (login) and installation
/// (repository access). See plan: "로그인·설치를 단일 GitHub App으로 통합".
#[derive(Clone)]
pub struct GithubConfig {
    pub app_id: String,
    /// PEM-encoded RSA private key used to mint short-lived App JWTs. Secret.
    pub app_private_key: String,
    pub client_id: String,
    /// OAuth client secret for the user-authorization code exchange. Secret.
    pub client_secret: String,
    pub app_slug: String,
    /// API origin, e.g. `https://api.github.com`. Overridable for tests.
    pub api_base: String,
    /// Web origin, e.g. `https://github.com`. Overridable for tests.
    pub web_base: String,
}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    /// Public origin this service is reached at; used to build OAuth redirect URIs.
    pub base_url: String,
    pub static_dir: String,
    /// 32-byte key-encryption-key that wraps per-record DEKs. Secret.
    pub kek: [u8; 32],
    pub mode: Mode,
    pub github: GithubConfig,
    /// Emit the session cookie with `Secure` (true behind HTTPS). Off for local/e2e http.
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Arc<Self>> {
        let mode = match env_or("FEATUREDOC_MODE", "real").to_ascii_lowercase().as_str() {
            "stub" => Mode::Stub,
            _ => Mode::Real,
        };

        let kek_secret = std::env::var("FEATUREDOC_KEK").ok();
        if kek_secret.is_none() {
            tracing::warn!(
                "FEATUREDOC_KEK is not set — deriving an insecure development KEK; \
                 set FEATUREDOC_KEK in any real deployment"
            );
        }
        let kek = derive_kek(kek_secret.as_deref().unwrap_or("insecure-development-kek"));

        let github = GithubConfig {
            app_id: env_or("GITHUB_APP_ID", ""),
            app_private_key: read_secret("GITHUB_APP_PRIVATE_KEY", "GITHUB_APP_PRIVATE_KEY_FILE"),
            client_id: env_or("GITHUB_CLIENT_ID", ""),
            client_secret: env_or("GITHUB_CLIENT_SECRET", ""),
            app_slug: env_or("GITHUB_APP_SLUG", "featuredoc"),
            api_base: trim_trailing_slash(&env_or("GITHUB_API_BASE", "https://api.github.com")),
            web_base: trim_trailing_slash(&env_or("GITHUB_WEB_BASE", "https://github.com")),
        };

        Ok(Arc::new(Self {
            database_url: env_or("DATABASE_URL", "sqlite://featuredoc.db?mode=rwc"),
            base_url: trim_trailing_slash(&env_or("BASE_URL", "http://localhost:8080")),
            static_dir: env_or("STATIC_DIR", "dist"),
            kek,
            mode,
            github,
            cookie_secure: env_or("COOKIE_SECURE", "").eq_ignore_ascii_case("true"),
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

// Redacting Debug: secrets (KEK, App private key, client secret) render as
// [REDACTED] so a `{:?}` of the config can never leak them (AC4.3). app_id and
// client_id are public identifiers and are shown.
impl std::fmt::Debug for GithubConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GithubConfig")
            .field("app_id", &self.app_id)
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
            .field("static_dir", &self.static_dir)
            .field("kek", &"[REDACTED]")
            .field("mode", &self.mode)
            .field("github", &self.github)
            .field("cookie_secure", &self.cookie_secure)
            .finish()
    }
}

/// Derives a stable 32-byte KEK from an arbitrary secret string via domain-separated SHA-256.
fn derive_kek(secret: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"featuredoc-kek-v1");
    h.update(secret.as_bytes());
    let out = h.finalize();
    let mut k = [0u8; 32];
    k.copy_from_slice(&out);
    k
}

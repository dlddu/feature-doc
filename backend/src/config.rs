//! Runtime configuration, loaded once from the environment at startup.
//!
//! Secrets (KEK, GitHub App private key, OAuth client secret) arrive via a
//! k8s Secret mounted as env vars (see deploy/k8s). Nothing here is hard-coded.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

use crate::crypto::{Kek, SecretString};

/// Selects whether outbound GitHub / LLM calls hit the real services or a
/// deterministic in-process double. CI and the kind e2e run in `Mock`; staging
/// and production run `Real` with injected credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Real,
    Mock,
}

impl Mode {
    fn from_env(key: &str) -> Self {
        match std::env::var(key).as_deref() {
            Ok("real") | Ok("Real") | Ok("REAL") => Mode::Real,
            _ => Mode::Mock,
        }
    }
}

#[derive(Clone)]
pub struct GitHubConfig {
    pub mode: Mode,
    /// GitHub App numeric id (for JWT `iss`).
    pub app_id: String,
    /// GitHub App user-authorization OAuth client id.
    pub client_id: String,
    pub client_secret: SecretString,
    /// PEM-encoded RSA private key used to sign the App JWT.
    pub private_key_pem: SecretString,
    /// Public slug used to build the install URL, e.g. `featuredoc`.
    pub app_slug: String,
    /// Base of the GitHub web (oauth) endpoints — overridable for tests.
    pub web_base: String,
    /// Base of the GitHub REST API — overridable for tests.
    pub api_base: String,
}

#[derive(Clone)]
pub struct Config {
    /// Public base URL of this service; OAuth/setup redirects come back here.
    pub base_url: String,
    /// Where the SQLite file lives (a PVC path in k8s).
    pub database_url: String,
    /// Set `Secure` on the session cookie. Off for plain-http local/e2e.
    pub cookie_secure: bool,
    pub github: GitHubConfig,
    /// LLM key validation mode (mirrors GitHub mode).
    pub llm_mode: Mode,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            base_url: env_or("BASE_URL", "http://localhost:8080"),
            database_url: env_or("DATABASE_URL", "sqlite:featuredoc.db?mode=rwc"),
            cookie_secure: matches!(
                std::env::var("COOKIE_SECURE").as_deref(),
                Ok("1") | Ok("true")
            ),
            github: GitHubConfig {
                mode: Mode::from_env("GITHUB_MODE"),
                app_id: env_or("GITHUB_APP_ID", "000000"),
                client_id: env_or("GITHUB_CLIENT_ID", "Iv1.mockclientid"),
                client_secret: SecretString::new(env_or("GITHUB_CLIENT_SECRET", "mock-secret")),
                private_key_pem: SecretString::new(env_or("GITHUB_APP_PRIVATE_KEY", "")),
                app_slug: env_or("GITHUB_APP_SLUG", "featuredoc"),
                web_base: env_or("GITHUB_WEB_BASE", "https://github.com"),
                api_base: env_or("GITHUB_API_BASE", "https://api.github.com"),
            },
            llm_mode: Mode::from_env("LLM_MODE"),
        }
    }

    /// A deterministic mock-mode config for tests and local boot.
    pub fn for_test() -> Self {
        Config {
            base_url: "http://localhost:8080".into(),
            database_url: "sqlite::memory:".into(),
            cookie_secure: false,
            github: GitHubConfig {
                mode: Mode::Mock,
                app_id: "000000".into(),
                client_id: "Iv1.mockclientid".into(),
                client_secret: SecretString::new("mock-secret"),
                private_key_pem: SecretString::new(""),
                app_slug: "featuredoc".into(),
                web_base: "https://github.com".into(),
                api_base: "https://api.github.com".into(),
            },
            llm_mode: Mode::Mock,
        }
    }

    /// Load and decode the 32-byte KEK. Generates an ephemeral key (with a
    /// warning) when unset so local `cargo run` works; production always
    /// injects `CREDENTIALS_KEK`.
    pub fn load_kek() -> Kek {
        match std::env::var("CREDENTIALS_KEK") {
            Ok(b64) => {
                let bytes = B64
                    .decode(b64.trim())
                    .expect("CREDENTIALS_KEK must be valid base64");
                let arr: [u8; 32] = bytes
                    .try_into()
                    .expect("CREDENTIALS_KEK must decode to exactly 32 bytes");
                Kek::from_bytes(arr)
            }
            Err(_) => {
                tracing::warn!(
                    "CREDENTIALS_KEK unset — generating an ephemeral key; stored \
                     credentials will not survive a restart"
                );
                let mut k = [0u8; 32];
                rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut k);
                Kek::from_bytes(k)
            }
        }
    }
}

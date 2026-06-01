//! Shared test scaffolding: a stub-mode AppState backed by a throwaway SQLite file.
//!
//! Each integration-test binary inlines this module but uses only the helpers it
//! needs, so allow the unused ones here rather than per call site.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use featuredoc::config::{Config, GithubConfig, Mode};
use featuredoc::db;
use featuredoc::state::AppState;

pub fn temp_db_url() -> (String, PathBuf) {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!(
        "featuredoc-test-{}-{}.db",
        std::process::id(),
        nanos
    ));
    (format!("sqlite://{}?mode=rwc", path.display()), path)
}

pub async fn stub_state() -> (AppState, PathBuf) {
    let (url, path) = temp_db_url();
    let pool = db::connect(&url).await.expect("connect + migrate");
    let config = Arc::new(Config {
        database_url: url,
        base_url: "http://localhost:8080".into(),
        static_dir: "dist".into(),
        kek: [9u8; 32],
        mode: Mode::Stub,
        github: GithubConfig {
            app_private_key: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            app_slug: "featuredoc".into(),
            api_base: "https://api.github.com".into(),
            oauth_base: "https://github.com".into(),
            web_base: "https://github.com".into(),
        },
        cookie_secure: false,
    });
    (
        AppState {
            db: pool,
            config,
            http: reqwest::Client::new(),
        },
        path,
    )
}

/// A throwaway RSA private key (PKCS#1 PEM) used only to sign App JWTs in tests.
/// The mock GitHub server does not verify the signature; the app only needs the
/// signing step to succeed.
pub const TEST_APP_PRIVATE_KEY: &str = include_str!("../fixtures/test_app_key.pem");

/// Keeps a spawned in-process mock GitHub server alive for a test's duration.
/// Bind it (e.g. `let (state, _mock, path) = real_state().await;`) — dropping it
/// shuts the server down.
pub struct MockGuard {
    pub base_url: String,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for MockGuard {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Spawns [`featuredoc::mock_github`] on an ephemeral loopback port.
pub async fn spawn_mock() -> MockGuard {
    use featuredoc::mock_github::{router, MockConfig};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock github");
    let addr = listener.local_addr().expect("mock github local_addr");
    let app = router(MockConfig {
        app_base_url: "http://localhost:8080".into(),
        default_login: "stub".into(),
    });
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    MockGuard {
        base_url: format!("http://{addr}"),
        handle,
    }
}

/// A real-mode AppState whose GitHub base URLs all point at a freshly-spawned mock
/// GitHub server. Use for the auth/login and GitHub App flows.
pub async fn real_state() -> (AppState, MockGuard, PathBuf) {
    let mock = spawn_mock().await;
    let (url, path) = temp_db_url();
    let pool = db::connect(&url).await.expect("connect + migrate");
    let config = Arc::new(Config {
        database_url: url,
        base_url: "http://localhost:8080".into(),
        static_dir: "dist".into(),
        kek: [9u8; 32],
        mode: Mode::Real,
        github: GithubConfig {
            app_private_key: TEST_APP_PRIVATE_KEY.to_string(),
            client_id: "test-client-id".into(),
            client_secret: "test-client-secret".into(),
            app_slug: "featuredoc".into(),
            api_base: mock.base_url.clone(),
            oauth_base: mock.base_url.clone(),
            web_base: mock.base_url.clone(),
        },
        cookie_secure: false,
    });
    (
        AppState {
            db: pool,
            config,
            http: reqwest::Client::new(),
        },
        mock,
        path,
    )
}

/// Pulls a cookie value out of a `Set-Cookie` header string.
pub fn cookie_value(set_cookie: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    set_cookie
        .split(';')
        .next()
        .and_then(|kv| kv.trim().strip_prefix(&prefix))
        .map(|s| s.to_string())
}

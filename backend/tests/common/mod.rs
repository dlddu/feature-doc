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
            app_id: String::new(),
            app_private_key: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            app_slug: "featuredoc".into(),
            api_base: "https://api.github.com".into(),
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

/// Pulls a cookie value out of a `Set-Cookie` header string.
pub fn cookie_value(set_cookie: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    set_cookie
        .split(';')
        .next()
        .and_then(|kv| kv.trim().strip_prefix(&prefix))
        .map(|s| s.to_string())
}

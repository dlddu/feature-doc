//! Shared test scaffolding: a stub-mode AppState backed by a throwaway SQLite file.
//!
//! Each integration-test binary inlines this module but uses only the helpers it
//! needs, so allow the unused ones here rather than per call site.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use featuredoc::config::{Config, Doubles, GithubConfig, Mode};
use featuredoc::db;
use featuredoc::state::AppState;

/// Distinguishes two calls that land inside the same clock tick.
///
/// `SystemTime::now().as_nanos()` is *not* unique here: the wall clock advances in
/// coarser steps than a nanosecond, so parallel tests drew the same value and shared
/// one database file. The counter makes the path unique by construction, not by luck.
static DB_SEQ: AtomicU64 = AtomicU64::new(0);

pub const WORKER_TOKEN: &str = "test-worker-token";

pub fn temp_db_url() -> (String, PathBuf) {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let seq = DB_SEQ.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "featuredoc-test-{}-{}-{}.db",
        std::process::id(),
        nanos,
        seq
    ));
    (format!("sqlite://{}?mode=rwc", path.display()), path)
}

pub async fn stub_state() -> (AppState, PathBuf) {
    state_with(Mode::Stub, "https://api.github.com").await
}

pub async fn real_state(api_base: &str) -> (AppState, PathBuf) {
    state_with(Mode::Real, api_base).await
}

async fn state_with(mode: Mode, api_base: &str) -> (AppState, PathBuf) {
    let (url, path) = temp_db_url();
    let pool = db::connect(&url).await.expect("connect + migrate");
    let config = Arc::new(Config {
        database_url: url,
        base_url: "http://localhost:8080".into(),
        oauth_redirect_base_url: "http://localhost:8080".into(),
        preview_id: None,
        static_dir: "dist".into(),
        kek: [9u8; 32],
        doubles: Doubles::all(mode),
        github: GithubConfig {
            app_private_key: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            app_slug: "featuredoc".into(),
            api_base: api_base.into(),
            web_base: "https://github.com".into(),
        },
        cookie_secure: false,
        worker_token: WORKER_TOKEN.into(),
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

pub fn cookie_value(set_cookie: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    set_cookie
        .split(';')
        .next()
        .and_then(|kv| kv.trim().strip_prefix(&prefix))
        .map(|s| s.to_string())
}

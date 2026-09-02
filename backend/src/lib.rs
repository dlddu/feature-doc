//! FeatureDoc backend library: configuration, persistence, and the HTTP router.
//!
//! The binary (`main.rs`) is a thin wrapper that loads [`config::Config`],
//! connects the database, and serves [`build_router`].

pub mod analysis;
pub mod audit;
pub mod auth;
pub mod config;
pub mod cookies;
pub mod crypto;
pub mod cross_cutting;
pub mod db;
pub mod discovery_strategy;
pub mod error;
pub mod feature_candidates;
pub mod github;
pub mod github_api;
pub mod github_app;
pub mod github_tokens;
pub mod installations;
pub mod llm;
pub mod llmkey;
pub mod models;
pub mod pipeline;
pub mod repo_scan;
pub mod session;
pub mod state;
pub mod users;
pub mod util;
pub mod worker_api;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

async fn hello() -> Json<Value> {
    Json(json!({ "message": "Hello from FeatureDoc backend" }))
}

/// Builds the full application router: the `/hello` probe, the (future) `/api/*`
/// surface, and a static-file fallback that serves the SPA shell.
pub fn build_router(state: AppState) -> Router {
    let dist = state.config.static_dir.clone();
    let index = format!("{dist}/index.html");
    let static_service = ServeDir::new(&dist).not_found_service(ServeFile::new(index));

    Router::new()
        .route("/hello", get(hello))
        .merge(auth::routes())
        .merge(github::routes())
        .merge(llmkey::routes())
        .merge(audit::routes())
        .merge(analysis::routes())
        .merge(worker_api::routes())
        .fallback_service(static_service)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Installs the tracing subscriber. `RUST_LOG` controls verbosity (default `info`).
///
/// ANSI colour is only enabled when stdout is an interactive terminal. Under
/// Kubernetes/Docker stdout is a pipe, so container logs stay free of escape
/// sequences (`\x1b[2m...`) that make `kubectl logs` and log shipping unreadable.
/// `NO_COLOR=1` forces it off even in a terminal.
pub fn init_tracing() {
    use std::io::IsTerminal;
    use tracing_subscriber::EnvFilter;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let ansi = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(ansi)
        .init();
}

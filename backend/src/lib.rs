//! FeatureDoc backend library: configuration, persistence, and the HTTP router.
//!
//! The binary (`main.rs`) is a thin wrapper that loads [`config::Config`],
//! connects the database, and serves [`build_router`].

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod github_api;
pub mod models;
pub mod session;
pub mod state;
pub mod users;
pub mod util;

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
        .fallback_service(static_service)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Installs the tracing subscriber. `RUST_LOG` controls verbosity (default `info`).
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

//! FeatureDoc backend library. The binary (`main.rs`) is a thin wrapper around
//! `router()`; integration tests mount the same router against an in-memory
//! database.

use axum::{
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

pub mod audit;
pub mod auth;
pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod github;
pub mod llm;
pub mod llmkey;
pub mod state;

use state::AppState;

pub async fn hello() -> Json<Value> {
    Json(json!({ "message": "Hello from FeatureDoc backend" }))
}

/// Build the full router. Shared by the binary and the integration tests.
pub fn router(app: AppState) -> Router {
    let dist = std::env::var("STATIC_DIR").unwrap_or_else(|_| "dist".to_string());
    let index = format!("{dist}/index.html");
    let static_service = ServeDir::new(&dist).not_found_service(ServeFile::new(index));

    let api = Router::new()
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/logout", post(auth::logout))
        .route("/me", get(auth::me))
        .route("/github/install-url", get(github::install_url))
        .route("/github/setup", get(github::setup))
        .route("/github/connection", get(github::connection))
        .route("/llm-keys", post(llmkey::register).get(llmkey::list))
        .route("/llm-keys/{id}", delete(llmkey::revoke))
        .with_state(app);

    Router::new()
        .route("/hello", get(hello))
        .nest("/api", api)
        .fallback_service(static_service)
        .layer(TraceLayer::new_for_http())
}

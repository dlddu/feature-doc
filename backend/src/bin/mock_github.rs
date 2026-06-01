//! Standalone mock GitHub server, used by the kind-based e2e (and handy for local
//! manual runs). It serves [`featuredoc::mock_github::router`]; point the app's
//! `GITHUB_API_BASE` / `GITHUB_OAUTH_BASE` at it (cluster-internal) and
//! `GITHUB_WEB_BASE` at the browser-reachable address of this server.
//!
//! Env:
//!   MOCK_GITHUB_PORT           listen port (default 8090)
//!   MOCK_GITHUB_APP_BASE_URL   the FeatureDoc app's public origin, where browser
//!                              redirects (authorize / install) are sent back to
//!                              (default http://localhost:8080)
//!   MOCK_GITHUB_DEFAULT_LOGIN  synthetic login used when no `?login=` is supplied
//!                              (default "stub")

use featuredoc::mock_github::{router, MockConfig};

#[tokio::main]
async fn main() {
    featuredoc::init_tracing();

    let port: u16 = std::env::var("MOCK_GITHUB_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8090);
    let app_base_url =
        std::env::var("MOCK_GITHUB_APP_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let default_login =
        std::env::var("MOCK_GITHUB_DEFAULT_LOGIN").unwrap_or_else(|_| "stub".into());

    let cfg = MockConfig {
        app_base_url,
        default_login,
    };

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind mock github");
    tracing::info!("mock github listening on {addr}");
    axum::serve(listener, router(cfg))
        .await
        .expect("serve mock github");
}

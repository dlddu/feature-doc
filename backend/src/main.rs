use std::net::SocketAddr;

use featuredoc::config::Config;
use featuredoc::state::AppState;
use featuredoc::{build_router, db, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;
    let http = reqwest::Client::builder()
        .user_agent("featuredoc/0.1")
        .build()?;

    let state = AppState {
        db: pool,
        config,
        http,
    };

    let app = build_router(state);

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("featuredoc listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

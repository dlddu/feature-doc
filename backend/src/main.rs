use std::net::SocketAddr;

use featuredoc::config::Config;
use featuredoc::state::AppState;
use featuredoc::{build_router, db, init_tracing};
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    // The API is its container's PID 1, and the kernel drops any signal sent to a
    // PID-namespace init that has no handler for it. Without these a rollout's
    // SIGTERM is ignored and the kubelet SIGKILLs only after the whole grace period
    // (30 s by default) — all of it downtime under the Recreate strategy. Installed
    // first so that a stop landing during migrations is still seen.
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

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
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let sig = tokio::select! {
                _ = sigterm.recv() => "SIGTERM",
                _ = sigint.recv() => "SIGINT",
            };
            tracing::info!(signal = sig, "stop requested, draining in-flight requests");
        })
        .await?;
    Ok(())
}

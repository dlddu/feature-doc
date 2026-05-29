use std::net::SocketAddr;

use featuredoc::config::Config;
use featuredoc::state::AppState;
use featuredoc::{db, github, llm, router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env();
    let kek = Config::load_kek();

    let db = db::connect(&config.database_url)
        .await
        .expect("connect + migrate database");

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("build http client");

    let gh = github::build(&config.github, http.clone());
    let validator = llm::build(config.llm_mode, http.clone());

    let app = AppState::new(db, kek, config, gh, validator);
    let app = router(app);

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("featuredoc listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

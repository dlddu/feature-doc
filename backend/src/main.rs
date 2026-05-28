use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

async fn hello() -> Json<Value> {
    Json(json!({ "message": "Hello from FeatureDoc backend" }))
}

#[tokio::main]
async fn main() {
    let dist = std::env::var("STATIC_DIR").unwrap_or_else(|_| "dist".to_string());
    let index = format!("{dist}/index.html");

    let static_service =
        ServeDir::new(&dist).not_found_service(ServeFile::new(index));

    let app = Router::new()
        .route("/hello", get(hello))
        .fallback_service(static_service);

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("featuredoc-hello listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

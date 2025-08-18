use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio;

#[derive(Clone)]
struct AppState {}

async fn simple_handler(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

async fn path_handler(
    State(_state): State<AppState>,
    Path(path): Path<String>,
) -> Json<serde_json::Value> {
    Json(json!({"path": path}))
}

async fn post_handler(
    State(_state): State<AppState>,
    Path(path): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    Json(json!({"path": path, "body": body}))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {};
    
    let app = Router::new()
        .route("/health", get(simple_handler))
        .route("/test/*path", get(path_handler).post(post_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::state::AppState;

pub async fn check(State(_state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "kagikanri"
    }))
}
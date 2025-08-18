use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::{
    error::{ApiResponse, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct OtpResponse {
    pub code: String,
    pub expires_in: u64, // seconds until next code
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtpCreateRequest {
    pub secret: String,
}

pub async fn get(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    ApiResponse::from(async move {
        let code = state.pass.get_otp(&path).await?;
        
        // Calculate expires_in (OTP codes typically refresh every 30 seconds)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_in = 30 - (current_time % 30);
        
        Ok(Json(OtpResponse {
            code,
            expires_in,
        }))
    }.await)
}

pub async fn create(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(request): Json<OtpCreateRequest>,
) -> impl IntoResponse {
    ApiResponse::from(async move {
        state.pass.create_otp(&path, &request.secret).await?;
        
        // Trigger git sync after OTP creation
        if let Err(e) = state.sync_git().await {
            tracing::warn!("Failed to sync git after OTP creation: {}", e);
        }
        
        Ok(Json(serde_json::json!({
            "success": true,
            "path": path,
            "message": "OTP secret added successfully"
        })))
    }.await)
}
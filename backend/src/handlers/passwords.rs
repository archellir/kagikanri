use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use crate::{
    error::{ApiResponse, AppResult},
    pass::{PasswordEntry, PasswordList},
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
) -> impl IntoResponse {
    ApiResponse::from(async move {
        let passwords = state.pass.list_passwords().await?;
        Ok(Json(passwords))
    }.await)
}

pub async fn get(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    ApiResponse::from(async move {
        let password = state.pass.get_password(&path).await?;
        Ok(Json(password))
    }.await)
}

#[axum::debug_handler]
pub async fn create_or_update(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(entry): Json<PasswordEntry>,
) -> impl IntoResponse {
    ApiResponse::from(async move {
        state.pass.create_or_update_password(&path, &entry).await?;
        
        // Trigger git sync after password change
        if let Err(e) = state.sync_git().await {
            tracing::warn!("Failed to sync git after password update: {}", e);
        }
        
        Ok(Json(serde_json::json!({"success": true, "path": path})))
    }.await)
}

pub async fn delete(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    ApiResponse::from(async move {
        state.pass.delete_password(&path).await?;
        
        // Trigger git sync after password deletion
        if let Err(e) = state.sync_git().await {
            tracing::warn!("Failed to sync git after password deletion: {}", e);
        }
        
        Ok(Json(serde_json::json!({"success": true, "deleted": path})))
    }.await)
}
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use crate::{
    error::{ApiResponse, AppResult},
    git::SyncStatus,
    state::AppState,
};

pub async fn trigger(
    State(state): State<AppState>,
) -> impl IntoResponse {
    ApiResponse::from(async move {
        // Perform git sync
        let mut git_sync = state.git_sync.write().await;
        let status = git_sync.sync().await?;
        
        Ok(Json(status))
    }.await)
}

pub async fn status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    ApiResponse::from(async move {
        let git_sync = state.git_sync.read().await;
        let status = git_sync.get_status();
        
        Ok(Json(status))
    }.await)
}
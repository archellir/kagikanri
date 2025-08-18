use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    error::{AppError, AppResult},
    passkey::{PasskeyRegistrationFinish, PasskeyRegistrationStart, StoredPasskey},
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<StoredPasskey>>> {
    let passkeys = state.passkey_store.list_passkeys().await?;
    Ok(Json(passkeys))
}

pub async fn register_start(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> AppResult<Json<PasskeyRegistrationStart>> {
    let domain = request["domain"]
        .as_str()
        .ok_or_else(|| AppError::ValidationError("Domain is required".to_string()))?;
    
    let user_id = request["user_id"]
        .as_str()
        .ok_or_else(|| AppError::ValidationError("User ID is required".to_string()))?;
    
    let registration = state.passkey_store.start_registration(domain, user_id).await?;
    Ok(Json(registration))
}

pub async fn register_finish(
    State(state): State<AppState>,
    Json(request): Json<PasskeyRegistrationFinish>,
) -> AppResult<Json<StoredPasskey>> {
    let passkey = state.passkey_store.finish_registration(request).await?;
    Ok(Json(passkey))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    state.passkey_store.delete_passkey(&id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "deleted": id
    })))
}
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use crate::{
    auth::{AuthService, LoginRequest, LoginResponse},
    error::{AppError, AppResult},
    state::AppState,
};

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let auth_service = AuthService::new(state.config.auth.clone(), state.pass.clone());
    let response = auth_service.authenticate(request).await?;
    
    // Create session in state
    let session_id = state.create_session("user").await;
    
    // Set session cookie
    let cookie = format!(
        "session={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}",
        session_id,
        state.config.auth.session_timeout_hours * 3600
    );
    
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
    
    Ok((headers, Json(response)))
}

pub async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let auth_service = AuthService::new(state.config.auth.clone(), state.pass.clone());
    
    // Extract session from cookie or Authorization header
    let session_id = extract_session(&headers);
    let is_authenticated = match &session_id {
        Some(id) => state.is_authenticated(id).await,
        None => false,
    };
    
    let auth_status = auth_service.get_auth_status(session_id).await;
    
    Json(serde_json::json!({
        "authenticated": is_authenticated,
        "user_id": auth_status.user_id,
        "expires_at": auth_status.expires_at
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Some(session_id) = extract_session(&headers) {
        state.remove_session(&session_id).await;
    }
    
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::SET_COOKIE,
        "session=; HttpOnly; Secure; SameSite=Strict; Max-Age=0".parse().unwrap(),
    );
    
    (response_headers, Json(serde_json::json!({"success": true})))
}

fn extract_session(headers: &HeaderMap) -> Option<String> {
    // Try to get session from cookie first
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(session_value) = cookie.strip_prefix("session=") {
                    return Some(session_value.to_string());
                }
            }
        }
    }
    
    // Fall back to Authorization header
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }
    
    None
}
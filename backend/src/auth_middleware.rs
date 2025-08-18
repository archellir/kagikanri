use axum::{
    extract::Request,
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::state::AppState;

pub async fn auth_middleware(
    headers: HeaderMap,
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for public routes
    let path = request.uri().path();
    if path.starts_with("/api/auth/login") || path.starts_with("/api/health") || path.starts_with("/assets") || path == "/" {
        return Ok(next.run(request).await);
    }

    // Extract session from cookie or Authorization header
    let session_id = extract_session(&headers);
    
    match session_id {
        Some(id) if state.is_authenticated(&id).await => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
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
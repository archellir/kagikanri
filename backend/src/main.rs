use axum::{
    extract::Path,
    http::{header, StatusCode},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod auth;
mod auth_middleware;
mod config;
mod error;
mod git;
mod handlers;
mod pass;
mod passkey;
mod state;

use config::Config;
use error::AppError;
use state::AppState;

#[derive(Parser)]
#[command(name = "kagikanri")]
#[command(about = "A modern, secure, self-hosted password manager")]
struct Cli {
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "kagikanri=info,tower_http=debug".to_string()),
        )
        .init();

    let cli = Cli::parse();
    
    // Load configuration
    let config = Config::load(cli.config.as_deref())?;
    
    // Initialize application state
    let state = AppState::new(config).await?;
    
    // Build our application with routes
    let app = create_router(state);

    // Create socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    
    info!("Starting Kagikanri server on {}", addr);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // Authentication routes
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/status", get(handlers::auth::status))
        .route("/auth/logout", post(handlers::auth::logout))
        
        // Password management routes
        .route("/passwords", get(handlers::passwords::list))
        .route("/passwords/*path", get(handlers::passwords::get)
            .post(handlers::passwords::create_or_update)
            .delete(handlers::passwords::delete))
        
        // OTP routes
        .route("/otp/*path", get(handlers::otp::get)
            .post(handlers::otp::create))
        
        // Passkey routes (optional feature)
        .route("/passkeys", get(handlers::passkeys::list))
        .route("/passkeys/register/start", post(handlers::passkeys::register_start))
        .route("/passkeys/register/finish", post(handlers::passkeys::register_finish))
        .route("/passkeys/:id", delete(handlers::passkeys::delete))
        
        // Sync routes
        .route("/sync", post(handlers::sync::trigger))
        .route("/sync/status", get(handlers::sync::status))
        
        // Health check
        .route("/health", get(handlers::health::check))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware::auth_middleware))
        .with_state(state.clone());

    Router::new()
        // API routes under /api prefix
        .nest("/api", api_routes)
        
        // Static assets
        .route("/assets/*file", get(serve_assets))
        
        // SPA fallback for all other routes
        .fallback(serve_spa)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

async fn serve_spa() -> impl IntoResponse {
    // Serve the built Svelte app's index.html
    match std::fs::read_to_string("../frontend/build/index.html") {
        Ok(content) => Html(content).into_response(),
        Err(_) => {
            // Fallback if frontend not built
            Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Kagikanri</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { 
            font-family: system-ui, -apple-system, sans-serif; 
            max-width: 800px; 
            margin: 0 auto; 
            padding: 2rem; 
            text-align: center;
        }
        .logo { font-size: 4rem; margin-bottom: 1rem; }
        .title { color: #1f2937; margin-bottom: 0.5rem; }
        .subtitle { color: #6b7280; margin-bottom: 2rem; }
        .status { 
            background: #fef3c7; 
            border: 1px solid #f59e0b; 
            border-radius: 0.5rem; 
            padding: 1rem; 
            margin: 2rem 0;
        }
    </style>
</head>
<body>
    <div class="logo">🔐</div>
    <h1 class="title">Kagikanri</h1>
    <p class="subtitle">鍵管理 - Key Management</p>
    <div class="status">
        <p><strong>Development Mode</strong></p>
        <p>Frontend not built. Run <code>cd frontend && pnpm build</code> to build the UI.</p>
        <p>API is available at <a href="/api/health">/api/health</a></p>
    </div>
</body>
</html>
            "#).into_response()
        }
    }
}

async fn serve_assets(Path(file): Path<String>) -> impl IntoResponse {
    // Serve assets from the built frontend
    let asset_path = format!("../frontend/build/{}", file);
    
    match std::fs::read(&asset_path) {
        Ok(content) => {
            // Determine content type from file extension
            let content_type = match file.split('.').last() {
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                Some("html") => "text/html",
                Some("png") => "image/png",
                Some("svg") => "image/svg+xml",
                Some("ico") => "image/x-icon",
                Some("json") => "application/json",
                _ => "application/octet-stream",
            };
            
            (
                [(header::CONTENT_TYPE, content_type)],
                content
            ).into_response()
        },
        Err(_) => (StatusCode::NOT_FOUND, "Asset not found").into_response()
    }
}

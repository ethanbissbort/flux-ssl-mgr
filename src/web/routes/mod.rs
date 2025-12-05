use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::config::Config;

use super::handlers;

/// Health check endpoint
async fn health_check() -> axum::Json<super::super::models::HealthResponse> {
    axum::Json(super::super::models::HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Create the main application router
pub fn create_router(config: Arc<Config>) -> Router {
    // API routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route(
            "/csr/upload",
            post({
                let config = Arc::clone(&config);
                move |multipart| handlers::handle_csr_upload(Arc::clone(&config), multipart)
            }),
        )
        .route(
            "/cert/generate",
            post({
                let config = Arc::clone(&config);
                move |request| handlers::handle_certificate_generate(Arc::clone(&config), request)
            }),
        )
        .route("/cert/info", post(handlers::handle_certificate_info));

    // Main router with API prefix
    Router::new()
        .nest("/api", api_routes)
        // Serve static files from the static directory
        .nest_service("/static", ServeDir::new("static"))
        // TODO: Add web UI routes (HTML pages)
        .route("/", get(|| async { "Flux SSL Manager Web Service" }))
}

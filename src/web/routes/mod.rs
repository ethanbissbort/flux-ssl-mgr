use axum::{
    routing::{get, post},
    Router, Json,
    response::Html,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::config::Config;

use super::handlers;
use super::models::HealthResponse;

// Simple HTML page handlers
async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../../../templates/index.html"))
}

async fn serve_csr_upload() -> Html<&'static str> {
    Html(include_str!("../../../templates/csr-upload.html"))
}

async fn serve_cert_generate() -> Html<&'static str> {
    Html(include_str!("../../../templates/cert-generate.html"))
}

async fn serve_cert_info() -> Html<&'static str> {
    Html(include_str!("../../../templates/cert-info.html"))
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
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
        // Web UI routes (HTML pages)
        .route("/", get(serve_index))
        .route("/csr-upload", get(serve_csr_upload))
        .route("/cert-generate", get(serve_cert_generate))
        .route("/cert-info", get(serve_cert_info))
}

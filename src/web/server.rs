use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::config::Config;
use crate::error::FluxError;

use super::routes;

/// Web server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8443,
        }
    }
}

/// Start the web server
pub async fn start_server(
    config: Arc<Config>,
    server_config: ServerConfig,
) -> Result<(), FluxError> {
    info!("Starting Flux SSL Manager web service");

    // Create the router
    let app = create_app(config);

    // Bind address
    let addr = format!("{}:{}", server_config.bind_address, server_config.port);
    let socket_addr: SocketAddr = addr
        .parse()
        .map_err(|e| FluxError::ConfigError(format!("Invalid bind address: {}", e)))?;

    info!("Server listening on http://{}", socket_addr);
    info!("API documentation available at http://{}/api/health", socket_addr);

    // Create TCP listener
    let listener = TcpListener::bind(socket_addr)
        .await
        .map_err(|e| FluxError::IoError(e))?;

    // Start server
    axum::serve(listener, app)
        .await
        .map_err(|e| FluxError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(())
}

/// Create the application with all middleware
fn create_app(config: Arc<Config>) -> Router {
    routes::create_router(config)
        // Add tracing/logging middleware
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 8443);
    }
}

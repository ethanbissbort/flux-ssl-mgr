//! Web service module for Flux SSL Manager
//!
//! Provides a REST API and web interface for certificate management operations:
//! - CSR upload and signing
//! - Manual certificate generation
//! - Certificate information display

pub mod handlers;
pub mod models;
pub mod routes;
pub mod server;

pub use models::*;
pub use server::{start_server, ServerConfig};

//! Error types for flux-ssl-mgr

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for flux-ssl-mgr operations
pub type Result<T> = std::result::Result<T, FluxError>;

/// Main error type for flux-ssl-mgr
#[derive(Debug, Error)]
pub enum FluxError {
    /// CA key file not found
    #[error("CA key not found: {0}")]
    CaKeyNotFound(PathBuf),

    /// CA certificate not found
    #[error("CA certificate not found: {0}")]
    CaCertNotFound(PathBuf),

    /// OpenSSL configuration file not found
    #[error("OpenSSL configuration file not found: {0}")]
    OpenSslConfigNotFound(PathBuf),

    /// Invalid certificate name
    #[error("Invalid certificate name: {0}")]
    InvalidCertName(String),

    /// Invalid Subject Alternative Name format
    #[error("Invalid SAN format: {0}")]
    InvalidSanFormat(String),

    /// Working directory not found
    #[error("Working directory not found: {0}")]
    WorkingDirNotFound(PathBuf),

    /// Output directory creation failed
    #[error("Failed to create output directory: {0}")]
    OutputDirCreationFailed(PathBuf),

    /// CSR file not found
    #[error("CSR file not found: {0}")]
    CsrNotFound(PathBuf),

    /// No CSR files found in directory
    #[error("No CSR files found in directory: {0}")]
    NoCsrFilesFound(PathBuf),

    /// Failed to read CSR file
    #[error("Failed to read CSR file: {0}")]
    CsrReadFailed(PathBuf),

    /// OpenSSL error
    #[error("OpenSSL error: {0}")]
    OpenSslError(#[from] openssl::error::ErrorStack),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    /// Permission error
    #[error("Permission error: {0}")]
    PermissionError(String),

    /// Password verification failed
    #[error("Password verification failed")]
    PasswordVerificationFailed,

    /// CA key unlock failed
    #[error("Failed to unlock CA key")]
    CaKeyUnlockFailed,

    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// CSR generation failed
    #[error("CSR generation failed: {0}")]
    CsrGenerationFailed(String),

    /// Certificate signing failed
    #[error("Certificate signing failed: {0}")]
    CertSigningFailed(String),

    /// File write failed
    #[error("Failed to write file {0}: {1}")]
    FileWriteFailed(PathBuf, String),

    /// File read failed
    #[error("Failed to read file {0}: {1}")]
    FileReadFailed(PathBuf, String),

    /// Invalid configuration value
    #[error("Invalid configuration value for {0}: {1}")]
    InvalidConfigValue(String, String),

    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    /// Certificate parsing error
    #[error("Failed to parse certificate: {0}")]
    CertParseError(String),

    /// User cancelled operation
    #[error("Operation cancelled by user")]
    UserCancelled,

    /// Interactive mode error
    #[error("Interactive mode error: {0}")]
    InteractiveError(String),

    /// Batch processing error
    #[error("Batch processing failed: {0} successful, {1} failed")]
    BatchProcessingError(usize, usize),
}

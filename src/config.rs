//! Configuration management for flux-ssl-mgr

use crate::error::{FluxError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// PKI working directory
    pub working_dir: PathBuf,

    /// Output directory for generated certificates
    pub output_dir: PathBuf,

    /// Input directory for CSR files
    pub csr_input_dir: PathBuf,

    /// Path to CA private key
    pub ca_key_path: PathBuf,

    /// Path to CA certificate
    pub ca_cert_path: PathBuf,

    /// Path to OpenSSL configuration file
    pub openssl_config: PathBuf,

    /// Default certificate settings
    #[serde(default)]
    pub defaults: Defaults,

    /// File permission settings
    #[serde(default)]
    pub permissions: Permissions,

    /// Batch processing settings
    #[serde(default)]
    pub batch: BatchConfig,

    /// Output formatting settings
    #[serde(default)]
    pub output: OutputConfig,
}

/// Default certificate settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    /// RSA key size in bits
    #[serde(default = "default_key_size")]
    pub key_size: u32,

    /// Certificate validity period in days
    #[serde(default = "default_cert_days")]
    pub cert_days: u32,

    /// Signature hash algorithm
    #[serde(default = "default_hash_algorithm")]
    pub hash_algorithm: String,

    /// Default file owner
    #[serde(default = "default_owner")]
    pub owner: String,

    /// Default file group
    #[serde(default = "default_group")]
    pub group: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            key_size: default_key_size(),
            cert_days: default_cert_days(),
            hash_algorithm: default_hash_algorithm(),
            owner: default_owner(),
            group: default_group(),
        }
    }
}

/// File permission settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    /// Private key file permissions (octal)
    #[serde(default = "default_private_key_perms")]
    pub private_key: u32,

    /// Certificate file permissions (octal)
    #[serde(default = "default_certificate_perms")]
    pub certificate: u32,

    /// Output directory permissions (octal)
    #[serde(default = "default_output_dir_perms")]
    pub output_dir: u32,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            private_key: default_private_key_perms(),
            certificate: default_certificate_perms(),
            output_dir: default_output_dir_perms(),
        }
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Enable parallel processing
    #[serde(default = "default_parallel")]
    pub parallel: bool,

    /// Maximum number of concurrent workers
    #[serde(default = "default_max_workers")]
    pub max_workers: usize,

    /// Show progress bar
    #[serde(default = "default_progress_bar")]
    pub progress_bar: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            parallel: default_parallel(),
            max_workers: default_max_workers(),
            progress_bar: default_progress_bar(),
        }
    }
}

/// Output formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Enable colored output
    #[serde(default = "default_colored")]
    pub colored: bool,

    /// Verbose logging
    #[serde(default)]
    pub verbose: bool,

    /// Quiet mode (suppress non-error output)
    #[serde(default)]
    pub quiet: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            colored: default_colored(),
            verbose: false,
            quiet: false,
        }
    }
}

// Default value functions
fn default_key_size() -> u32 { 4096 }
fn default_cert_days() -> u32 { 375 }
fn default_hash_algorithm() -> String { "sha256".to_string() }
fn default_owner() -> String { "fluxadmin".to_string() }
fn default_group() -> String { "root".to_string() }
fn default_private_key_perms() -> u32 { 0o400 }
fn default_certificate_perms() -> u32 { 0o755 }
fn default_output_dir_perms() -> u32 { 0o755 }
fn default_parallel() -> bool { true }
fn default_max_workers() -> usize { 4 }
fn default_progress_bar() -> bool { true }
fn default_colored() -> bool { true }

impl Config {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        // Try to find config file in standard locations
        let config_paths = [
            PathBuf::from("./flux-ssl-mgr.toml"),
            dirs::config_dir()
                .map(|d| d.join("flux-ssl-mgr/config.toml"))
                .unwrap_or_default(),
            PathBuf::from("/etc/flux-ssl-mgr/config.toml"),
        ];

        for path in &config_paths {
            if path.exists() {
                return Self::from_file(path);
            }
        }

        // Return default config if no file found
        Ok(Self::default())
    }

    /// Load configuration from a specific file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_str = std::fs::read_to_string(path.as_ref())
            .map_err(|e| FluxError::FileReadFailed(
                path.as_ref().to_path_buf(),
                e.to_string()
            ))?;

        let config: Config = toml::from_str(&config_str)
            .map_err(|e| FluxError::InvalidConfigValue(
                "config file".to_string(),
                e.to_string()
            ))?;

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Check if working directory exists
        if !self.working_dir.exists() {
            return Err(FluxError::WorkingDirNotFound(self.working_dir.clone()));
        }

        // Check if CA key exists
        if !self.ca_key_path.exists() {
            return Err(FluxError::CaKeyNotFound(self.ca_key_path.clone()));
        }

        // Check if CA cert exists
        if !self.ca_cert_path.exists() {
            return Err(FluxError::CaCertNotFound(self.ca_cert_path.clone()));
        }

        // Check if OpenSSL config exists
        if !self.openssl_config.exists() {
            return Err(FluxError::OpenSslConfigNotFound(self.openssl_config.clone()));
        }

        Ok(())
    }

    /// Create default config file template
    pub fn create_default_template() -> String {
        toml::to_string_pretty(&Self::default()).unwrap_or_default()
    }

    /// Save configuration to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_str = toml::to_string_pretty(self)
            .map_err(|e| FluxError::InvalidConfigValue(
                "serialization".to_string(),
                e.to_string()
            ))?;

        std::fs::write(path.as_ref(), config_str)
            .map_err(|e| FluxError::FileWriteFailed(
                path.as_ref().to_path_buf(),
                e.to_string()
            ))?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            working_dir: PathBuf::from("/root/ca"),
            output_dir: PathBuf::from("/home/fluxadmin/ssl/pem-out"),
            csr_input_dir: PathBuf::from("/home/fluxadmin/ssl"),
            ca_key_path: PathBuf::from("/root/ca/intermediate/private/intermediate.key.pem"),
            ca_cert_path: PathBuf::from("/root/ca/intermediate/certs/intermediate.cert.pem"),
            openssl_config: PathBuf::from("/root/ca/intermediate/openssl.cnf"),
            defaults: Defaults::default(),
            permissions: Permissions::default(),
            batch: BatchConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

// Helper module for dirs crate functionality
mod dirs {
    use std::path::PathBuf;

    pub fn config_dir() -> Option<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            Some(PathBuf::from(home).join(".config"))
        } else {
            None
        }
    }
}

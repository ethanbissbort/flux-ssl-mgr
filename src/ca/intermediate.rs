//! Intermediate CA management

use crate::config::Config;
use crate::crypto::{load_private_key, load_cert, is_key_encrypted, unlock_ca_key};
use crate::error::{FluxError, Result};
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;
use secrecy::{Secret, ExposeSecret};
use std::path::PathBuf;

/// Represents an intermediate Certificate Authority
pub struct IntermediateCA {
    /// CA private key
    key: PKey<Private>,
    /// CA certificate
    cert: X509,
    /// Temporary file handle (if CA key was unlocked)
    _temp_file: Option<tempfile::NamedTempFile>,
}

impl IntermediateCA {
    /// Load the intermediate CA from configuration
    pub fn load(config: &Config) -> Result<Self> {
        // Load CA certificate
        let cert = load_cert(&config.ca_cert_path)?;

        // Check if CA key is encrypted
        let is_encrypted = is_key_encrypted(&config.ca_key_path)?;

        let (key, temp_file) = if is_encrypted {
            // Prompt for password
            use dialoguer::Password;
            let password = Password::new()
                .with_prompt("Enter intermediate CA private key password")
                .interact()
                .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

            // Unlock the CA key
            let (key, temp) = unlock_ca_key(&config.ca_key_path, &password)?;
            (key, Some(temp))
        } else {
            // Load unencrypted key
            let key = load_private_key(&config.ca_key_path, None)?;
            (key, None)
        };

        Ok(Self {
            key,
            cert,
            _temp_file: temp_file,
        })
    }

    /// Load CA with provided password
    pub fn load_with_password(config: &Config, password: &str) -> Result<Self> {
        let cert = load_cert(&config.ca_cert_path)?;

        let is_encrypted = is_key_encrypted(&config.ca_key_path)?;

        let (key, temp_file) = if is_encrypted {
            let (key, temp) = unlock_ca_key(&config.ca_key_path, password)?;
            (key, Some(temp))
        } else {
            let key = load_private_key(&config.ca_key_path, None)?;
            (key, None)
        };

        Ok(Self {
            key,
            cert,
            _temp_file: temp_file,
        })
    }

    /// Get reference to CA private key
    pub fn key(&self) -> &PKey<Private> {
        &self.key
    }

    /// Get reference to CA certificate
    pub fn cert(&self) -> &X509 {
        &self.cert
    }

    /// Get CA subject name
    pub fn subject(&self) -> String {
        format!("{:?}", self.cert.subject_name())
    }

    /// Verify CA certificate is valid
    pub fn verify(&self) -> Result<bool> {
        self.cert.verify(&self.key)
            .map_err(|e| FluxError::CertParseError(e.to_string()))
    }
}

impl Drop for IntermediateCA {
    fn drop(&mut self) {
        // Temp file will be automatically cleaned up
        tracing::debug!("Cleaning up intermediate CA resources");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a real CA setup
    // For now, we'll skip them in the test environment
}

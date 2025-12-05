use axum::Json;
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;
use tracing::{debug, info};
use validator::Validate;

use crate::ca::IntermediateCA;
use crate::config::Config;
use crate::crypto;

use super::super::models::{
    CertificateGenerateRequest, CertificateGenerateResponse, CertificateWithKey, WebError,
};

/// Handle manual certificate generation request
pub async fn handle_certificate_generate(
    config: Arc<Config>,
    Json(request): Json<CertificateGenerateRequest>,
) -> Result<Json<CertificateGenerateResponse>, WebError> {
    info!(
        "Processing certificate generation request for CN={}",
        request.common_name
    );

    // Validate request
    request
        .validate()
        .map_err(|e| WebError::invalid_input(format!("Validation failed: {}", e)))?;

    // Validate password requirement
    if request.password_protect && request.key_password.is_none() {
        return Err(WebError::invalid_input(
            "Password required when password_protect is true",
        ));
    }

    // Generate private key
    debug!("Generating RSA private key (size: {})", request.key_size);
    let private_key = crypto::key::generate_rsa_key(request.key_size)
        .map_err(|e| WebError::key_generation_failed(format!("Failed to generate key: {}", e)))?;

    // Convert key to PEM (optionally encrypted)
    let key_pem = if request.password_protect {
        let password = request.key_password.as_ref().unwrap();
        let secret = Secret::new(password.clone());

        crypto::key::save_encrypted_key(&private_key, &secret).map_err(|e| {
            WebError::key_generation_failed(format!("Failed to encrypt key: {}", e))
        })?
    } else {
        crypto::key::save_key(&private_key)
            .map_err(|e| WebError::key_generation_failed(format!("Failed to save key: {}", e)))?
    };

    debug!("Private key generated successfully");

    // Parse SANs
    let sans: Vec<crypto::csr::SanEntry> = request
        .sans
        .iter()
        .map(|s| crypto::csr::parse_san(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| WebError::invalid_input(format!("Invalid SAN format: {}", e)))?;

    // Create CSR
    debug!("Creating CSR with CN={}", request.common_name);
    let csr = crypto::csr::create_csr(&private_key, &request.common_name, sans.clone())
        .map_err(|e| WebError::signing_failed(format!("Failed to create CSR: {}", e)))?;

    debug!("CSR created successfully");

    // Load CA
    let ca = IntermediateCA::load(config.clone())
        .map_err(|e| WebError::ca_error(format!("Failed to load CA: {}", e)))?;

    debug!("CA loaded successfully");

    // Sign certificate
    let cert = crypto::cert::sign_csr(&csr, &ca.cert, &ca.key, request.validity_days, vec![])
        .map_err(|e| WebError::signing_failed(format!("Failed to sign certificate: {}", e)))?;

    info!("Certificate signed successfully");

    // Extract certificate information
    let cert_info = crypto::cert::extract_certificate_info(&cert)
        .map_err(|e| WebError::internal_error(format!("Failed to extract cert info: {}", e)))?;

    // Convert certificate to PEM
    let cert_pem = crypto::cert::to_pem(&cert)
        .map_err(|e| WebError::internal_error(format!("Failed to convert cert to PEM: {}", e)))?;

    // TODO: Load CA chain from config
    // For now, we'll leave it as None
    let ca_chain = None;

    let response = CertificateGenerateResponse {
        success: true,
        certificate: CertificateWithKey {
            pem: String::from_utf8_lossy(&cert_pem).to_string(),
            private_key: String::from_utf8_lossy(&key_pem).to_string(),
            ca_chain,
            subject: cert_info.subject,
            serial: cert_info.serial_number,
            not_before: DateTime::from_timestamp(cert_info.not_before.timestamp(), 0)
                .unwrap_or_default(),
            not_after: DateTime::from_timestamp(cert_info.not_after.timestamp(), 0)
                .unwrap_or_default(),
            sans: cert_info.sans,
            download_url: None, // TODO: Implement download functionality
        },
    };

    Ok(Json(response))
}

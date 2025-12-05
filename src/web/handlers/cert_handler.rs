use axum::Json;
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;
use tracing::{debug, info};

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

    // Validate common name
    if request.common_name.is_empty() {
        return Err(WebError::invalid_input("Common name cannot be empty"));
    }

    // Validate validity days
    if request.validity_days == 0 || request.validity_days > 825 {
        return Err(WebError::invalid_input("Validity days must be between 1 and 825"));
    }

    // Validate key size
    if request.key_size != 2048 && request.key_size != 4096 {
        return Err(WebError::invalid_input("Key size must be 2048 or 4096"));
    }

    // Validate password requirement
    if request.password_protect && request.key_password.is_none() {
        return Err(WebError::invalid_input(
            "Password required when password_protect is true",
        ));
    }

    // Generate private key
    debug!("Generating RSA private key (size: {})", request.key_size);
    let private_key = crypto::generate_rsa_key(request.key_size, None)
        .map_err(|e| WebError::key_generation_failed(format!("Failed to generate key: {}", e)))?;

    // Convert key to PEM (optionally encrypted)
    let key_pem = if request.password_protect {
        let password = request.key_password.as_ref().unwrap();
        let secret = Secret::new(password.clone());

        crypto::key_to_encrypted_pem(&private_key, &secret).map_err(|e| {
            WebError::key_generation_failed(format!("Failed to encrypt key: {}", e))
        })?
    } else {
        crypto::key_to_pem(&private_key)
            .map_err(|e| WebError::key_generation_failed(format!("Failed to save key: {}", e)))?
    };

    debug!("Private key generated successfully");

    // Parse SANs
    let sans: Vec<crypto::SanEntry> = request
        .sans
        .iter()
        .map(|s| crypto::SanEntry::parse(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| WebError::invalid_input(format!("Invalid SAN format: {}", e)))?;

    // Create CSR
    debug!("Creating CSR with CN={}", request.common_name);
    let csr = crypto::create_csr(&request.common_name, &private_key, &sans, Some(&request.common_name))
        .map_err(|e| WebError::signing_failed(format!("Failed to create CSR: {}", e)))?;

    debug!("CSR created successfully");

    // Load CA
    let ca = IntermediateCA::load(&config)
        .map_err(|e| WebError::ca_error(format!("Failed to load CA: {}", e)))?;

    debug!("CA loaded successfully");

    // Sign certificate
    let cert = crypto::sign_csr(&csr, ca.cert(), ca.key(), request.validity_days)
        .map_err(|e| WebError::signing_failed(format!("Failed to sign certificate: {}", e)))?;

    info!("Certificate signed successfully");

    // Extract certificate information
    let cert_info = crypto::extract_certificate_info(&cert)
        .map_err(|e| WebError::internal_error(format!("Failed to extract cert info: {}", e)))?;

    // Convert certificate to PEM
    let cert_pem = crypto::cert_to_pem(&cert)
        .map_err(|e| WebError::internal_error(format!("Failed to convert cert to PEM: {}", e)))?;

    // Load CA chain (intermediate + root CA)
    let ca_chain = build_ca_chain(&config, &ca).ok();

    let response = CertificateGenerateResponse {
        success: true,
        certificate: CertificateWithKey {
            pem: String::from_utf8_lossy(&cert_pem).to_string(),
            private_key: String::from_utf8_lossy(&key_pem).to_string(),
            ca_chain,
            subject: cert_info.subject,
            serial: cert_info.serial_number,
            not_before: cert_info.not_before,
            not_after: cert_info.not_after,
            sans: cert_info.sans,
            download_url: None, // API returns PEM data directly; clients can save locally
        },
    };

    Ok(Json(response))
}

/// Build CA certificate chain (intermediate + root)
fn build_ca_chain(config: &Config, ca: &IntermediateCA) -> std::result::Result<String, WebError> {
    let mut chain = String::new();

    // Add intermediate CA certificate
    let intermediate_pem = crypto::cert_to_pem(ca.cert())
        .map_err(|e| WebError::internal_error(format!("Failed to convert intermediate cert: {}", e)))?;
    chain.push_str(&String::from_utf8_lossy(&intermediate_pem));

    // Try to load root CA certificate
    // Standard PKI structure places root CA at /root/ca/certs/ca.cert.pem
    let root_ca_path = config.working_dir.join("certs").join("ca.cert.pem");

    if root_ca_path.exists() {
        debug!("Loading root CA from {:?}", root_ca_path);
        match crypto::load_cert(&root_ca_path) {
            Ok(root_cert) => {
                let root_pem = crypto::cert_to_pem(&root_cert)
                    .map_err(|e| WebError::internal_error(format!("Failed to convert root cert: {}", e)))?;
                chain.push_str(&String::from_utf8_lossy(&root_pem));
                debug!("Root CA added to chain");
            }
            Err(e) => {
                debug!("Failed to load root CA: {}", e);
                // Continue without root CA
            }
        }
    } else {
        debug!("Root CA not found at {:?}, chain will only contain intermediate", root_ca_path);
    }

    Ok(chain)
}

use axum::{extract::Multipart, Json};
use std::sync::Arc;
use tracing::{debug, info};

use crate::ca::IntermediateCA;
use crate::config::Config;
use crate::crypto;

use super::super::models::{
    CertificateInfo, CsrUploadMetadata, CsrUploadResponse, WebError,
};

/// Handle CSR upload and signing
pub async fn handle_csr_upload(
    config: Arc<Config>,
    mut multipart: Multipart,
) -> Result<Json<CsrUploadResponse>, WebError> {
    info!("Processing CSR upload request");

    let mut csr_data: Option<Vec<u8>> = None;
    let mut metadata = CsrUploadMetadata {
        sans: Vec::new(),
        validity_days: config.defaults.cert_days,
    };

    // Parse multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| WebError::bad_request(format!("Failed to parse form data: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        debug!("Processing field: {}", name);

        match name.as_str() {
            "csr_file" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| WebError::bad_request(format!("Failed to read CSR file: {}", e)))?;

                if data.is_empty() {
                    return Err(WebError::invalid_csr("CSR file is empty"));
                }

                // Check file size (5MB limit)
                if data.len() > 5 * 1024 * 1024 {
                    return Err(WebError::file_too_large("CSR file exceeds 5MB limit"));
                }

                csr_data = Some(data.to_vec());
            }
            "sans" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| WebError::bad_request(format!("Failed to read SANs: {}", e)))?;

                // Parse comma-separated SANs
                metadata.sans = text
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            "validity_days" => {
                let text = field.text().await.unwrap_or_default();
                metadata.validity_days = text.parse().unwrap_or(config.defaults.cert_days);
            }
            _ => {
                debug!("Ignoring unknown field: {}", name);
            }
        }
    }

    let csr_data = csr_data.ok_or_else(|| WebError::bad_request("No CSR file provided"))?;

    // Parse CSR
    let csr = crypto::csr_from_pem_bytes(&csr_data)
        .map_err(|e| WebError::invalid_csr(format!("Failed to parse CSR: {}", e)))?;

    debug!("CSR parsed successfully");

    // Parse additional SANs (currently not used in sign_csr, but could be extended)
    let _additional_sans: Vec<crypto::SanEntry> = metadata
        .sans
        .iter()
        .map(|s| crypto::SanEntry::parse(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| WebError::invalid_input(format!("Invalid SAN format: {}", e)))?;

    // Load CA
    let ca = IntermediateCA::load(&config)
        .map_err(|e| WebError::ca_error(format!("Failed to load CA: {}", e)))?;

    debug!("CA loaded successfully");

    // Sign certificate
    let cert = crypto::sign_csr(&csr, ca.cert(), ca.key(), metadata.validity_days)
        .map_err(|e| WebError::signing_failed(format!("Failed to sign certificate: {}", e)))?;

    info!("Certificate signed successfully");

    // Extract certificate information
    let cert_info = crypto::extract_certificate_info(&cert)
        .map_err(|e| WebError::internal_error(format!("Failed to extract cert info: {}", e)))?;

    // Convert to PEM
    let pem = crypto::cert_to_pem(&cert)
        .map_err(|e| WebError::internal_error(format!("Failed to convert to PEM: {}", e)))?;

    let response = CsrUploadResponse {
        success: true,
        certificate: CertificateInfo {
            pem: String::from_utf8_lossy(&pem).to_string(),
            subject: cert_info.subject,
            issuer: cert_info.issuer,
            serial: cert_info.serial_number,
            not_before: cert_info.not_before,
            not_after: cert_info.not_after,
            sans: cert_info.sans,
        },
    };

    Ok(Json(response))
}

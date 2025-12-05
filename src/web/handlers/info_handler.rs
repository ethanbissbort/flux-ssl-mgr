use axum::{extract::Multipart, Json};
use chrono::{DateTime, Utc};
use openssl::hash::MessageDigest;
use openssl::x509::X509;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::crypto;

use super::super::models::{
    CertificateInfoResponse, DetailedCertificateInfo, ExtensionInfo, FingerprintInfo,
    PublicKeyInfo, ValidityInfo, WebError,
};

/// Handle certificate information request
pub async fn handle_certificate_info(
    mut multipart: Multipart,
) -> Result<Json<CertificateInfoResponse>, WebError> {
    info!("Processing certificate info request");

    let mut cert_data: Option<Vec<u8>> = None;
    let mut verify_chain = false;

    // Parse multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| WebError::bad_request(format!("Failed to parse form data: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        debug!("Processing field: {}", name);

        match name.as_str() {
            "cert_file" => {
                let data = field.bytes().await.map_err(|e| {
                    WebError::bad_request(format!("Failed to read certificate file: {}", e))
                })?;

                if data.is_empty() {
                    return Err(WebError::invalid_certificate("Certificate file is empty"));
                }

                // Check file size (5MB limit)
                if data.len() > 5 * 1024 * 1024 {
                    return Err(WebError::file_too_large(
                        "Certificate file exceeds 5MB limit",
                    ));
                }

                cert_data = Some(data.to_vec());
            }
            "verify_chain" => {
                let text = field.text().await.unwrap_or_default();
                verify_chain = text.parse().unwrap_or(false);
            }
            _ => {
                debug!("Ignoring unknown field: {}", name);
            }
        }
    }

    let cert_data =
        cert_data.ok_or_else(|| WebError::bad_request("No certificate file provided"))?;

    // Parse certificate
    let cert = X509::from_pem(&cert_data).map_err(|e| {
        WebError::invalid_certificate(format!("Failed to parse certificate: {}", e))
    })?;

    debug!("Certificate parsed successfully");

    // Extract certificate information
    let cert_info = crypto::extract_certificate_info(&cert).map_err(|e| {
        WebError::internal_error(format!("Failed to extract certificate info: {}", e))
    })?;

    // Calculate fingerprints
    let sha1_digest = cert
        .digest(MessageDigest::sha1())
        .map_err(|e| WebError::internal_error(format!("Failed to calculate SHA1: {}", e)))?;

    let sha256_digest = cert
        .digest(MessageDigest::sha256())
        .map_err(|e| WebError::internal_error(format!("Failed to calculate SHA256: {}", e)))?;

    let sha1 = sha1_digest
        .as_ref()
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":");

    let sha256 = sha256_digest
        .as_ref()
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":");

    // Extract issuer and subject into HashMaps
    let issuer = parse_x509_name(cert.issuer_name());
    let subject = parse_x509_name(cert.subject_name());

    // Calculate validity info
    let now = chrono::Utc::now();
    let not_before = cert_info.not_before;
    let not_after = cert_info.not_after;
    let days_remaining = (not_after - now).num_days();
    let is_expired = now > not_after;
    let is_expiring_soon = days_remaining < 30 && !is_expired;

    // Get public key info
    let public_key_info = extract_public_key_info(&cert)?;

    // Extract extensions
    let extensions = extract_extensions(&cert);

    // Convert to PEM
    let pem = cert
        .to_pem()
        .map_err(|e| WebError::internal_error(format!("Failed to convert to PEM: {}", e)))?;

    let response = CertificateInfoResponse {
        success: true,
        certificate: DetailedCertificateInfo {
            version: cert.version() + 1, // OpenSSL uses 0-based versioning
            serial_number: cert_info.serial_number.clone(),
            signature_algorithm: cert_info.signature_algorithm.clone(),
            issuer,
            validity: ValidityInfo {
                not_before,
                not_after,
                days_remaining,
                is_expired,
                is_expiring_soon,
            },
            subject,
            subject_alternative_names: cert_info.sans.clone(),
            public_key: public_key_info,
            extensions,
            fingerprints: FingerprintInfo { sha1, sha256 },
            pem: String::from_utf8_lossy(&pem).to_string(),
        },
    };

    info!("Certificate info extracted successfully");

    Ok(Json(response))
}

/// Parse X509Name into HashMap
fn parse_x509_name(name: &openssl::x509::X509NameRef) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for entry in name.entries() {
        if let Ok(data) = entry.data().as_utf8() {
            let key = entry.object().nid().short_name().unwrap_or("UNKNOWN");
            map.insert(key.to_string(), data.to_string());
        }
    }

    map
}

/// Extract public key information
fn extract_public_key_info(cert: &X509) -> Result<PublicKeyInfo, WebError> {
    let public_key = cert
        .public_key()
        .map_err(|e| WebError::internal_error(format!("Failed to get public key: {}", e)))?;

    let algorithm = if public_key.rsa().is_ok() {
        "RSA"
    } else if public_key.ec_key().is_ok() {
        "ECDSA"
    } else {
        "UNKNOWN"
    };

    let size = public_key.bits();

    let exponent = if let Ok(rsa) = public_key.rsa() {
        rsa.e().to_dec_str().ok().and_then(|s| s.parse().ok())
    } else {
        None
    };

    Ok(PublicKeyInfo {
        algorithm: algorithm.to_string(),
        size,
        exponent,
    })
}

/// Extract certificate extensions
fn extract_extensions(cert: &X509) -> Vec<ExtensionInfo> {
    let mut extensions = Vec::new();

    // Extract Subject Alternative Names
    if let Some(san_ext) = cert.subject_alt_names() {
        let mut sans = Vec::new();
        for san in san_ext {
            if let Some(dns) = san.dnsname() {
                sans.push(format!("DNS:{}", dns));
            }
            if let Some(ip) = san.ipaddress() {
                let ip_str = ip.iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(".");
                sans.push(format!("IP:{}", ip_str));
            }
            if let Some(email) = san.email() {
                sans.push(format!("EMAIL:{}", email));
            }
        }

        if !sans.is_empty() {
            extensions.push(ExtensionInfo {
                oid: "2.5.29.17".to_string(),
                name: "Subject Alternative Name".to_string(),
                critical: false,
                value: sans.join(", "),
            });
        }
    }

    // Extract Authority Key Identifier
    if let Some(aki) = cert.authority_key_id() {
        let keyid = aki.as_slice()
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":");

        extensions.push(ExtensionInfo {
            oid: "2.5.29.35".to_string(),
            name: "Authority Key Identifier".to_string(),
            critical: false,
            value: format!("keyid:{}", keyid),
        });
    }

    // Extract Subject Key Identifier
    if let Some(ski) = cert.subject_key_id() {
        let keyid = ski.as_slice()
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":");

        extensions.push(ExtensionInfo {
            oid: "2.5.29.14".to_string(),
            name: "Subject Key Identifier".to_string(),
            critical: false,
            value: keyid,
        });
    }

    // Note: OpenSSL version in use doesn't expose direct methods for
    // Basic Constraints, Key Usage, Extended Key Usage extraction.
    // These would require parsing the extension stack directly which is
    // version-dependent. The above extensions cover the most critical
    // certificate information for web service use.

    extensions
}

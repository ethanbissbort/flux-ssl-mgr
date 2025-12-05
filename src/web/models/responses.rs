use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Certificate information in API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// Certificate in PEM format
    pub pem: String,

    /// Subject DN
    pub subject: String,

    /// Issuer DN
    pub issuer: String,

    /// Serial number (hex)
    pub serial: String,

    /// Not valid before timestamp
    pub not_before: DateTime<Utc>,

    /// Not valid after timestamp
    pub not_after: DateTime<Utc>,

    /// Subject Alternative Names
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sans: Vec<String>,
}

/// Certificate information with private key (for generation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateWithKey {
    /// Certificate in PEM format
    pub pem: String,

    /// Private key in PEM format (encrypted if password was provided)
    pub private_key: String,

    /// CA chain in PEM format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_chain: Option<String>,

    /// Subject DN
    pub subject: String,

    /// Serial number (hex)
    pub serial: String,

    /// Not valid before timestamp
    pub not_before: DateTime<Utc>,

    /// Not valid after timestamp
    pub not_after: DateTime<Utc>,

    /// Subject Alternative Names
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sans: Vec<String>,

    /// Download URL for certificate bundle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
}

/// Detailed certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedCertificateInfo {
    /// X.509 version
    pub version: i32,

    /// Serial number (hex)
    pub serial_number: String,

    /// Signature algorithm
    pub signature_algorithm: String,

    /// Issuer DN components
    pub issuer: HashMap<String, String>,

    /// Validity information
    pub validity: ValidityInfo,

    /// Subject DN components
    pub subject: HashMap<String, String>,

    /// Subject Alternative Names
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub subject_alternative_names: Vec<String>,

    /// Public key information
    pub public_key: PublicKeyInfo,

    /// Certificate extensions
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extensions: Vec<ExtensionInfo>,

    /// Fingerprints
    pub fingerprints: FingerprintInfo,

    /// Certificate in PEM format
    pub pem: String,
}

/// Validity period information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidityInfo {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub days_remaining: i64,
    pub is_expired: bool,
    pub is_expiring_soon: bool,
}

/// Public key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyInfo {
    pub algorithm: String,
    pub size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exponent: Option<u64>,
}

/// Extension information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub oid: String,
    pub name: String,
    pub critical: bool,
    pub value: String,
}

/// Fingerprint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintInfo {
    pub sha1: String,
    pub sha256: String,
}

/// Generic success response for CSR upload
#[derive(Debug, Serialize, Deserialize)]
pub struct CsrUploadResponse {
    pub success: bool,
    pub certificate: CertificateInfo,
}

/// Generic success response for certificate generation
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateGenerateResponse {
    pub success: bool,
    pub certificate: CertificateWithKey,
}

/// Generic success response for certificate info
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateInfoResponse {
    pub success: bool,
    pub certificate: DetailedCertificateInfo,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

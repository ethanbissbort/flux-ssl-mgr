use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request to generate a certificate from manual input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CertificateGenerateRequest {
    /// Common Name for the certificate
    #[validate(length(min = 1, max = 64))]
    pub common_name: String,

    /// Subject Alternative Names
    #[serde(default)]
    pub sans: Vec<String>,

    /// Validity period in days
    #[validate(range(min = 1, max = 825))]
    #[serde(default = "default_validity_days")]
    pub validity_days: u32,

    /// RSA key size in bits
    #[validate(custom(function = "validate_key_size"))]
    #[serde(default = "default_key_size")]
    pub key_size: u32,

    /// Whether to password-protect the private key
    #[serde(default)]
    pub password_protect: bool,

    /// Password for the private key (if password_protect is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_password: Option<String>,
}

fn default_validity_days() -> u32 {
    375
}

fn default_key_size() -> u32 {
    4096
}

fn validate_key_size(key_size: u32) -> Result<(), validator::ValidationError> {
    if key_size == 2048 || key_size == 4096 {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_key_size"))
    }
}

/// Request metadata for CSR upload (from form data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrUploadMetadata {
    /// Additional SANs to add to the certificate
    #[serde(default)]
    pub sans: Vec<String>,

    /// Validity period in days
    #[serde(default = "default_validity_days")]
    pub validity_days: u32,
}

/// Request metadata for certificate info (from form data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertInfoMetadata {
    /// Whether to verify the certificate chain
    #[serde(default)]
    pub verify_chain: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_certificate_generate_request() {
        let req = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec!["DNS:www.example.com".to_string()],
            validity_days: 375,
            key_size: 4096,
            password_protect: false,
            key_password: None,
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_key_size() {
        let req = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec![],
            validity_days: 375,
            key_size: 1024, // Invalid
            password_protect: false,
            key_password: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_validity_days() {
        let req = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec![],
            validity_days: 1000, // Too long
            key_size: 4096,
            password_protect: false,
            key_password: None,
        };

        assert!(req.validate().is_err());
    }
}

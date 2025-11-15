//! Certificate Signing Request (CSR) generation and management

use crate::error::{FluxError, Result};
use openssl::x509::{X509Req, X509ReqBuilder, X509Name, X509NameBuilder};
use openssl::x509::extension::SubjectAlternativeName;
use openssl::pkey::{PKey, Private};
use openssl::hash::MessageDigest;
use std::path::Path;

/// Subject Alternative Name entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SanEntry {
    /// DNS name
    Dns(String),
    /// IP address
    Ip(String),
    /// Email address
    Email(String),
}

impl SanEntry {
    /// Parse SAN entry from string (e.g., "DNS:example.com" or "IP:192.168.1.1")
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(FluxError::InvalidSanFormat(s.to_string()));
        }

        let san_type = parts[0].to_uppercase();
        let value = parts[1].to_string();

        match san_type.as_str() {
            "DNS" => Ok(SanEntry::Dns(value)),
            "IP" => Ok(SanEntry::Ip(value)),
            "EMAIL" => Ok(SanEntry::Email(value)),
            _ => Err(FluxError::InvalidSanFormat(format!("Unknown SAN type: {}", san_type))),
        }
    }

    /// Parse multiple SAN entries from comma-separated string
    pub fn parse_multiple(s: &str) -> Result<Vec<Self>> {
        s.split(',')
            .map(|entry| Self::parse(entry.trim()))
            .collect()
    }
}

/// Create a Certificate Signing Request
pub fn create_csr(
    cert_name: &str,
    key: &PKey<Private>,
    sans: &[SanEntry],
    common_name: Option<&str>,
) -> Result<X509Req> {
    let mut req_builder = X509ReqBuilder::new()
        .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

    // Set subject name
    let mut name_builder = X509NameBuilder::new()
        .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

    // Use common name if provided, otherwise use cert_name
    let cn = common_name.unwrap_or(cert_name);
    name_builder.append_entry_by_text("CN", cn)
        .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

    let name = name_builder.build();
    req_builder.set_subject_name(&name)
        .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

    // Set public key
    req_builder.set_pubkey(key)
        .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

    // Add Subject Alternative Names
    if !sans.is_empty() {
        let mut san_ext = SubjectAlternativeName::new();

        for san in sans {
            match san {
                SanEntry::Dns(dns) => {
                    san_ext.dns(dns);
                }
                SanEntry::Ip(ip) => {
                    san_ext.ip(ip);
                }
                SanEntry::Email(email) => {
                    san_ext.email(email);
                }
            }
        }

        let san_extension = san_ext.build(&req_builder.x509v3_context(None))
            .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

        // Create extension stack
        let mut extensions = openssl::stack::Stack::new()
            .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;
        extensions.push(san_extension)
            .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

        req_builder.add_extensions(&extensions)
            .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;
    }

    // Sign the request
    req_builder.sign(key, MessageDigest::sha256())
        .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

    Ok(req_builder.build())
}

/// Save CSR to file in PEM format
pub fn save_csr<P: AsRef<Path>>(csr: &X509Req, path: P) -> Result<()> {
    let pem_bytes = csr.to_pem()
        .map_err(|e| FluxError::CsrGenerationFailed(e.to_string()))?;

    std::fs::write(path.as_ref(), &pem_bytes)
        .map_err(|e| FluxError::FileWriteFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    Ok(())
}

/// Load CSR from file
pub fn load_csr<P: AsRef<Path>>(path: P) -> Result<X509Req> {
    let pem_bytes = std::fs::read(path.as_ref())
        .map_err(|e| FluxError::FileReadFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    let csr = X509Req::from_pem(&pem_bytes)
        .map_err(|_e| FluxError::CsrReadFailed(path.as_ref().to_path_buf()))?;

    Ok(csr)
}

/// Get subject from CSR
pub fn get_csr_subject(csr: &X509Req) -> Result<String> {
    let subject = csr.subject_name();
    let cn = subject.entries()
        .find(|entry| {
            entry.object().nid() == openssl::nid::Nid::COMMONNAME
        })
        .and_then(|entry| entry.data().as_utf8().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| FluxError::CsrGenerationFailed("No CN found in CSR".to_string()))?;

    Ok(cn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key::generate_rsa_key;

    #[test]
    fn test_san_entry_parse() {
        let dns = SanEntry::parse("DNS:example.com").unwrap();
        assert_eq!(dns, SanEntry::Dns("example.com".to_string()));

        let ip = SanEntry::parse("IP:192.168.1.1").unwrap();
        assert_eq!(ip, SanEntry::Ip("192.168.1.1".to_string()));

        let email = SanEntry::parse("EMAIL:test@example.com").unwrap();
        assert_eq!(email, SanEntry::Email("test@example.com".to_string()));
    }

    #[test]
    fn test_san_entry_parse_multiple() {
        let sans = SanEntry::parse_multiple("DNS:example.com,IP:192.168.1.1").unwrap();
        assert_eq!(sans.len(), 2);
        assert_eq!(sans[0], SanEntry::Dns("example.com".to_string()));
        assert_eq!(sans[1], SanEntry::Ip("192.168.1.1".to_string()));
    }

    #[test]
    fn test_create_csr() {
        let key = generate_rsa_key(2048, None).unwrap();
        let sans = vec![
            SanEntry::Dns("example.com".to_string()),
            SanEntry::Ip("192.168.1.1".to_string()),
        ];

        let csr = create_csr("test", &key, &sans, None).unwrap();
        assert!(csr.verify(&key).unwrap());
    }

    #[test]
    fn test_save_and_load_csr() {
        let temp_dir = tempfile::tempdir().unwrap();
        let csr_path = temp_dir.path().join("test.csr");

        let key = generate_rsa_key(2048, None).unwrap();
        let sans = vec![SanEntry::Dns("example.com".to_string())];
        let csr = create_csr("test", &key, &sans, None).unwrap();

        save_csr(&csr, &csr_path).unwrap();
        let loaded_csr = load_csr(&csr_path).unwrap();

        assert!(loaded_csr.verify(&key).unwrap());
    }
}

//! Certificate signing and management

use crate::error::{FluxError, Result};
use openssl::x509::{X509, X509Req, X509Builder};
use openssl::pkey::{PKey, Private};
use openssl::hash::MessageDigest;
use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use std::path::Path;

/// Sign a CSR with the CA key
pub fn sign_csr(
    csr: &X509Req,
    ca_cert: &X509,
    ca_key: &PKey<Private>,
    days: u32,
) -> Result<X509> {
    let mut cert_builder = X509Builder::new()
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    // Set version to X509v3
    cert_builder.set_version(2)
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    // Generate random serial number
    let mut serial = BigNum::new()
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;
    serial.rand(159, MsbOption::MAYBE_ZERO, false)
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;
    let serial_asn1 = serial.to_asn1_integer()
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;
    cert_builder.set_serial_number(&serial_asn1)
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    // Set subject from CSR
    cert_builder.set_subject_name(csr.subject_name())
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    // Set issuer from CA certificate
    cert_builder.set_issuer_name(ca_cert.subject_name())
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    // Set public key from CSR
    let pubkey = csr.public_key()
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;
    cert_builder.set_pubkey(&pubkey)
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    // Set validity period
    let not_before = Asn1Time::days_from_now(0)
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;
    cert_builder.set_not_before(&not_before)
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    let not_after = Asn1Time::days_from_now(days)
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;
    cert_builder.set_not_after(&not_after)
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    // Copy extensions from CSR to certificate
    if let Ok(extensions) = csr.extensions() {
        for ext in extensions {
            cert_builder.append_extension(ext)
                .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;
        }
    }

    // Sign the certificate
    cert_builder.sign(ca_key, MessageDigest::sha256())
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    Ok(cert_builder.build())
}

/// Save certificate to file in PEM format
pub fn save_cert_pem<P: AsRef<Path>>(cert: &X509, path: P) -> Result<()> {
    let pem_bytes = cert.to_pem()
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    std::fs::write(path.as_ref(), &pem_bytes)
        .map_err(|e| FluxError::FileWriteFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    Ok(())
}

/// Save certificate to file in DER format (CRT)
pub fn save_cert_der<P: AsRef<Path>>(cert: &X509, path: P) -> Result<()> {
    let der_bytes = cert.to_der()
        .map_err(|e| FluxError::CertSigningFailed(e.to_string()))?;

    std::fs::write(path.as_ref(), &der_bytes)
        .map_err(|e| FluxError::FileWriteFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    Ok(())
}

/// Load certificate from PEM file
pub fn load_cert<P: AsRef<Path>>(path: P) -> Result<X509> {
    let pem_bytes = std::fs::read(path.as_ref())
        .map_err(|e| FluxError::FileReadFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    let cert = X509::from_pem(&pem_bytes)
        .map_err(|e| FluxError::CertParseError(e.to_string()))?;

    Ok(cert)
}

/// Get certificate information as a formatted string
pub fn get_cert_info(cert: &X509) -> Result<String> {
    let mut info = String::new();

    // Subject
    let subject = cert.subject_name();
    info.push_str(&format!("Subject: {:?}\n", subject));

    // Issuer
    let issuer = cert.issuer_name();
    info.push_str(&format!("Issuer: {:?}\n", issuer));

    // Serial number
    let serial = cert.serial_number();
    let serial_hex = serial.to_bn()
        .map_err(|e| FluxError::CertParseError(e.to_string()))?
        .to_hex_str()
        .map_err(|e| FluxError::CertParseError(e.to_string()))?;
    info.push_str(&format!("Serial: {}\n", serial_hex));

    // Validity
    let not_before = cert.not_before();
    let not_after = cert.not_after();
    info.push_str(&format!("Not Before: {}\n", not_before));
    info.push_str(&format!("Not After: {}\n", not_after));

    // Subject Alternative Names
    if let Some(san_ext) = cert.subject_alt_names() {
        info.push_str("Subject Alternative Names:\n");
        for san in san_ext {
            if let Some(dns) = san.dnsname() {
                info.push_str(&format!("  DNS: {}\n", dns));
            }
            if let Some(ip) = san.ipaddress() {
                let ip_str = ip.iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(".");
                info.push_str(&format!("  IP: {}\n", ip_str));
            }
        }
    }

    Ok(info)
}

/// Check if certificate is expired
pub fn is_cert_expired(cert: &X509) -> Result<bool> {
    let now = Asn1Time::days_from_now(0)
        .map_err(|e| FluxError::CertParseError(e.to_string()))?;

    let not_after = cert.not_after();

    // Compare returns Ordering
    Ok(not_after < now)
}

/// Get days until expiration (negative if already expired)
pub fn days_until_expiration(cert: &X509) -> Result<i64> {
    let now = Asn1Time::days_from_now(0)
        .map_err(|e| FluxError::CertParseError(e.to_string()))?;

    let not_after = cert.not_after();

    // Calculate difference in days
    let diff = not_after.diff(&now)
        .map_err(|e| FluxError::CertParseError(e.to_string()))?;

    Ok(diff.days as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key::generate_rsa_key;
    use crate::crypto::csr::{create_csr, SanEntry};

    fn create_test_ca() -> (X509, PKey<Private>) {
        let key = generate_rsa_key(2048, None).unwrap();

        let mut cert_builder = X509Builder::new().unwrap();
        cert_builder.set_version(2).unwrap();

        let mut serial = BigNum::new().unwrap();
        serial.rand(159, MsbOption::MAYBE_ZERO, false).unwrap();
        let serial_asn1 = serial.to_asn1_integer().unwrap();
        cert_builder.set_serial_number(&serial_asn1).unwrap();

        let mut name_builder = openssl::x509::X509NameBuilder::new().unwrap();
        name_builder.append_entry_by_text("CN", "Test CA").unwrap();
        let name = name_builder.build();

        cert_builder.set_subject_name(&name).unwrap();
        cert_builder.set_issuer_name(&name).unwrap();
        cert_builder.set_pubkey(&key).unwrap();

        let not_before = Asn1Time::days_from_now(0).unwrap();
        cert_builder.set_not_before(&not_before).unwrap();
        let not_after = Asn1Time::days_from_now(365).unwrap();
        cert_builder.set_not_after(&not_after).unwrap();

        cert_builder.sign(&key, MessageDigest::sha256()).unwrap();

        (cert_builder.build(), key)
    }

    #[test]
    fn test_sign_csr() {
        let (ca_cert, ca_key) = create_test_ca();
        let key = generate_rsa_key(2048, None).unwrap();
        let sans = vec![SanEntry::Dns("example.com".to_string())];
        let csr = create_csr("test", &key, &sans, None).unwrap();

        let cert = sign_csr(&csr, &ca_cert, &ca_key, 365).unwrap();
        assert!(cert.verify(&ca_key).unwrap());
    }

    #[test]
    fn test_save_and_load_cert() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cert_path = temp_dir.path().join("test.pem");

        let (ca_cert, ca_key) = create_test_ca();
        let key = generate_rsa_key(2048, None).unwrap();
        let sans = vec![SanEntry::Dns("example.com".to_string())];
        let csr = create_csr("test", &key, &sans, None).unwrap();
        let cert = sign_csr(&csr, &ca_cert, &ca_key, 365).unwrap();

        save_cert_pem(&cert, &cert_path).unwrap();
        let loaded_cert = load_cert(&cert_path).unwrap();

        assert!(loaded_cert.verify(&ca_key).unwrap());
    }

    #[test]
    fn test_is_cert_expired() {
        let (ca_cert, _) = create_test_ca();
        assert!(!is_cert_expired(&ca_cert).unwrap());
    }
}

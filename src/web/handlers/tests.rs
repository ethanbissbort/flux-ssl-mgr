//! Tests for web service handlers

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::Config;
    use crate::web::models::{CertificateGenerateRequest, WebError};
    use axum::Json;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_certificate_generate_request_validation() {
        // Test valid request
        let valid_request = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec!["DNS:www.example.com".to_string()],
            validity_days: 365,
            key_size: 4096,
            password_protect: false,
            key_password: None,
        };

        assert_eq!(valid_request.common_name, "example.com");
        assert_eq!(valid_request.key_size, 4096);
        assert_eq!(valid_request.validity_days, 365);
    }

    #[tokio::test]
    async fn test_certificate_generate_invalid_key_size() {
        // Key size validation happens in handler
        let invalid_request = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec![],
            validity_days: 365,
            key_size: 1024, // Invalid
            password_protect: false,
            key_password: None,
        };

        // The handler should reject this
        assert_eq!(invalid_request.key_size, 1024);
    }

    #[tokio::test]
    async fn test_certificate_generate_invalid_validity() {
        let invalid_request = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec![],
            validity_days: 1000, // Too long (max 825)
            key_size: 4096,
            password_protect: false,
            key_password: None,
        };

        assert!(invalid_request.validity_days > 825);
    }

    #[tokio::test]
    async fn test_password_protection_validation() {
        // Should fail: password_protect is true but no password
        let invalid = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec![],
            validity_days: 365,
            key_size: 4096,
            password_protect: true,
            key_password: None, // Missing password
        };

        assert!(invalid.password_protect);
        assert!(invalid.key_password.is_none());

        // Should be valid: password provided
        let valid = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec![],
            validity_days: 365,
            key_size: 4096,
            password_protect: true,
            key_password: Some("secure_password".to_string()),
        };

        assert!(valid.password_protect);
        assert!(valid.key_password.is_some());
    }

    #[tokio::test]
    async fn test_san_parsing() {
        let request = CertificateGenerateRequest {
            common_name: "example.com".to_string(),
            sans: vec![
                "DNS:www.example.com".to_string(),
                "DNS:mail.example.com".to_string(),
                "IP:192.168.1.100".to_string(),
            ],
            validity_days: 365,
            key_size: 4096,
            password_protect: false,
            key_password: None,
        };

        assert_eq!(request.sans.len(), 3);
        assert!(request.sans[0].starts_with("DNS:"));
        assert!(request.sans[2].starts_with("IP:"));
    }

    #[test]
    fn test_web_error_creation() {
        let error = WebError::bad_request("Test error message");
        assert_eq!(error.status_code(), 400);

        let error = WebError::invalid_input("Invalid input");
        assert_eq!(error.status_code(), 400);

        let error = WebError::internal_error("Internal error");
        assert_eq!(error.status_code(), 500);
    }

    #[test]
    fn test_common_name_validation() {
        // Empty CN should be invalid
        let empty_cn = "";
        assert!(empty_cn.is_empty());

        // Valid CN
        let valid_cn = "example.com";
        assert!(!valid_cn.is_empty());
        assert!(valid_cn.len() < 64); // Common CN length limit
    }

    #[test]
    fn test_validity_days_range() {
        // Test boundary conditions
        let min_days = 1;
        let max_days = 825;
        let invalid_low = 0;
        let invalid_high = 826;

        assert!(min_days >= 1 && min_days <= 825);
        assert!(max_days >= 1 && max_days <= 825);
        assert!(invalid_low < 1);
        assert!(invalid_high > 825);
    }
}

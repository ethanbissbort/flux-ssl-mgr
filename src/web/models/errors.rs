use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// API error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "BAD_REQUEST")]
    BadRequest,
    #[serde(rename = "INVALID_CSR")]
    InvalidCsr,
    #[serde(rename = "INVALID_INPUT")]
    InvalidInput,
    #[serde(rename = "INVALID_CERTIFICATE")]
    InvalidCertificate,
    #[serde(rename = "FILE_TOO_LARGE")]
    FileTooLarge,
    #[serde(rename = "UNSUPPORTED_FORMAT")]
    UnsupportedFormat,
    #[serde(rename = "CA_ERROR")]
    CaError,
    #[serde(rename = "SIGNING_FAILED")]
    SigningFailed,
    #[serde(rename = "KEY_GENERATION_FAILED")]
    KeyGenerationFailed,
    #[serde(rename = "INTERNAL_ERROR")]
    InternalError,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::BadRequest => write!(f, "BAD_REQUEST"),
            ErrorCode::InvalidCsr => write!(f, "INVALID_CSR"),
            ErrorCode::InvalidInput => write!(f, "INVALID_INPUT"),
            ErrorCode::InvalidCertificate => write!(f, "INVALID_CERTIFICATE"),
            ErrorCode::FileTooLarge => write!(f, "FILE_TOO_LARGE"),
            ErrorCode::UnsupportedFormat => write!(f, "UNSUPPORTED_FORMAT"),
            ErrorCode::CaError => write!(f, "CA_ERROR"),
            ErrorCode::SigningFailed => write!(f, "SIGNING_FAILED"),
            ErrorCode::KeyGenerationFailed => write!(f, "KEY_GENERATION_FAILED"),
            ErrorCode::InternalError => write!(f, "INTERNAL_ERROR"),
        }
    }
}

/// Error detail structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetail,
}

impl ErrorResponse {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: ErrorDetail {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn with_details(
        code: ErrorCode,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            success: false,
            error: ErrorDetail {
                code,
                message: message.into(),
                details: Some(details.into()),
            },
        }
    }
}

/// Web service error type
#[derive(Debug)]
pub struct WebError {
    pub status: StatusCode,
    pub response: ErrorResponse,
}

impl WebError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            response: ErrorResponse::new(code, message),
        }
    }

    pub fn with_details(
        status: StatusCode,
        code: ErrorCode,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            status,
            response: ErrorResponse::with_details(code, message, details),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, message)
    }

    pub fn invalid_csr(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::InvalidCsr, message)
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::InvalidInput, message)
    }

    pub fn invalid_certificate(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidCertificate,
            message,
        )
    }

    pub fn file_too_large(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            ErrorCode::FileTooLarge,
            message,
        )
    }

    pub fn unsupported_format(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ErrorCode::UnsupportedFormat,
            message,
        )
    }

    pub fn ca_error(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::CaError,
            message,
        )
    }

    pub fn signing_failed(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::SigningFailed,
            message,
        )
    }

    pub fn key_generation_failed(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::KeyGenerationFailed,
            message,
        )
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            message,
        )
    }

    /// Get the HTTP status code
    pub fn status_code(&self) -> u16 {
        self.status.as_u16()
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        (self.status, Json(self.response)).into_response()
    }
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.response.error.code, self.response.error.message
        )
    }
}

impl std::error::Error for WebError {}

/// Convert flux-ssl-mgr errors to web errors
impl From<crate::error::FluxError> for WebError {
    fn from(err: crate::error::FluxError) -> Self {
        use crate::error::FluxError;

        match err {
            FluxError::CaKeyNotFound(_) | FluxError::CaCertNotFound(_) => {
                WebError::ca_error(err.to_string())
            }
            FluxError::InvalidSanFormat(_) => WebError::invalid_input(err.to_string()),
            FluxError::OpenSslError(_) => WebError::signing_failed(err.to_string()),
            FluxError::ConfigError(_) => WebError::internal_error(err.to_string()),
            FluxError::IoError(_) => WebError::internal_error(err.to_string()),
            _ => WebError::internal_error(err.to_string()),
        }
    }
}

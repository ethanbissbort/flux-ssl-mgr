# Flux SSL Manager - Web Service Roadmap

## Overview

This document outlines the implementation plan for adding a web-based interface to Flux SSL Manager. The web service will provide a user-friendly interface for certificate management operations, making the tool accessible to users who prefer a graphical interface over CLI.

**Status**: In Development
**Target Version**: 2.5.0
**Started**: December 2025

---

## Objectives

### Primary Goals

1. **CSR Upload & Signing**: Accept CSR file uploads and return signed certificates
2. **Manual Certificate Request**: Interactive form for certificate generation without pre-existing CSR
3. **Certificate Information Display**: Upload certificates to view detailed information

### Secondary Goals

- RESTful API for automation and integration
- Modern, responsive web interface
- Secure authentication and authorization
- Comprehensive error handling and validation
- Audit logging for all operations
- API documentation (OpenAPI/Swagger)

---

## Architecture

### Technology Stack

#### Backend (Rust)
- **Web Framework**: `axum` (tokio-based, composable, type-safe)
- **Async Runtime**: `tokio` (already used in the ecosystem)
- **Serialization**: `serde` + `serde_json` (already in use)
- **File Uploads**: `axum-multipart` or `multer`
- **Templates**: `askama` or `tera` (Rust templating engines)
- **Static Files**: `tower-http` with `ServeDir`
- **Session Management**: `tower-sessions` or `axum-sessions`
- **Authentication**: `jsonwebtoken` for JWT or session-based auth
- **Validation**: `validator` crate for input validation

#### Frontend
- **Framework**: Vanilla HTML5/CSS3/JavaScript initially
- **UI Library**: Simple, modern CSS (potentially Bootstrap or Tailwind CSS)
- **Future Enhancement**: React/Vue.js/Svelte for SPA experience
- **File Upload**: HTML5 `FormData` with drag-and-drop support

#### API
- **Style**: RESTful JSON API
- **Documentation**: OpenAPI 3.0 specification
- **Response Format**: JSON with consistent structure
- **Error Handling**: RFC 7807 Problem Details

### Directory Structure

```
flux-ssl-mgr/
├── src/
│   ├── web/                    # Web service modules
│   │   ├── mod.rs              # Web module exports
│   │   ├── server.rs           # Axum server setup
│   │   ├── routes/             # Route handlers
│   │   │   ├── mod.rs
│   │   │   ├── api.rs          # API routes (/api/*)
│   │   │   ├── csr.rs          # CSR upload endpoints
│   │   │   ├── cert.rs         # Certificate endpoints
│   │   │   └── info.rs         # Certificate info endpoints
│   │   ├── handlers/           # Request handlers
│   │   │   ├── mod.rs
│   │   │   ├── csr_handler.rs
│   │   │   ├── cert_handler.rs
│   │   │   └── info_handler.rs
│   │   ├── models/             # Request/response models
│   │   │   ├── mod.rs
│   │   │   ├── requests.rs     # API request types
│   │   │   ├── responses.rs    # API response types
│   │   │   └── errors.rs       # Error responses
│   │   ├── middleware/         # Custom middleware
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs         # Authentication
│   │   │   ├── logging.rs      # Request logging
│   │   │   └── cors.rs         # CORS handling
│   │   └── templates/          # Askama templates
│   │       ├── base.html
│   │       ├── index.html
│   │       ├── csr_upload.html
│   │       ├── cert_request.html
│   │       └── cert_info.html
│   └── ...
├── static/                     # Static web assets
│   ├── css/
│   │   └── styles.css
│   ├── js/
│   │   └── app.js
│   └── images/
├── templates/                  # Additional templates
└── ...
```

---

## Implementation Phases

### Phase 1: Foundation & Infrastructure ✅ (In Progress)

**Estimated Effort**: 1-2 weeks

#### Tasks

- [x] Research and select web framework (Decision: axum)
- [ ] Add web dependencies to Cargo.toml
- [ ] Create basic web module structure
- [ ] Set up axum server with basic routes
- [ ] Configure static file serving
- [ ] Implement health check endpoint
- [ ] Set up logging and tracing
- [ ] Create basic HTML templates
- [ ] Add configuration for web service (port, bind address, etc.)

#### Deliverables

- Basic web server running on configurable port
- Health check endpoint: `GET /health`
- Index page with navigation
- Static asset serving
- Request logging

---

### Phase 2: CSR Upload & Signing 🔄 (Current Phase)

**Estimated Effort**: 1-2 weeks

#### Tasks

- [ ] Design CSR upload API
  - [ ] Define request format (multipart/form-data)
  - [ ] Define response format (JSON with cert data)
  - [ ] Error handling specifications
- [ ] Implement CSR upload endpoint
  - [ ] File upload handling
  - [ ] CSR validation
  - [ ] Integration with existing crypto module
  - [ ] Certificate signing logic
- [ ] Create web UI for CSR upload
  - [ ] File upload form
  - [ ] Drag-and-drop support
  - [ ] Progress indicator
  - [ ] Success/error display
  - [ ] Certificate download
- [ ] Add SAN input support
  - [ ] Form fields for additional SANs
  - [ ] Validation
- [ ] Testing
  - [ ] Unit tests for handlers
  - [ ] Integration tests
  - [ ] Error case coverage

#### API Endpoints

```
POST /api/csr/upload
  Request:
    - Content-Type: multipart/form-data
    - Body:
      - csr_file: File (required)
      - sans: String[] (optional, additional SANs)
      - validity_days: Integer (optional, default from config)

  Response (Success):
    - Status: 200 OK
    - Body: {
        "success": true,
        "certificate": {
          "pem": "-----BEGIN CERTIFICATE-----...",
          "subject": "CN=example.com",
          "issuer": "CN=Intermediate CA",
          "serial": "0A1B2C3D",
          "not_before": "2025-12-05T00:00:00Z",
          "not_after": "2026-12-05T23:59:59Z",
          "sans": ["DNS:example.com", "DNS:www.example.com"]
        }
      }

  Response (Error):
    - Status: 400 Bad Request / 500 Internal Server Error
    - Body: {
        "success": false,
        "error": {
          "code": "INVALID_CSR",
          "message": "Invalid CSR format",
          "details": "..."
        }
      }
```

#### UI Pages

- `/csr-upload` - CSR upload interface
- `/csr-result` - Display signed certificate and download links

---

### Phase 3: Manual Certificate Request 📋 (Planned)

**Estimated Effort**: 1-2 weeks

#### Tasks

- [ ] Design manual request API
  - [ ] Input validation rules
  - [ ] Request/response models
- [ ] Implement certificate request endpoint
  - [ ] Form data processing
  - [ ] Key generation
  - [ ] CSR creation
  - [ ] Certificate signing
  - [ ] Bundle creation (cert + key)
- [ ] Create web UI form
  - [ ] Input fields (CN, SANs, validity)
  - [ ] Key size selection
  - [ ] Password protection option
  - [ ] Real-time validation
  - [ ] Preview before generation
- [ ] Add certificate bundle download
  - [ ] ZIP file creation
  - [ ] Include cert, key, CA chain
- [ ] Testing

#### API Endpoints

```
POST /api/cert/generate
  Request:
    - Content-Type: application/json
    - Body: {
        "common_name": "example.com",
        "sans": [
          "DNS:www.example.com",
          "DNS:mail.example.com",
          "IP:192.168.1.100"
        ],
        "validity_days": 375,
        "key_size": 4096,
        "password_protect": false,
        "key_password": "optional_password"
      }

  Response (Success):
    - Status: 200 OK
    - Body: {
        "success": true,
        "certificate": {
          "pem": "-----BEGIN CERTIFICATE-----...",
          "private_key": "-----BEGIN PRIVATE KEY-----...",
          "ca_chain": "-----BEGIN CERTIFICATE-----...",
          "subject": "CN=example.com",
          "serial": "0A1B2C3D",
          "not_before": "2025-12-05T00:00:00Z",
          "not_after": "2026-12-05T23:59:59Z",
          "sans": ["DNS:example.com", "DNS:www.example.com"],
          "download_url": "/api/cert/download/SESSION_ID"
        }
      }

GET /api/cert/download/{session_id}
  Response:
    - Content-Type: application/zip
    - Content-Disposition: attachment; filename="example.com.zip"
    - Body: ZIP file containing:
      - example.com.cert.pem
      - example.com.key.pem
      - ca-chain.cert.pem
      - README.txt (usage instructions)
```

#### UI Pages

- `/cert-request` - Manual certificate request form
- `/cert-result` - Display generated certificate and download

---

### Phase 4: Certificate Information Display 🔍 (Planned)

**Estimated Effort**: 1 week

#### Tasks

- [ ] Design certificate info API
  - [ ] Response format for certificate details
  - [ ] Support for various cert formats (PEM, DER, CRT)
- [ ] Implement certificate parsing endpoint
  - [ ] File upload handling
  - [ ] Certificate parsing
  - [ ] Information extraction
  - [ ] Chain validation (if provided)
- [ ] Create web UI for cert display
  - [ ] File upload interface
  - [ ] Certificate details display
  - [ ] Visual certificate chain
  - [ ] Expiration warnings
  - [ ] Export/download options
- [ ] Add certificate validation
  - [ ] Chain verification
  - [ ] Expiration checking
  - [ ] Revocation status (future)
- [ ] Testing

#### API Endpoints

```
POST /api/cert/info
  Request:
    - Content-Type: multipart/form-data
    - Body:
      - cert_file: File (required)
      - verify_chain: Boolean (optional, default: false)

  Response (Success):
    - Status: 200 OK
    - Body: {
        "success": true,
        "certificate": {
          "version": 3,
          "serial_number": "0A1B2C3D4E5F",
          "signature_algorithm": "sha256WithRSAEncryption",
          "issuer": {
            "CN": "Intermediate CA",
            "O": "Flux Lab",
            "C": "US"
          },
          "validity": {
            "not_before": "2025-12-05T00:00:00Z",
            "not_after": "2026-12-05T23:59:59Z",
            "days_remaining": 365,
            "is_expired": false,
            "is_expiring_soon": false
          },
          "subject": {
            "CN": "example.com",
            "O": "Example Corp"
          },
          "subject_alternative_names": [
            "DNS:example.com",
            "DNS:www.example.com",
            "IP:192.168.1.100"
          ],
          "public_key": {
            "algorithm": "RSA",
            "size": 4096,
            "exponent": 65537
          },
          "extensions": [
            {
              "oid": "2.5.29.15",
              "name": "keyUsage",
              "critical": true,
              "value": "Digital Signature, Key Encipherment"
            }
          ],
          "fingerprints": {
            "sha1": "A1:B2:C3:D4:E5:F6...",
            "sha256": "1A2B3C4D5E6F..."
          },
          "pem": "-----BEGIN CERTIFICATE-----..."
        }
      }
```

#### UI Pages

- `/cert-info` - Certificate information viewer
- `/cert-verify` - Certificate chain verification tool

---

### Phase 5: Security & Authentication 🔒 (Planned)

**Estimated Effort**: 1-2 weeks

#### Tasks

- [ ] Design authentication system
  - [ ] User model
  - [ ] Role-based access control (RBAC)
  - [ ] Session management
- [ ] Implement authentication
  - [ ] Login/logout endpoints
  - [ ] Password hashing (argon2)
  - [ ] JWT token generation
  - [ ] Session storage
- [ ] Add authorization middleware
  - [ ] Route protection
  - [ ] Role checking
  - [ ] Rate limiting
- [ ] Security enhancements
  - [ ] CSRF protection
  - [ ] Input sanitization
  - [ ] File upload restrictions
  - [ ] Request size limits
- [ ] Audit logging
  - [ ] Log all certificate operations
  - [ ] User action tracking
  - [ ] Structured logging to file/database
- [ ] Testing

#### Features

- User authentication (optional, configurable)
- API key authentication for programmatic access
- Rate limiting to prevent abuse
- HTTPS/TLS support
- Security headers (CSP, HSTS, etc.)
- Audit trail of all operations

---

### Phase 6: Enhanced UI/UX 🎨 (Planned)

**Estimated Effort**: 1-2 weeks

#### Tasks

- [ ] Improve visual design
  - [ ] Modern CSS framework (Tailwind/Bootstrap)
  - [ ] Responsive design
  - [ ] Dark mode support
  - [ ] Accessibility (WCAG 2.1)
- [ ] Add interactive features
  - [ ] Real-time validation
  - [ ] Form auto-save
  - [ ] Keyboard shortcuts
  - [ ] Tooltips and help text
- [ ] Create dashboard
  - [ ] Certificate overview
  - [ ] Expiration warnings
  - [ ] Recent activity
  - [ ] Quick actions
- [ ] Add certificate management
  - [ ] List all certificates
  - [ ] Search and filter
  - [ ] Bulk operations
  - [ ] Export to various formats
- [ ] Testing
  - [ ] Cross-browser testing
  - [ ] Mobile responsiveness
  - [ ] Accessibility audit

#### UI Components

- Dashboard with stats and recent certificates
- Certificate list with search/filter
- Certificate detail view
- User profile and settings
- Activity log viewer
- Help and documentation

---

### Phase 7: API Documentation & Testing 📚 (Planned)

**Estimated Effort**: 1 week

#### Tasks

- [ ] Create OpenAPI specification
  - [ ] Document all endpoints
  - [ ] Request/response schemas
  - [ ] Authentication schemes
  - [ ] Error codes
- [ ] Set up Swagger UI
  - [ ] Interactive API documentation
  - [ ] Try-it-out functionality
- [ ] Write API examples
  - [ ] cURL examples
  - [ ] Python client examples
  - [ ] JavaScript/Node.js examples
- [ ] Comprehensive testing
  - [ ] Unit tests for all handlers
  - [ ] Integration tests
  - [ ] API contract tests
  - [ ] Load testing
  - [ ] Security testing
- [ ] Documentation
  - [ ] User guide
  - [ ] API reference
  - [ ] Deployment guide
  - [ ] Troubleshooting

#### Deliverables

- OpenAPI 3.0 specification
- Swagger UI at `/api/docs`
- API client libraries (optional)
- Postman collection
- Comprehensive test suite
- User and developer documentation

---

## Configuration

### Web Service Configuration

Add to `config.toml`:

```toml
[web]
enabled = true
bind_address = "127.0.0.1"
port = 8443
tls_enabled = true
tls_cert_path = "/path/to/server.cert.pem"
tls_key_path = "/path/to/server.key.pem"
workers = 4
request_timeout = 30
max_request_size = 10485760  # 10MB

[web.auth]
enabled = false
jwt_secret = "change-me-in-production"
session_timeout = 3600  # 1 hour
require_https = true

[web.limits]
rate_limit = 100  # requests per minute
max_file_size = 5242880  # 5MB
max_concurrent_uploads = 10

[web.cors]
enabled = true
allowed_origins = ["https://example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["Content-Type", "Authorization"]

[web.logging]
access_log = "/var/log/flux-ssl-mgr/access.log"
audit_log = "/var/log/flux-ssl-mgr/audit.log"
log_level = "info"
```

---

## API Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `SUCCESS` | 200 | Operation successful |
| `CREATED` | 201 | Resource created |
| `BAD_REQUEST` | 400 | Invalid request |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource conflict |
| `INVALID_CSR` | 400 | CSR validation failed |
| `INVALID_INPUT` | 400 | Input validation failed |
| `FILE_TOO_LARGE` | 413 | Uploaded file exceeds limit |
| `UNSUPPORTED_FORMAT` | 415 | Unsupported file format |
| `CA_ERROR` | 500 | CA operation failed |
| `SIGNING_FAILED` | 500 | Certificate signing failed |
| `INTERNAL_ERROR` | 500 | Internal server error |

---

## Testing Strategy

### Unit Tests
- Handler logic
- Input validation
- Error handling
- Response formatting

### Integration Tests
- End-to-end API workflows
- Multi-part file uploads
- Certificate generation pipeline
- Error scenarios

### Security Tests
- Authentication bypass attempts
- Authorization checks
- Input injection (XSS, SQL injection)
- File upload exploits
- Rate limiting

### Performance Tests
- Concurrent request handling
- Large file uploads
- Certificate generation under load
- Memory usage profiling

### Browser Tests
- Cross-browser compatibility
- Mobile responsiveness
- Accessibility compliance
- JavaScript functionality

---

## Security Considerations

### Input Validation
- Validate all user inputs
- Sanitize file uploads
- Limit file sizes
- Check file types (magic numbers)
- Prevent path traversal

### Authentication & Authorization
- Secure password storage (Argon2)
- JWT token security
- CSRF protection
- Session management
- API key rotation

### Network Security
- HTTPS/TLS enforcement
- Secure headers (CSP, HSTS, etc.)
- CORS configuration
- Rate limiting
- DDoS protection

### Data Protection
- Temporary file cleanup
- Secure certificate storage
- Memory zeroing for keys
- Audit logging
- Privacy compliance

### Best Practices
- Follow OWASP Top 10
- Regular security audits
- Dependency scanning
- Security headers
- Content Security Policy

---

## Deployment

### Systemd Service

```ini
[Unit]
Description=Flux SSL Manager Web Service
After=network.target

[Service]
Type=simple
User=flux-ssl-mgr
Group=flux-ssl-mgr
WorkingDirectory=/opt/flux-ssl-mgr
ExecStart=/usr/local/bin/flux-ssl-mgr serve --config /etc/flux-ssl-mgr/config.toml
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### Docker Support

```dockerfile
FROM rust:1.70 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/flux-ssl-mgr /usr/local/bin/
COPY --from=builder /app/static /opt/flux-ssl-mgr/static
EXPOSE 8443
CMD ["flux-ssl-mgr", "serve"]
```

### Reverse Proxy (Nginx)

```nginx
server {
    listen 443 ssl http2;
    server_name cert-manager.example.com;

    ssl_certificate /etc/ssl/certs/cert-manager.crt;
    ssl_certificate_key /etc/ssl/private/cert-manager.key;

    location / {
        proxy_pass https://127.0.0.1:8443;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

---

## Progress Tracking

### Completed ✅
- [x] Project planning and roadmap creation
- [x] Technology stack selection
- [x] Architecture design

### In Progress 🔄
- [ ] Phase 1: Foundation & Infrastructure (75%)
  - [x] Research web frameworks
  - [x] Design architecture
  - [ ] Implement basic server
  - [ ] Create templates

### Planned 📋
- [ ] Phase 2: CSR Upload & Signing
- [ ] Phase 3: Manual Certificate Request
- [ ] Phase 4: Certificate Information Display
- [ ] Phase 5: Security & Authentication
- [ ] Phase 6: Enhanced UI/UX
- [ ] Phase 7: API Documentation & Testing

---

## Future Enhancements

### Short-term
- Certificate renewal via web interface
- Certificate revocation UI
- Batch certificate operations
- Certificate templates
- Export to various formats (PKCS12, JKS)

### Medium-term
- Multi-CA management
- Certificate lifecycle dashboard
- Monitoring and alerting integration
- Webhook notifications
- Advanced search and filtering

### Long-term
- Full SPA with React/Vue
- Mobile app (PWA)
- GraphQL API
- WebSocket for real-time updates
- Certificate deployment automation
- Integration with cloud providers
- ACME protocol support
- Service mesh integration

---

## Resources

### Documentation
- [Axum Documentation](https://docs.rs/axum/)
- [Tokio Documentation](https://tokio.rs/)
- [OpenAPI Specification](https://swagger.io/specification/)
- [OWASP Web Security](https://owasp.org/www-project-top-ten/)

### Tools
- Postman - API testing
- Swagger UI - API documentation
- curl - Command-line testing
- cargo-tarpaulin - Code coverage

### Related Projects
- cert-manager (Kubernetes)
- Let's Encrypt
- Vault (HashiCorp)
- CFSSL (CloudFlare)

---

## Changelog

### 2025-12-05
- Initial roadmap creation
- Defined 7 implementation phases
- Outlined API endpoints and data models
- Created architecture and technology stack
- Added security considerations
- Defined testing strategy

---

## Feedback & Contributions

This is a living document. Feedback and contributions are welcome!

**GitHub Issues**: https://github.com/ethanbissbort/flux-ssl-mgr/issues
**Discussions**: https://github.com/ethanbissbort/flux-ssl-mgr/discussions

---

**Last Updated**: 2025-12-05
**Status**: Active Development
**Target Completion**: Q2 2026

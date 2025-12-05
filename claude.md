# Flux SSL Manager - Technical Documentation

## Project Overview

**flux-ssl-mgr** is a secure certificate management tool designed for homestead/homelab internal PKI environments. It automates the generation, signing, and management of SSL/TLS certificates using an intermediate Certificate Authority (CA).

### Current State
- **Language**: Rust (edition 2021)
- **Version**: 2.0.0
- **Status**: Production-ready with core features complete
- **Previous Version**: Bash shell script (archived in `old/` directory)

### Purpose
This tool simplifies certificate lifecycle management in a homelab PKI setup by:
- Generating RSA private keys (2048-4096 bit, configurable)
- Creating Certificate Signing Requests (CSRs) with Subject Alternative Names (SANs)
- Signing certificates using an intermediate CA
- Managing file permissions and ownership
- Supporting both single and batch certificate processing with parallelization
- Providing secure password handling for private keys and CA keys

## Architecture & Design

### Rust Implementation (v2.0)

#### Project Structure

```
flux-ssl-mgr/
├── Cargo.toml              # Project manifest and dependencies
├── Cargo.lock              # Locked dependency versions
├── src/
│   ├── main.rs             # CLI entry point with clap
│   ├── lib.rs              # Library root, public API exports
│   ├── config.rs           # Configuration management (TOML)
│   ├── error.rs            # Error types using thiserror
│   ├── batch.rs            # Batch processing with rayon
│   ├── interactive.rs      # Interactive prompts with dialoguer
│   ├── output.rs           # Colored terminal output
│   ├── crypto/
│   │   ├── mod.rs          # Crypto module exports
│   │   ├── key.rs          # RSA key generation and management
│   │   ├── csr.rs          # CSR creation with SAN support
│   │   └── cert.rs         # Certificate signing and validation
│   └── ca/
│       ├── mod.rs          # CA module exports
│       └── intermediate.rs # Intermediate CA operations
├── tests/                  # Integration tests (to be expanded)
├── .github/
│   └── workflows/
│       └── rust.yml        # CI/CD workflow
├── old/                    # Archived bash implementation
│   ├── flux-ssl-mgr.sh
│   └── flux-ssl-mgr-ansible.txt
├── config.toml.example     # Example configuration
├── README.md               # User documentation
├── claude.md               # This file - technical documentation
└── ROADMAP.md              # Project roadmap and future plans
```

#### Key Features

1. **Type Safety & Memory Safety**
   - Leverages Rust's ownership system to prevent memory errors
   - No buffer overflows, use-after-free, or memory leaks
   - Compile-time guarantees for thread safety

2. **Secure Password Handling**
   - `secrecy` crate wraps sensitive strings in `Secret<String>`
   - `zeroize` crate ensures memory is zeroed before deallocation
   - Passwords never appear in debug output or logs
   - Temporary CA keys automatically cleaned up via RAII

3. **Comprehensive Error Handling**
   - Custom `FluxError` enum covers all error cases
   - Rich error context with file paths and descriptions
   - Proper error propagation using `Result<T>` type
   - User-friendly error messages

4. **Dual Processing Modes**
   - **Single Mode**: Interactive or CLI certificate generation
   - **Batch Mode**: Process multiple CSRs with optional parallelization

5. **Certificate Generation Workflow**
   - Generate RSA private key (optionally password-protected with AES-256)
   - Create CSR with SANs (DNS, IP, Email)
   - Sign certificate with intermediate CA
   - Output in both PEM and CRT formats
   - Copy to output directory with proper permissions
   - Set file ownership (Unix only)

6. **Configuration Management**
   - TOML-based configuration files
   - Multiple search paths (local, user, system)
   - Serde for serialization/deserialization
   - Runtime validation of configuration

7. **User Experience**
   - Colored output using `console` crate
   - Interactive prompts using `dialoguer`
   - Progress tracking
   - Clear error messages with helpful suggestions

### Technical Components

#### 1. Configuration Module (`src/config.rs`)

Manages application configuration with TOML files.

**Key Structures:**
```rust
pub struct Config {
    pub working_dir: PathBuf,
    pub output_dir: PathBuf,
    pub csr_input_dir: PathBuf,
    pub ca_key_path: PathBuf,
    pub ca_cert_path: PathBuf,
    pub openssl_config: PathBuf,
    pub defaults: Defaults,
    pub permissions: Permissions,
    pub batch: BatchConfig,
    pub output: OutputConfig,
}
```

**Features:**
- Validates paths on load (ensures CA keys and configs exist)
- Supports multiple config file locations
- Default values using serde defaults
- Configuration serialization for saving

**Search Order:**
1. `./flux-ssl-mgr.toml`
2. `~/.config/flux-ssl-mgr/config.toml`
3. `/etc/flux-ssl-mgr/config.toml`

#### 2. Error Handling Module (`src/error.rs`)

Comprehensive error types using `thiserror`.

**Error Categories:**
- File I/O errors (read, write, not found)
- CA-related errors (key not found, unlock failed)
- Cryptographic errors (key generation, signing failures)
- Configuration errors (invalid values, missing settings)
- Interactive mode errors (user cancellation, prompt failures)

**Example:**
```rust
#[derive(Debug, Error)]
pub enum FluxError {
    #[error("CA key not found: {0}")]
    CaKeyNotFound(PathBuf),

    #[error("Invalid SAN format: {0}")]
    InvalidSanFormat(String),

    #[error("OpenSSL error: {0}")]
    OpenSslError(#[from] openssl::error::ErrorStack),
}
```

#### 3. Cryptography Module (`src/crypto/`)

Handles all cryptographic operations using OpenSSL.

**Key Generation (`key.rs`):**
- RSA key generation (configurable size: 2048, 4096, etc.)
- Password protection using AES-256-CBC
- Key encryption detection
- Secure password prompting
- Temporary CA key unlocking with automatic cleanup

**CSR Creation (`csr.rs`):**
- X.509 CSR generation
- Subject Alternative Name (SAN) support
  - DNS names
  - IP addresses
  - Email addresses
- SAN parsing from string format (`DNS:example.com,IP:192.168.1.1`)
- SHA-256 signing

**Certificate Signing (`cert.rs`):**
- CSR signing with CA key
- Random serial number generation
- Configurable validity period
- Extension copying from CSR
- Certificate information extraction
- Expiration checking and days-until-expiration calculation
- PEM and DER format support

#### 4. CA Module (`src/ca/`)

Manages intermediate Certificate Authority operations.

**IntermediateCA Structure:**
```rust
pub struct IntermediateCA {
    key: PKey<Private>,
    cert: X509,
    _temp_file: Option<tempfile::NamedTempFile>,
}
```

**Features:**
- Automatic CA key encryption detection
- Password prompting for encrypted CA keys
- Temporary unlocked key creation (RAII cleanup)
- CA certificate loading and validation
- Subject name extraction

**Security:**
- Temporary files created with mode 0600
- Automatic cleanup via `Drop` trait
- Memory zeroing for sensitive data

#### 5. Batch Processing Module (`src/batch.rs`)

Efficient processing of multiple certificates.

**Features:**
- CSR file discovery using `walkdir`
- Name pattern filtering
- Sequential or parallel processing (via `rayon`)
- Progress tracking
- Error aggregation

**Batch Result:**
```rust
pub struct BatchResult {
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<(String, String)>,
}
```

**Processing Flow:**
1. Discover CSR files in directory
2. Apply optional filters
3. Load CA once (reuse for all certificates)
4. Process certificates (parallel or sequential)
5. Collect results and errors
6. Display summary

#### 6. Interactive Module (`src/interactive.rs`)

User-friendly interactive prompts using `dialoguer`.

**Prompt Types:**
- Text input with validation
- Confirmation (yes/no)
- Single selection
- Multi-selection
- Password input with confirmation

**Validation:**
- Certificate name: alphanumeric, hyphens, underscores, dots
- SANs: proper format (TYPE:value)
- Certificate days: 1-825 (CA/Browser Forum limit)

#### 7. Output Module (`src/output.rs`)

Colored terminal output using `console`.

**Output Types:**
- Success (green ✓)
- Error (red ✗)
- Warning (yellow ⚠)
- Info (blue ℹ)
- Headers and separators
- Progress steps

**Configuration:**
- Enable/disable colors
- Verbose mode
- Quiet mode

#### 8. Main Module (`src/main.rs`)

CLI entry point using `clap` v4.

**Commands:**
- `single` - Generate single certificate
- `batch` - Process multiple CSRs
- `info` - Display certificate information
- `config` - Configuration management

**Global Options:**
- `-c, --config` - Custom config file path
- `-v, --verbose` - Enable verbose output
- `-q, --quiet` - Suppress non-error output

**Logging:**
- `tracing` crate for structured logging
- `RUST_LOG` environment variable support
- Log levels: error, warn, info, debug, trace

### Dependencies

#### CLI & User Interface
- **clap** (4.5) - Command-line argument parsing with derive macros
- **dialoguer** (0.11) - Interactive prompts
- **indicatif** (0.17) - Progress bars (for future use)
- **console** (0.15) - Terminal colors and formatting

#### Cryptography
- **openssl** (0.10) - OpenSSL bindings for crypto operations

#### Configuration & Serialization
- **serde** (1.0) - Serialization framework
- **toml** (0.8) - TOML format support
- **config** (0.14) - Layered configuration management

#### Error Handling
- **thiserror** (1.0) - Error derive macros
- **anyhow** (1.0) - Flexible error handling for applications

#### Security
- **secrecy** (0.8) - Wrapper types for sensitive data
- **zeroize** (1.7) - Secure memory zeroing

#### File Operations
- **walkdir** (2.4) - Recursive directory traversal
- **tempfile** (3.8) - Temporary file management

#### Utilities
- **chrono** (0.4) - Date and time handling
- **tracing** (0.1) - Structured logging
- **tracing-subscriber** (0.3) - Logging subscriber with env filter

#### Async/Concurrency
- **rayon** (1.8) - Data parallelism

#### Development Dependencies
- **assert_cmd** (2.0) - Command testing
- **predicates** (3.0) - Assertion helpers
- **tempfile** (3.8) - Temporary files for tests

### Security Considerations

#### Best Practices Implemented

1. **Memory Safety**
   - Rust's ownership system prevents:
     - Use-after-free
     - Double-free
     - Buffer overflows
     - Data races
   - Compile-time guarantees

2. **Secure Password Handling**
   - Passwords wrapped in `Secret<String>` (secrecy crate)
   - Memory zeroed on drop (zeroize crate)
   - Never logged or printed
   - Not included in Debug output

3. **Temporary File Security**
   - Created with mode 0600 (owner read/write only)
   - Automatic cleanup via RAII (`Drop` trait)
   - Secure random filenames (tempfile crate)
   - No temporary file leaks

4. **File Permissions**
   - Private keys: 0o400 (owner read only)
   - Certificates: 0o755 (world readable)
   - Configurable via config file

5. **Cryptographic Best Practices**
   - RSA 4096-bit keys by default
   - SHA-256 for signing
   - AES-256-CBC for key encryption
   - Cryptographically secure random number generation

6. **Input Validation**
   - Certificate names validated against allowed characters
   - SANs validated for proper format
   - Configuration validated on load
   - Path validation before use

#### Known Limitations

1. **File Ownership**
   - Currently commented out (requires `users` and `nix` crates)
   - Ownership changes require root privileges
   - Platform-specific implementation needed

2. **Platform Support**
   - Primary target: Linux
   - macOS: Should work but less tested
   - Windows: Limited support (no Unix permissions)

3. **CA Database Management**
   - Does not manage OpenSSL CA database (index.txt, serial)
   - Assumes existing PKI infrastructure
   - No certificate revocation list (CRL) management

4. **Key Algorithm Support**
   - Currently RSA only
   - ECDSA support planned for v2.1

## Testing Strategy

### Current Test Coverage

**Unit Tests:**
- `src/crypto/key.rs` - Key generation, encryption, loading
- `src/crypto/csr.rs` - SAN parsing, CSR creation
- `src/crypto/cert.rs` - Certificate signing, validation

**Test Helpers:**
- Mock CA certificate and key creation
- Temporary directory management
- Test fixtures

### Testing Checklist

- [x] RSA key generation (multiple sizes)
- [x] Password-protected key saving/loading
- [x] Key encryption detection
- [x] SAN entry parsing (DNS, IP, Email)
- [x] CSR creation with SANs
- [x] CSR saving and loading
- [x] Certificate signing
- [x] Certificate PEM/DER export
- [x] Certificate information extraction
- [x] Certificate expiration checking
- [ ] Configuration file loading
- [ ] Batch processing
- [ ] Interactive prompts (requires mocking)
- [ ] File permission setting
- [ ] CA key unlocking
- [ ] Error handling paths
- [ ] Integration tests (end-to-end)

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --lib crypto::key
cargo test --lib crypto::csr
cargo test --lib crypto::cert

# Run with output
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test

# Generate coverage report
cargo tarpaulin --out Html
```

## Development Guidelines

### Code Organization

1. **Module Hierarchy**
   - Top-level modules for major functionality
   - Sub-modules for related components
   - Clear public API via `lib.rs`

2. **Error Handling**
   - Use `Result<T>` for fallible operations
   - Define custom error types in `error.rs`
   - Provide context in error messages
   - Use `?` operator for error propagation

3. **Documentation**
   - Public APIs documented with `///` comments
   - Module-level documentation with `//!`
   - Examples in documentation
   - Link to relevant types and functions

4. **Testing**
   - Unit tests in same file as code (`#[cfg(test)]`)
   - Integration tests in `tests/` directory
   - Test edge cases and error conditions
   - Use descriptive test names

### Code Style

**Rust API Guidelines:**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Use `SCREAMING_SNAKE_CASE` for constants

**Error Messages:**
- Be specific and actionable
- Include relevant context (paths, values)
- Suggest solutions when possible

**Comments:**
- Explain "why", not "what"
- Use TODO/FIXME for known issues
- Keep comments up-to-date with code

### Security Guidelines

1. **Never Hardcode Secrets**
   - Use configuration files
   - Prompt for sensitive data
   - Use environment variables if needed

2. **Validate All Inputs**
   - Check file paths before use
   - Validate user input formats
   - Sanitize data for display

3. **Handle Errors Securely**
   - Don't leak sensitive information in errors
   - Use generic messages for security errors
   - Log details for debugging (not to user)

4. **Use Secure Defaults**
   - Strong key sizes (4096-bit RSA)
   - Short certificate validity (375 days)
   - Restrictive file permissions

### Performance Considerations

1. **Batch Processing**
   - Use rayon for CPU-bound parallelization
   - Reuse CA key across certificates
   - Minimize file I/O operations

2. **Memory Usage**
   - Stream large files when possible
   - Clean up resources promptly (RAII)
   - Avoid unnecessary allocations

3. **Cryptographic Operations**
   - Most time spent in OpenSSL
   - Key generation is slowest operation
   - Signing is relatively fast

## Instructions for Claude/AI Assistants

### When Working on This Project

1. **Understanding the Codebase**
   - Start with `src/lib.rs` for module overview
   - Read `src/error.rs` to understand error handling
   - Check `src/config.rs` for configuration structure
   - Review crypto modules for core functionality

2. **Making Code Changes**
   - Always run tests after changes: `cargo test`
   - Format code before committing: `cargo fmt`
   - Run clippy for linting: `cargo clippy`
   - Update documentation comments
   - Add tests for new functionality

3. **Adding New Features**
   - Define error types first (in `error.rs`)
   - Implement core logic with proper error handling
   - Add configuration options if needed
   - Write unit tests
   - Update CLI if user-facing
   - Document in README.md

4. **Common Tasks**

   **Add Support for ECDSA Keys:**
   1. Update `crypto::key` module for ECDSA generation
   2. Modify CSR creation to support ECDSA
   3. Update configuration for key type selection
   4. Add tests for ECDSA operations
   5. Update documentation

   **Implement Certificate Renewal:**
   1. Add expiration checking function (already exists)
   2. Create renewal command in CLI
   3. Load existing certificate and key
   4. Generate new CSR with same SANs
   5. Sign and replace certificate

   **Add Certificate Revocation Support:**
   1. Add CRL management functions
   2. Update CA module for revocation operations
   3. Add CLI command for revocation
   4. Update OpenSSL config handling
   5. Document CRL workflow

   **Enhance Batch Processing:**
   1. Add progress bar using indicatif
   2. Implement filtering options
   3. Add dry-run mode
   4. Export batch reports (JSON/CSV)
   5. Add resume capability

5. **Debugging Tips**
   - Use `RUST_LOG=debug` or `RUST_LOG=trace` for detailed logs
   - Check `tracing::debug!` and `tracing::info!` calls
   - Use `dbg!()` macro for quick debugging
   - Run with `--verbose` flag for more output
   - Check error source chains for root causes

6. **Security Checklist**
   - [ ] No hardcoded secrets or passwords
   - [ ] Sensitive data wrapped in `Secret<T>`
   - [ ] Memory zeroed after use (zeroize)
   - [ ] File permissions set correctly
   - [ ] Input validation performed
   - [ ] Error messages don't leak sensitive info
   - [ ] Temporary files cleaned up
   - [ ] Cryptographic randomness used

### Questions to Ask Users

When extending or modifying the project:

1. **Feature Requirements**
   - What is the use case?
   - Who are the users (human or automation)?
   - What are the security requirements?
   - Platform targets (Linux, macOS, Windows)?

2. **Configuration**
   - Should it be configurable?
   - Default values?
   - Required vs. optional settings?

3. **Error Handling**
   - How should errors be presented?
   - Retry behavior?
   - Fallback options?

4. **Compatibility**
   - Backward compatibility needed?
   - Breaking changes acceptable?
   - Migration path for existing users?

### Resources

- [OpenSSL Documentation](https://www.openssl.org/docs/)
- [X.509 Certificate Format (RFC 5280)](https://datatracker.ietf.org/doc/html/rfc5280)
- [Rust OpenSSL Crate Docs](https://docs.rs/openssl/latest/openssl/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [PKI Best Practices](https://www.ietf.org/rfc/rfc4210.txt)
- [clap Documentation](https://docs.rs/clap/latest/clap/)
- [secrecy Crate](https://docs.rs/secrecy/latest/secrecy/)

## Troubleshooting

### Common Development Issues

**Build Fails with OpenSSL Not Found**
```bash
# Install OpenSSL development libraries
sudo apt-get install libssl-dev pkg-config  # Ubuntu/Debian
sudo dnf install openssl-devel              # Fedora
brew install openssl@3 pkg-config           # macOS

# Set PKG_CONFIG_PATH on macOS
export PKG_CONFIG_PATH="/usr/local/opt/openssl@3/lib/pkgconfig"
```

**Tests Fail Due to Missing CA**
- Tests create mock CA certificates
- Check test helper functions in `cert.rs`
- Ensure tempfile cleanup is working

**Clippy Warnings**
```bash
# Fix automatically when possible
cargo clippy --fix

# Allow specific warnings
#[allow(clippy::module_name_repetitions)]

# Project-wide allows in src/lib.rs or Cargo.toml
```

**Format Issues**
```bash
# Format all code
cargo fmt

# Check without modifying
cargo fmt --check
```

## Web Service Architecture (v2.5+)

### Overview

Starting with v2.5, Flux SSL Manager includes a web-based interface for certificate management operations. This provides a user-friendly alternative to the CLI, making certificate operations accessible through a browser.

See [web-roadmap.md](web-roadmap.md) for comprehensive web service implementation details.

### Web Service Components

#### Project Structure

```
src/web/
├── mod.rs              # Web module exports
├── server.rs           # Axum server setup and configuration
├── routes/             # Route definitions
│   ├── mod.rs
│   ├── api.rs          # API routes (/api/*)
│   ├── csr.rs          # CSR upload endpoints
│   ├── cert.rs         # Certificate generation endpoints
│   └── info.rs         # Certificate info endpoints
├── handlers/           # Request handlers (business logic)
│   ├── mod.rs
│   ├── csr_handler.rs  # CSR upload and signing
│   ├── cert_handler.rs # Manual certificate generation
│   └── info_handler.rs # Certificate information display
├── models/             # Request/response models
│   ├── mod.rs
│   ├── requests.rs     # API request types
│   ├── responses.rs    # API response types
│   └── errors.rs       # Error response models
├── middleware/         # Custom middleware
│   ├── mod.rs
│   ├── auth.rs         # Authentication middleware
│   ├── logging.rs      # Request logging
│   └── cors.rs         # CORS handling
└── templates/          # HTML templates (Askama)
    ├── base.html       # Base template
    ├── index.html      # Landing page
    ├── csr_upload.html # CSR upload interface
    ├── cert_request.html # Manual cert request form
    └── cert_info.html  # Certificate viewer

static/                 # Static web assets
├── css/
│   └── styles.css     # Main stylesheet
├── js/
│   └── app.js         # Frontend JavaScript
└── images/            # Images and icons
```

#### Technology Stack

**Backend:**
- **axum** - Web framework (type-safe, composable, tokio-based)
- **tokio** - Async runtime
- **tower** - Middleware and service composition
- **tower-http** - HTTP-specific middleware (CORS, compression, etc.)
- **askama** - Type-safe templating engine
- **multer** - Multipart form data parsing
- **serde_json** - JSON serialization
- **validator** - Input validation

**Frontend:**
- HTML5/CSS3/JavaScript (initial implementation)
- Potential future migration to React/Vue/Svelte
- Responsive design (mobile-friendly)
- Progressive enhancement approach

**Security:**
- **argon2** - Password hashing
- **jsonwebtoken** - JWT authentication
- **tower-sessions** - Session management
- HTTPS/TLS support with rustls or openssl

### Core Features

#### 1. CSR Upload & Signing

**Endpoint**: `POST /api/csr/upload`

Accepts CSR file uploads and returns signed certificates.

**Request:**
```json
{
  "csr_file": "multipart/form-data",
  "sans": ["DNS:example.com", "IP:192.168.1.1"],
  "validity_days": 375
}
```

**Response:**
```json
{
  "success": true,
  "certificate": {
    "pem": "-----BEGIN CERTIFICATE-----...",
    "subject": "CN=example.com",
    "serial": "0A1B2C3D",
    "not_before": "2025-12-05T00:00:00Z",
    "not_after": "2026-12-05T23:59:59Z",
    "sans": ["DNS:example.com"]
  }
}
```

**Implementation:**
- Reuses existing `crypto::csr` module for CSR parsing
- Integrates with `ca::intermediate` for signing
- Validates CSR format and signature
- Supports additional SANs via form input

#### 2. Manual Certificate Request

**Endpoint**: `POST /api/cert/generate`

Interactive form for certificate generation without pre-existing CSR.

**Request:**
```json
{
  "common_name": "example.com",
  "sans": ["DNS:www.example.com", "IP:192.168.1.100"],
  "validity_days": 375,
  "key_size": 4096,
  "password_protect": false,
  "key_password": null
}
```

**Response:**
```json
{
  "success": true,
  "certificate": {
    "pem": "-----BEGIN CERTIFICATE-----...",
    "private_key": "-----BEGIN PRIVATE KEY-----...",
    "ca_chain": "-----BEGIN CERTIFICATE-----...",
    "download_url": "/api/cert/download/SESSION_ID"
  }
}
```

**Implementation:**
- Generates RSA/ECDSA key pairs using `crypto::key`
- Creates CSR with specified SANs using `crypto::csr`
- Signs certificate with CA using `crypto::cert`
- Creates downloadable ZIP bundle
- Optionally password-protects private key

#### 3. Certificate Information Display

**Endpoint**: `POST /api/cert/info`

Upload certificate files to view detailed information.

**Request:**
```json
{
  "cert_file": "multipart/form-data",
  "verify_chain": true
}
```

**Response:**
```json
{
  "success": true,
  "certificate": {
    "version": 3,
    "serial_number": "0A1B2C3D4E5F",
    "signature_algorithm": "sha256WithRSAEncryption",
    "issuer": { "CN": "Intermediate CA" },
    "validity": {
      "not_before": "2025-12-05T00:00:00Z",
      "not_after": "2026-12-05T23:59:59Z",
      "days_remaining": 365,
      "is_expired": false
    },
    "subject": { "CN": "example.com" },
    "subject_alternative_names": ["DNS:example.com"],
    "public_key": {
      "algorithm": "RSA",
      "size": 4096
    },
    "fingerprints": {
      "sha1": "A1:B2:C3:...",
      "sha256": "1A2B3C4D..."
    }
  }
}
```

**Implementation:**
- Parses certificates using OpenSSL
- Extracts and formats all certificate information
- Validates certificate chains
- Checks expiration status
- Calculates fingerprints

### Web Service Configuration

Add to `config.toml`:

```toml
[web]
enabled = false           # Enable web service
bind_address = "127.0.0.1"
port = 8443
tls_enabled = true
tls_cert_path = "/path/to/server.cert.pem"
tls_key_path = "/path/to/server.key.pem"
workers = 4
request_timeout = 30      # seconds
max_request_size = 10485760  # 10MB

[web.auth]
enabled = false           # Enable authentication
jwt_secret = "change-me"
session_timeout = 3600    # 1 hour
require_https = true

[web.limits]
rate_limit = 100          # requests per minute
max_file_size = 5242880   # 5MB
max_concurrent_uploads = 10

[web.cors]
enabled = true
allowed_origins = ["https://example.com"]
allowed_methods = ["GET", "POST"]

[web.logging]
access_log = "/var/log/flux-ssl-mgr/access.log"
audit_log = "/var/log/flux-ssl-mgr/audit.log"
log_level = "info"
```

### Security Considerations

#### Input Validation
- All user inputs validated before processing
- File uploads restricted by size and type
- Magic number verification for file types
- Path traversal prevention
- SAN format validation

#### Authentication & Authorization
- Optional authentication system (JWT-based)
- API key support for automation
- Role-based access control (RBAC)
- CSRF protection for web forms
- Secure session management

#### Network Security
- HTTPS/TLS enforcement (configurable)
- Secure headers (CSP, HSTS, X-Frame-Options)
- CORS configuration
- Rate limiting per IP
- Request size limits

#### Data Protection
- Temporary file cleanup (RAII)
- Secure memory handling for keys
- Audit logging for all operations
- No sensitive data in logs
- Certificate data encryption at rest (optional)

### API Error Handling

Consistent error response format following RFC 7807:

```json
{
  "success": false,
  "error": {
    "code": "INVALID_CSR",
    "message": "CSR validation failed",
    "details": "Invalid signature",
    "request_id": "req_123456"
  }
}
```

**Error Codes:**
- `INVALID_CSR` - CSR validation failed
- `INVALID_INPUT` - Input validation failed
- `FILE_TOO_LARGE` - File exceeds size limit
- `UNSUPPORTED_FORMAT` - Unsupported file format
- `CA_ERROR` - CA operation failed
- `SIGNING_FAILED` - Certificate signing failed
- `UNAUTHORIZED` - Authentication required
- `FORBIDDEN` - Insufficient permissions
- `INTERNAL_ERROR` - Internal server error

### Usage Examples

#### Start Web Service

```bash
# Start web service with default config
flux-ssl-mgr serve

# Start with custom config
flux-ssl-mgr serve --config /etc/flux-ssl-mgr/config.toml

# Start with custom port
flux-ssl-mgr serve --port 8080 --bind 0.0.0.0

# Start without TLS (development only)
flux-ssl-mgr serve --no-tls
```

#### API Usage with cURL

**Upload CSR:**
```bash
curl -X POST https://localhost:8443/api/csr/upload \
  -F "csr_file=@example.csr.pem" \
  -F "sans=DNS:www.example.com,IP:192.168.1.1" \
  -F "validity_days=375"
```

**Generate Certificate:**
```bash
curl -X POST https://localhost:8443/api/cert/generate \
  -H "Content-Type: application/json" \
  -d '{
    "common_name": "example.com",
    "sans": ["DNS:www.example.com"],
    "validity_days": 375,
    "key_size": 4096
  }'
```

**View Certificate Info:**
```bash
curl -X POST https://localhost:8443/api/cert/info \
  -F "cert_file=@example.cert.pem"
```

### Testing Web Service

```bash
# Run all web tests
cargo test --features web

# Run specific web module tests
cargo test --test web_integration

# Run with logging
RUST_LOG=debug cargo test --features web

# Test API endpoints
cargo test api::tests
```

### Deployment

#### Systemd Service

```bash
# Install and enable service
sudo cp flux-ssl-mgr.service /etc/systemd/system/
sudo systemctl enable flux-ssl-mgr
sudo systemctl start flux-ssl-mgr

# Check status
sudo systemctl status flux-ssl-mgr

# View logs
sudo journalctl -u flux-ssl-mgr -f
```

#### Docker

```bash
# Build image
docker build -t flux-ssl-mgr:latest .

# Run container
docker run -d \
  -p 8443:8443 \
  -v /path/to/ca:/root/ca:ro \
  -v /path/to/config.toml:/etc/flux-ssl-mgr/config.toml:ro \
  --name flux-ssl-mgr \
  flux-ssl-mgr:latest
```

#### Reverse Proxy (Nginx)

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
        proxy_read_timeout 60s;
    }
}
```

### Web Service Dependencies

Additional dependencies for web service:

```toml
[dependencies]
# Web framework
axum = { version = "0.7", features = ["multipart", "ws"] }
tokio = { version = "1.35", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors", "compression-gzip"] }

# Templates
askama = "0.12"
askama_axum = "0.4"

# Multipart forms
multer = "3.0"

# Authentication
jsonwebtoken = "9.2"
argon2 = "0.5"

# Sessions
tower-sessions = "0.9"

# Validation
validator = { version = "0.17", features = ["derive"] }

# API documentation
utoipa = { version = "4.1", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }
```

### Instructions for Claude/AI Assistants - Web Service

When working on the web service:

1. **Understanding Web Architecture**
   - Start with `src/web/mod.rs` for module overview
   - Review `src/web/server.rs` for server setup
   - Check route definitions in `src/web/routes/`
   - Understand handlers in `src/web/handlers/`

2. **Adding New Endpoints**
   - Define request/response models in `src/web/models/`
   - Implement handler logic in appropriate handler module
   - Add route in `src/web/routes/`
   - Update OpenAPI documentation
   - Write tests in `tests/web/`

3. **Security Checklist**
   - [ ] Input validation implemented
   - [ ] Authentication/authorization checked
   - [ ] CSRF protection (for forms)
   - [ ] Rate limiting applied
   - [ ] File upload restrictions
   - [ ] Error messages don't leak sensitive info
   - [ ] Audit logging enabled

4. **Testing Requirements**
   - Unit tests for handlers
   - Integration tests for endpoints
   - Security tests for auth/validation
   - Error case coverage
   - Load testing for performance

5. **Common Tasks**

   **Add New API Endpoint:**
   1. Define request/response models
   2. Implement handler function
   3. Add route to router
   4. Add middleware (auth, validation)
   5. Write unit tests
   6. Update OpenAPI spec
   7. Document in web-roadmap.md

   **Add Authentication:**
   1. Implement JWT token generation
   2. Create auth middleware
   3. Add login/logout endpoints
   4. Protect routes with middleware
   5. Add user session management
   6. Test auth flows

   **Improve UI:**
   1. Update templates in `templates/`
   2. Enhance CSS in `static/css/`
   3. Add JavaScript in `static/js/`
   4. Test responsiveness
   5. Check accessibility

## Future Enhancements

See [ROADMAP.md](ROADMAP.md) for comprehensive future plans.
See [web-roadmap.md](web-roadmap.md) for web service implementation details.

### Short-term (v2.1)
- Complete test coverage (unit + integration)
- Add file ownership management (users + nix crates)
- Certificate renewal tracking
- Expiration notifications
- ECDSA key support
- Enhanced logging and auditing

### Medium-term (v2.2-2.5)
- Certificate revocation support
- Multiple CA support
- Database backend for tracking
- Certificate search and filtering
- Automated backup/restore
- REST API for automation
- **Web service implementation (v2.5)** - See web-roadmap.md

### Long-term (v3.0+)
- ACME protocol support (Let's Encrypt)
- Enhanced web UI with modern framework (React/Vue)
- Hardware Security Module (HSM) support
- Certificate monitoring and alerting
- Integration with service discovery (Consul, etcd)
- Plugin system for extensions
- Mobile app (PWA)

## Contributing

When contributing to this project:

1. **Code Quality**
   - Write idiomatic Rust code
   - Follow project conventions
   - Add comprehensive tests
   - Document public APIs

2. **Security**
   - Follow security best practices
   - No unsafe code without justification
   - Validate all inputs
   - Handle secrets properly

3. **Testing**
   - Test with non-production CA if possible
   - Include unit tests for new functions
   - Add integration tests for workflows
   - Test error paths

4. **Documentation**
   - Update README.md for user-facing changes
   - Update this file for architecture changes
   - Add inline documentation
   - Include examples

5. **Pull Requests**
   - Clear description of changes
   - Link to related issues
   - Include test results
   - Note breaking changes

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Changelog

### Version 2.0.0 (Current)
- Complete Rust rewrite from bash
- Interactive and CLI modes
- Batch processing with parallelization
- Configuration file support (TOML)
- Secure password handling
- Comprehensive error types
- Modern CLI with clap
- Colored output
- Unit tests for crypto operations

### Version 1.0.0 (Bash - Archived)
- Original bash implementation
- Single and batch modes
- Basic error handling
- Hardcoded paths
- See `old/flux-ssl-mgr.sh`

---

**Last Updated**: 2025-01-21
**Current Version**: 2.0.0 (Rust)
**Target Version**: 2.1.0 (Certificate lifecycle management)

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

## Future Enhancements

See [ROADMAP.md](ROADMAP.md) for comprehensive future plans.

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

### Long-term (v3.0+)
- ACME protocol support (Let's Encrypt)
- Web UI for certificate management
- Hardware Security Module (HSM) support
- Certificate monitoring and alerting
- Integration with service discovery (Consul, etcd)
- Plugin system for extensions

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

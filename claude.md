# Flux SSL Manager - Claude.md

## Project Overview

**flux-ssl-mgr** is a certificate management tool designed for homestead/homelab internal PKI environments. It automates the generation, signing, and management of SSL/TLS certificates using an intermediate Certificate Authority (CA).

### Current State
- **Language**: Bash shell script
- **Primary Script**: `flux-certs.sh`
- **Status**: Production-ready bash implementation
- **Future Goal**: Convert to Rust for improved safety, performance, and maintainability

### Purpose
This tool simplifies certificate lifecycle management in a homelab PKI setup by:
- Generating RSA private keys (4096-bit)
- Creating Certificate Signing Requests (CSRs) with Subject Alternative Names (SANs)
- Signing certificates using an intermediate CA
- Managing file permissions and ownership
- Supporting both single and batch certificate processing

## Architecture & Design

### Current Bash Implementation

#### Directory Structure
```
/root/ca/                           # PKI working directory
├── intermediate/
│   ├── private/                    # Private keys (CA and generated)
│   │   └── intermediate.key.pem    # Intermediate CA key
│   ├── certs/                      # Signed certificates
│   ├── csr/                        # Certificate signing requests
│   └── openssl.cnf                 # OpenSSL configuration
/home/fluxadmin/ssl/                # Input directory for CSR files
└── pem-out/                        # Output directory for generated certs
```

#### Key Features
1. **Password-Protected CA Key Handling**
   - Detects if intermediate CA key is password-protected
   - Creates temporary unlocked key for session
   - Automatic cleanup on exit via trap mechanism

2. **Dual Processing Modes**
   - **Single Mode**: Interactive certificate generation
   - **Batch Mode**: Process multiple CSR files from directory

3. **Certificate Generation Workflow**
   - Generate private key (optionally password-protected)
   - Create CSR with SANs
   - Sign certificate with intermediate CA (375-day validity)
   - Convert to both PEM and CRT formats
   - Copy to output directory with proper permissions

4. **File Permissions**
   - Private keys: 400 (read-only for owner)
   - Certificates: 755 (readable by all)
   - Ownership: fluxadmin:root

5. **User Experience**
   - Colored output (green/yellow/red)
   - Progress indicators
   - Error handling with `set -e`
   - Cleanup on exit

### Technical Components

#### OpenSSL Operations
- **Key Generation**: RSA 4096-bit with optional AES-256 encryption
- **CSR Creation**: SHA-256 with SAN extensions
- **Certificate Signing**: SHA-256, 375-day validity, server_cert extensions

#### Security Considerations
- Temporary files created with PID suffix (`$$`)
- Restrictive permissions (600 on temp CA key, 400 on private keys)
- Automatic cleanup via trap mechanism
- Password-protected CA key support

## Usage Instructions

### Prerequisites
- Root access or sudo privileges
- OpenSSL installed
- Intermediate CA already set up at `/root/ca/intermediate/`
- Proper directory permissions

### Running the Script

#### Single Certificate Mode
```bash
sudo ./flux-certs.sh
# Select option 1
# Provide:
#   - Certificate name (e.g., "myservice")
#   - SANs (e.g., "DNS:myservice.fluxlab.systems,IP:10.0.2.100")
#   - Password protection preference
```

#### Batch Processing Mode
```bash
sudo ./flux-certs.sh
# Select option 2
# Provide:
#   - CSR directory path
#   - Select CSRs to process (all, range, or specific)
#   - Common SANs or individual configuration
#   - Password protection preference
```

### Output Files
For each certificate named `example`:
- `example.cert.pem` - Certificate in PEM format
- `example.crt` - Certificate in CRT format
- `example.key.pem` - Private key

Location: `/home/fluxadmin/ssl/pem-out/`

## Development Guidelines

### Code Organization
- Single-responsibility functions
- Clear variable naming conventions
- Error handling at each step
- User-friendly output messages

### Testing Checklist
- [ ] Password-protected CA key handling
- [ ] Unprotected CA key handling
- [ ] Single certificate generation
- [ ] Batch processing with common SANs
- [ ] Batch processing with individual SANs
- [ ] Password-protected private keys
- [ ] Unprotected private keys
- [ ] File permission verification
- [ ] Cleanup mechanism (temp files removed)
- [ ] Error handling (missing directories, invalid inputs)

### Modifying the Script
1. **Adding new certificate extensions**: Modify the OpenSSL config or `-extensions` parameter
2. **Changing key sizes**: Update the `openssl genrsa` command parameter
3. **Adjusting certificate validity**: Modify the `-days` parameter in signing command
4. **Customizing output locations**: Update `OUTPUT_DIR` and related paths

### Known Limitations
- Requires root privileges
- Hardcoded paths (need manual adjustment for different environments)
- Limited to RSA keys (no ECDSA support)
- No certificate renewal tracking
- No integration with ACME/Let's Encrypt

## Future Rust Conversion

### Conversion Goals
1. **Type Safety**: Eliminate runtime errors through Rust's type system
2. **Memory Safety**: No buffer overflows or memory leaks
3. **Concurrency**: Safe concurrent batch processing
4. **Error Handling**: Rich error types and proper error propagation
5. **Cross-Platform**: Support Linux, macOS, and potentially Windows
6. **Configuration**: TOML/YAML config files instead of hardcoded paths
7. **Testing**: Comprehensive unit and integration tests

### Recommended Rust Architecture

#### Project Structure
```
flux-ssl-mgr/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration management
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── key.rs           # Key generation and management
│   │   ├── csr.rs           # CSR creation
│   │   └── cert.rs          # Certificate signing and management
│   ├── ca/
│   │   ├── mod.rs
│   │   └── intermediate.rs  # Intermediate CA operations
│   ├── batch.rs             # Batch processing logic
│   ├── interactive.rs       # Interactive mode
│   ├── output.rs            # Output formatting and colors
│   └── error.rs             # Error types
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/            # Test certificates and configs
└── benches/                 # Performance benchmarks
```

#### Key Rust Crates to Use

**Cryptography**
- `openssl` or `rust-openssl` - OpenSSL bindings
- `rustls` - (Optional) Pure Rust TLS implementation
- `x509-parser` - X.509 certificate parsing
- `rcgen` - (Alternative) Pure Rust X.509 certificate generation

**CLI & User Interface**
- `clap` v4 - Command-line argument parsing (derive API)
- `dialoguer` - Interactive prompts
- `indicatif` - Progress bars and spinners
- `console` - Terminal colors and formatting
- `crossterm` - Cross-platform terminal manipulation

**Configuration**
- `serde` - Serialization/deserialization
- `toml` - TOML configuration files
- `config` - Layered configuration management

**Error Handling**
- `thiserror` - Error derive macros
- `anyhow` - Flexible error handling for applications

**Async/Concurrency** (for batch processing)
- `tokio` - Async runtime
- `rayon` - Data parallelism

**Security**
- `secrecy` - Wrapper types for secret data
- `zeroize` - Securely zero memory

**File Operations**
- `walkdir` - Directory traversal
- `tempfile` - Temporary file management

**Utilities**
- `chrono` - Date/time handling
- `tracing` - Structured logging

#### Core Features to Implement

1. **Configuration Module** (`config.rs`)
   ```rust
   pub struct Config {
       pub working_dir: PathBuf,
       pub output_dir: PathBuf,
       pub ca_key_path: PathBuf,
       pub openssl_config: PathBuf,
       pub default_key_size: u32,
       pub default_cert_days: u32,
       pub default_owner: String,
       pub default_group: String,
   }
   ```

2. **Error Handling** (`error.rs`)
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum FluxError {
       #[error("CA key not found: {0}")]
       CaKeyNotFound(PathBuf),
       #[error("Invalid certificate name: {0}")]
       InvalidCertName(String),
       #[error("OpenSSL error: {0}")]
       OpenSslError(#[from] openssl::error::ErrorStack),
       // ... more error variants
   }
   ```

3. **Key Generation Module** (`crypto/key.rs`)
   - Generate RSA/ECDSA keys
   - Password protection using secrecy crate
   - Key format conversion

4. **CSR Module** (`crypto/csr.rs`)
   - Create CSR with SANs
   - Parse existing CSR files
   - Validate CSR contents

5. **Certificate Module** (`crypto/cert.rs`)
   - Sign certificates
   - Convert between formats (PEM/CRT/DER)
   - Extract certificate information
   - Validate certificates

6. **CA Module** (`ca/intermediate.rs`)
   - Load intermediate CA
   - Unlock password-protected keys
   - Manage CA certificate chain

7. **Batch Processing** (`batch.rs`)
   - Discover CSR files
   - Parallel processing with Rayon or Tokio
   - Progress tracking
   - Error aggregation

8. **Interactive Mode** (`interactive.rs`)
   - Use dialoguer for prompts
   - Input validation
   - Confirmation prompts

9. **CLI Interface** (`main.rs`)
   ```rust
   #[derive(Parser)]
   #[command(name = "flux-ssl-mgr")]
   #[command(about = "Certificate management for homelab PKI")]
   struct Cli {
       #[command(subcommand)]
       command: Commands,
   }

   #[derive(Subcommand)]
   enum Commands {
       /// Generate a single certificate
       Single {
           /// Certificate name
           #[arg(short, long)]
           name: String,
           /// Subject Alternative Names
           #[arg(short, long, value_delimiter = ',')]
           sans: Vec<String>,
           /// Password-protect the private key
           #[arg(short, long)]
           password: bool,
       },
       /// Batch process CSR files
       Batch {
           /// Directory containing CSR files
           #[arg(short, long)]
           dir: PathBuf,
           /// Process all CSRs without prompting
           #[arg(short, long)]
           all: bool,
       },
       /// Show certificate information
       Info {
           /// Certificate file path
           cert: PathBuf,
       },
   }
   ```

### Conversion Strategy

#### Phase 1: Project Setup
1. Initialize Cargo project
2. Set up directory structure
3. Add dependencies to Cargo.toml
4. Create basic CLI with clap
5. Implement configuration loading

#### Phase 2: Core Functionality
1. Implement error types
2. Create crypto modules (key, csr, cert)
3. Implement CA module
4. Add file operations and permissions

#### Phase 3: User Interface
1. Implement single certificate mode
2. Implement batch processing mode
3. Add interactive prompts
4. Implement colored output and progress indicators

#### Phase 4: Testing & Documentation
1. Unit tests for all modules
2. Integration tests with test fixtures
3. Documentation comments (rustdoc)
4. User guide and examples

#### Phase 5: Polish & Distribution
1. Error message refinement
2. Performance optimization
3. Cross-platform testing
4. Create installation instructions
5. Package as binary (cargo install, releases)

### Migration Considerations

#### Compatibility
- Maintain same output directory structure
- Preserve file permission model
- Support existing OpenSSL config files
- Backward compatibility with existing CSR files

#### Improvements Over Bash Version
- **Configuration files**: No hardcoded paths
- **Better error messages**: Rich error types with context
- **Validation**: Input validation before processing
- **Dry-run mode**: Preview operations without execution
- **Certificate tracking**: Optional database or JSON tracking
- **Renewal reminders**: Track expiration dates
- **ACME support**: (Future) Let's Encrypt integration
- **Multiple CA support**: Work with multiple intermediate CAs
- **Key algorithm flexibility**: RSA + ECDSA support
- **Logging**: Structured logging with different levels
- **Non-interactive mode**: Full CLI args for automation

### Testing Strategy for Rust Version

1. **Unit Tests**
   - Each function in crypto modules
   - Configuration parsing
   - Error handling paths

2. **Integration Tests**
   - Full certificate generation workflow
   - Batch processing
   - File permission verification
   - Cleanup verification

3. **Test Fixtures**
   - Sample CA certificates
   - Test OpenSSL configurations
   - Various CSR files

4. **Security Tests**
   - Memory zeroization for sensitive data
   - Proper file permissions
   - No temporary file leaks

## Instructions for Claude/AI Assistants

### When Working on This Project

1. **Current Implementation**
   - The `old/` directory contains deprecated code and can be ignored
   - Focus on `flux-certs.sh` for understanding current functionality
   - All paths are currently hardcoded for a specific homelab environment

2. **Making Changes to Bash Script**
   - Always test error handling paths
   - Maintain the cleanup trap mechanism
   - Preserve colored output for user feedback
   - Keep backward compatibility with existing file structures
   - Update usage prompts if adding new features

3. **Starting Rust Conversion**
   - Begin with Phase 1 (Project Setup) from the conversion strategy
   - Prioritize type safety and error handling
   - Use modern Rust idioms (edition 2021+)
   - Follow Rust API guidelines
   - Write tests alongside implementation

4. **Key Design Principles**
   - Security first: Handle secrets properly, validate inputs
   - User experience: Clear messages, progress indicators
   - Reliability: Comprehensive error handling, cleanup guarantees
   - Maintainability: Well-documented, modular code
   - Performance: Efficient batch processing for multiple certificates

5. **Questions to Ask User**
   - Configuration preferences (TOML vs YAML)
   - Desired CLI style (interactive vs pure CLI args)
   - Platform targets (Linux-only or cross-platform)
   - Additional features to prioritize
   - Existing PKI infrastructure details

6. **Before Making Major Changes**
   - Understand the full certificate workflow
   - Review OpenSSL command parameters
   - Test with a non-production CA if possible
   - Consider impact on existing certificate files

7. **Documentation Updates**
   - Keep this claude.md file updated
   - Document any breaking changes
   - Update usage examples
   - Note security considerations

### Common Tasks

**Add Support for ECDSA Keys**
1. Add new key type parameter
2. Update key generation function
3. Modify CSR creation for ECDSA
4. Test with intermediate CA

**Implement Certificate Renewal**
1. Add expiration checking
2. Create renewal command/mode
3. Preserve certificate metadata
4. Optional notification system

**Add Configuration File Support**
1. Define config schema
2. Implement config parser
3. Support config file path via CLI
4. Merge CLI args with config file

**Enhance Batch Processing**
1. Add parallel processing
2. Implement better progress tracking
3. Add filtering options (by date, name pattern)
4. Export batch reports

### Resources

- [OpenSSL Documentation](https://www.openssl.org/docs/)
- [X.509 Certificate Format](https://datatracker.ietf.org/doc/html/rfc5280)
- [Rust OpenSSL Crate](https://docs.rs/openssl/latest/openssl/)
- [rcgen - Pure Rust X.509](https://docs.rs/rcgen/latest/rcgen/)
- [PKI Best Practices](https://www.ietf.org/rfc/rfc4210.txt)

### Environment Notes

- **Target Environment**: Linux homelab/homestead
- **User Context**: Root or fluxadmin user
- **CA Type**: Two-tier PKI (root + intermediate)
- **Certificate Use Cases**: Internal services (HTTPS, TLS)
- **Integration Points**: Services consuming certificates from output directory

## Troubleshooting

### Common Issues

**CA Key Not Found**
- Verify `/root/ca/intermediate/private/intermediate.key.pem` exists
- Check file permissions (should be readable by root)

**Permission Denied**
- Script requires root privileges
- Use `sudo ./flux-certs.sh`

**Certificate Signing Fails**
- Verify intermediate CA certificate is valid
- Check OpenSSL configuration file
- Ensure CA database files exist (index.txt, serial)

**CSR Not Found**
- Check CSR file location
- Verify file extension is `.csr`
- Ensure file path is correct

## Future Enhancements

### Short-term (Bash)
- [ ] Add certificate revocation support
- [ ] Implement certificate renewal checking
- [ ] Add JSON output mode for automation
- [ ] Support custom certificate validity periods
- [ ] Add certificate verification command

### Long-term (Rust)
- [ ] Complete Rust conversion (see conversion strategy)
- [ ] Add ACME protocol support
- [ ] Implement certificate monitoring/alerting
- [ ] Create web UI for certificate management
- [ ] Add database backend for certificate tracking
- [ ] Support multiple intermediate CAs
- [ ] Implement automated renewal workflows
- [ ] Add metrics and reporting
- [ ] Support hardware security modules (HSM)

## Contributing

When contributing to this project:
1. Understand the PKI workflow
2. Test thoroughly with non-production CA
3. Maintain backward compatibility
4. Update documentation
5. Follow security best practices
6. Add tests for new features

## License

(To be determined by repository owner)

---

**Last Updated**: 2025-11-15
**Current Version**: 1.0 (Bash)
**Target Version**: 2.0 (Rust - planned)

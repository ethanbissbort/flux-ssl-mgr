# Flux SSL Manager

A powerful, secure certificate management tool for homestead/homelab internal PKI environments.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Flux SSL Manager automates the generation, signing, and management of SSL/TLS certificates using an intermediate Certificate Authority (CA). Built with security and ease-of-use in mind, it streamlines certificate lifecycle management for homelab infrastructure.

### Features

- **Automated Certificate Generation**: Create RSA 4096-bit private keys and certificates
- **Batch Processing**: Process multiple certificates efficiently
- **Interactive & CLI Modes**: Choose between guided prompts or command-line arguments
- **Subject Alternative Names (SANs)**: Full support for DNS and IP SANs
- **Password Protection**: Optional password encryption for private keys
- **Secure CA Key Handling**: Automatic unlocking and cleanup of password-protected CA keys
- **Multiple Output Formats**: PEM and CRT certificate formats
- **Permission Management**: Automatic file ownership and permission configuration
- **Progress Tracking**: Clear visual feedback with colored output

## Installation

### From Binary (Recommended)

```bash
# Download latest release
wget https://github.com/ethanbissbort/flux-ssl-mgr/releases/latest/download/flux-ssl-mgr

# Make executable
chmod +x flux-ssl-mgr

# Move to PATH
sudo mv flux-ssl-mgr /usr/local/bin/
```

### From Source

```bash
# Clone repository
git clone https://github.com/ethanbissbort/flux-ssl-mgr.git
cd flux-ssl-mgr

# Build with Cargo
cargo build --release

# Install binary
cargo install --path .
```

### Prerequisites

- Rust 1.70+ (for building from source)
- OpenSSL development libraries
- An existing intermediate Certificate Authority setup

**Ubuntu/Debian:**
```bash
sudo apt-get install libssl-dev pkg-config
```

**RHEL/Fedora:**
```bash
sudo dnf install openssl-devel
```

**macOS:**
```bash
brew install openssl@3
```

## Quick Start

### Initial Setup

1. **Create Configuration File**

```bash
# Generate default config
flux-ssl-mgr config --init

# Edit configuration
vim ~/.config/flux-ssl-mgr/config.toml
```

2. **Configure Your PKI Paths**

Edit `config.toml`:
```toml
working_dir = "/root/ca"
output_dir = "/home/fluxadmin/ssl/pem-out"
ca_key_path = "/root/ca/intermediate/private/intermediate.key.pem"
openssl_config = "/root/ca/intermediate/openssl.cnf"
```

### Generate a Single Certificate

**Interactive Mode:**
```bash
flux-ssl-mgr single
```

**CLI Mode:**
```bash
flux-ssl-mgr single \
  --name myservice \
  --sans DNS:myservice.fluxlab.systems,DNS:myservice.local,IP:10.0.2.100 \
  --password
```

### Batch Process Certificates

**Interactive Mode:**
```bash
flux-ssl-mgr batch
```

**Process All CSRs:**
```bash
flux-ssl-mgr batch --dir /home/fluxadmin/ssl --all
```

### View Certificate Information

```bash
flux-ssl-mgr info /path/to/certificate.pem
```

## Usage

### Single Certificate Mode

Generate one certificate at a time with full control over configuration.

```bash
flux-ssl-mgr single [OPTIONS]

Options:
  -n, --name <NAME>           Certificate name (e.g., myservice)
  -s, --sans <SANS>...        Subject Alternative Names (comma-separated)
                              Example: DNS:*.example.com,IP:192.168.1.100
  -p, --password              Password-protect the private key
  -d, --days <DAYS>           Certificate validity in days [default: 375]
  -k, --key-size <SIZE>       RSA key size in bits [default: 4096]
  -h, --help                  Print help information
```

**Examples:**

```bash
# Basic certificate
flux-ssl-mgr single --name webserver --sans DNS:web.local

# Certificate with multiple SANs
flux-ssl-mgr single \
  --name apiserver \
  --sans DNS:api.fluxlab.systems,DNS:api.local,IP:10.0.2.50

# Password-protected certificate with custom validity
flux-ssl-mgr single \
  --name database \
  --sans DNS:db.fluxlab.systems \
  --password \
  --days 730
```

### Batch Processing Mode

Process multiple CSR files efficiently.

```bash
flux-ssl-mgr batch [OPTIONS]

Options:
  -d, --dir <DIR>             Directory containing CSR files
  -a, --all                   Process all CSRs without prompting
  -f, --filter <PATTERN>      Filter CSRs by name pattern
  -s, --sans <SANS>...        Common SANs for all certificates
  -p, --password              Password-protect all private keys
  -h, --help                  Print help information
```

**Examples:**

```bash
# Interactive batch processing
flux-ssl-mgr batch --dir /home/fluxadmin/ssl

# Process all with common SANs
flux-ssl-mgr batch \
  --dir /home/fluxadmin/ssl \
  --all \
  --sans DNS:*.fluxlab.systems

# Process filtered certificates
flux-ssl-mgr batch \
  --dir /home/fluxadmin/ssl \
  --filter "web*" \
  --all
```

### Certificate Information

View detailed certificate information.

```bash
flux-ssl-mgr info <CERTIFICATE>

Options:
  -v, --verbose               Show full certificate details
  -h, --help                  Print help information
```

**Example:**

```bash
flux-ssl-mgr info /home/fluxadmin/ssl/pem-out/myservice.cert.pem
```

## Configuration

Flux SSL Manager uses TOML configuration files for flexible setup.

### Configuration File Locations

The tool searches for configuration in the following order:

1. `./flux-ssl-mgr.toml` (current directory)
2. `~/.config/flux-ssl-mgr/config.toml` (user config)
3. `/etc/flux-ssl-mgr/config.toml` (system config)

### Configuration Options

```toml
# PKI Directory Configuration
working_dir = "/root/ca"
output_dir = "/home/fluxadmin/ssl/pem-out"
csr_input_dir = "/home/fluxadmin/ssl"

# CA Configuration
ca_key_path = "/root/ca/intermediate/private/intermediate.key.pem"
ca_cert_path = "/root/ca/intermediate/certs/intermediate.cert.pem"
openssl_config = "/root/ca/intermediate/openssl.cnf"

# Default Certificate Settings
[defaults]
key_size = 4096              # RSA key size in bits
cert_days = 375              # Certificate validity period
hash_algorithm = "sha256"    # Signature hash algorithm
owner = "fluxadmin"          # File owner
group = "root"               # File group

# File Permissions (octal)
[permissions]
private_key = 0o400          # Private key permissions
certificate = 0o755          # Certificate permissions
output_dir = 0o755           # Output directory permissions

# Batch Processing
[batch]
parallel = true              # Enable parallel processing
max_workers = 4              # Maximum concurrent operations
progress_bar = true          # Show progress bar

# Output Formatting
[output]
colored = true               # Enable colored output
verbose = false              # Verbose logging
quiet = false                # Suppress non-error output
```

## Directory Structure

### PKI Directory Layout

```
/root/ca/                                    # Working directory
├── intermediate/
│   ├── private/
│   │   ├── intermediate.key.pem             # Intermediate CA private key
│   │   └── *.key.pem                        # Generated private keys
│   ├── certs/
│   │   ├── intermediate.cert.pem            # Intermediate CA certificate
│   │   ├── *.cert.pem                       # Signed certificates (PEM)
│   │   └── *.crt                            # Signed certificates (CRT)
│   ├── csr/
│   │   └── *.csr.pem                        # Certificate signing requests
│   └── openssl.cnf                          # OpenSSL configuration
└── certs/
    └── ca.cert.pem                          # Root CA certificate
```

### Output Directory

```
/home/fluxadmin/ssl/pem-out/                 # Output directory
├── myservice.cert.pem                       # Certificate (PEM format)
├── myservice.crt                            # Certificate (CRT format)
└── myservice.key.pem                        # Private key
```

## Security Considerations

### Best Practices

1. **Protect CA Keys**: Keep intermediate CA keys secure and password-protected
2. **Limit Access**: Restrict access to output directories (certificate files)
3. **Use Strong Passwords**: When password-protecting private keys
4. **Regular Rotation**: Rotate certificates before expiration
5. **Audit Logging**: Monitor certificate generation activities
6. **Secure Storage**: Store certificates in appropriate locations with proper permissions

### File Permissions

Default permissions set by Flux SSL Manager:

- **Private Keys**: `400` (read-only for owner)
- **Certificates**: `755` (readable by all, writable by owner)
- **Temporary Files**: `600` (read/write for owner only)

### Temporary File Handling

- Temporary CA keys are created only when needed
- Automatic cleanup on exit (success or failure)
- Memory is zeroed before cleanup for sensitive data

## Troubleshooting

### Common Issues

#### CA Key Not Found

**Error:** `CA key not found at /path/to/key`

**Solution:**
```bash
# Verify CA key exists
ls -l /root/ca/intermediate/private/intermediate.key.pem

# Check permissions
sudo chmod 400 /root/ca/intermediate/private/intermediate.key.pem
```

#### Permission Denied

**Error:** `Permission denied when accessing CA directory`

**Solution:**
```bash
# Run with appropriate privileges
sudo flux-ssl-mgr single --name myservice --sans DNS:myservice.local
```

#### OpenSSL Configuration Error

**Error:** `OpenSSL configuration file not found`

**Solution:**
```bash
# Verify OpenSSL config path in configuration
flux-ssl-mgr config --show

# Update config if needed
vim ~/.config/flux-ssl-mgr/config.toml
```

#### Certificate Signing Failed

**Error:** `Failed to sign certificate`

**Solutions:**
1. Verify intermediate CA certificate is valid
2. Check OpenSSL configuration file syntax
3. Ensure CA database files exist (index.txt, serial)
4. Verify CA key password is correct

```bash
# Verify CA certificate
openssl x509 -in /root/ca/intermediate/certs/intermediate.cert.pem -noout -text

# Check CA database
ls -l /root/ca/intermediate/index.txt
ls -l /root/ca/intermediate/serial
```

### Enable Debug Logging

```bash
# Set log level to debug
RUST_LOG=debug flux-ssl-mgr single --name test --sans DNS:test.local

# Full trace logging
RUST_LOG=trace flux-ssl-mgr batch --dir /path/to/csrs
```

### Getting Help

```bash
# General help
flux-ssl-mgr --help

# Command-specific help
flux-ssl-mgr single --help
flux-ssl-mgr batch --help

# Show current configuration
flux-ssl-mgr config --show
```

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/ethanbissbort/flux-ssl-mgr.git
cd flux-ssl-mgr

# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- single --name test --sans DNS:test.local
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_key_generation

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Code Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root
├── config.rs            # Configuration management
├── error.rs             # Error types
├── crypto/
│   ├── mod.rs
│   ├── key.rs           # Key generation
│   ├── csr.rs           # CSR creation
│   └── cert.rs          # Certificate signing
├── ca/
│   ├── mod.rs
│   └── intermediate.rs  # CA operations
├── batch.rs             # Batch processing
├── interactive.rs       # Interactive mode
└── output.rs            # Output formatting
```

### Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for new functionality
4. Ensure all tests pass (`cargo test`)
5. Format code (`cargo fmt`)
6. Run linter (`cargo clippy`)
7. Commit changes (`git commit -m 'Add amazing feature'`)
8. Push to branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Roadmap

### Version 2.0 (Current - Rust)
- [x] Core certificate generation functionality
- [x] Interactive and CLI modes
- [x] Batch processing
- [x] Configuration file support
- [ ] Comprehensive test coverage
- [ ] Documentation and examples

### Version 2.1 (Planned)
- [ ] Certificate renewal tracking
- [ ] Expiration notifications
- [ ] Certificate revocation support
- [ ] Multiple CA support
- [ ] ECDSA key support

### Version 3.0 (Future)
- [ ] ACME protocol support (Let's Encrypt)
- [ ] Web UI for certificate management
- [ ] REST API
- [ ] Database backend for tracking
- [ ] Hardware Security Module (HSM) support
- [ ] Automated renewal workflows

## Migration from Bash Version

If you're upgrading from the bash version (`flux-certs.sh`):

1. **Configuration**: Create a config file instead of editing hardcoded paths
2. **CLI Arguments**: Use `--name` and `--sans` flags instead of interactive prompts
3. **Output Location**: Same default location, configurable via config file
4. **Batch Processing**: Enhanced with filtering and parallel processing
5. **Backward Compatible**: Same output directory structure and file formats

**Quick Migration:**

```bash
# Old way (bash)
sudo ./flux-certs.sh

# New way (Rust)
flux-ssl-mgr single  # Interactive mode
# or
flux-ssl-mgr single --name myservice --sans DNS:myservice.local  # CLI mode
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built for homelab enthusiasts managing internal PKI
- Uses OpenSSL for cryptographic operations
- Inspired by the need for simple, secure certificate management

## Support

- **Documentation**: See [claude.md](claude.md) for detailed technical documentation
- **Issues**: [GitHub Issues](https://github.com/ethanbissbort/flux-ssl-mgr/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ethanbissbort/flux-ssl-mgr/discussions)

## Authors

- **Ethan Bissbort** - Initial work and maintenance

---

**Note**: This tool is designed for internal/homelab PKI environments. For production use, consider additional security hardening and compliance requirements.

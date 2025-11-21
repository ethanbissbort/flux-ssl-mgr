# Flux SSL Manager - Project Roadmap

This document outlines the planned features, improvements, and long-term vision for Flux SSL Manager.

## Current Status (v2.0.0)

**Released**: January 2025

### Completed Features

- ✅ Complete Rust rewrite from bash implementation
- ✅ Interactive and CLI modes
- ✅ Batch processing with parallelization (rayon)
- ✅ Configuration file support (TOML)
- ✅ Secure password handling (secrecy + zeroize)
- ✅ Comprehensive error types (thiserror)
- ✅ Modern CLI with clap v4
- ✅ Colored terminal output
- ✅ Unit tests for crypto operations
- ✅ RSA 4096-bit key generation
- ✅ Subject Alternative Names (DNS, IP, Email)
- ✅ Certificate signing with intermediate CA
- ✅ Multiple output formats (PEM, CRT)
- ✅ File permission management (Unix)
- ✅ Certificate information display
- ✅ Expiration checking

### Known Limitations

- 🚧 File ownership changes (commented out, requires users + nix crates)
- 🚧 Limited test coverage (crypto modules only)
- 🚧 No certificate renewal automation
- 🚧 RSA keys only (no ECDSA)
- 🚧 No certificate revocation support
- 🚧 No certificate tracking/database
- 🚧 Basic GitHub Actions workflow

## Version 2.1 - Certificate Lifecycle Management

**Target Release**: Q2 2025

### Goals

Enhance certificate management with lifecycle tracking, renewal automation, and improved operational features.

### Features

#### Certificate Tracking & Management
- [ ] Certificate database (SQLite backend)
  - Track all generated certificates
  - Store metadata (issue date, expiration, SANs, etc.)
  - Link certificates to services/applications
  - Search and filter capabilities
- [ ] Certificate renewal command
  - `flux-ssl-mgr renew <name>` to renew specific certificate
  - Automatic detection of expiring certificates
  - Preserve existing SANs and settings
  - Generate new key or reuse existing
- [ ] Expiration monitoring
  - `flux-ssl-mgr list --expiring` to show certificates expiring soon
  - Configurable warning thresholds (30, 60, 90 days)
  - JSON/CSV export for monitoring systems
- [ ] Certificate history
  - Track renewal history
  - Store previous certificates
  - Audit trail of certificate operations

#### ECDSA Key Support
- [ ] ECDSA key generation
  - P-256, P-384, P-521 curves
  - Configuration option for key algorithm
  - Auto-detection of existing key type
- [ ] Update CSR creation for ECDSA
- [ ] Update certificate signing for ECDSA
- [ ] Tests for ECDSA operations

#### File Ownership Management
- [ ] Add `users` and `nix` crates
- [ ] Implement ownership changes (chown)
- [ ] Configurable owner/group per certificate
- [ ] Proper error handling for permission issues
- [ ] Root privilege checking

#### Testing & Quality
- [ ] Integration tests (end-to-end workflows)
- [ ] Configuration loading tests
- [ ] Batch processing tests
- [ ] Error path testing
- [ ] Coverage target: 80%+
- [ ] Add to CI/CD pipeline

#### Enhanced Logging & Auditing
- [ ] Structured logging with context
- [ ] Audit log file (JSON format)
- [ ] Log rotation support
- [ ] Syslog integration option
- [ ] Configurable log levels per module

#### CLI Improvements
- [ ] `flux-ssl-mgr list` - List all tracked certificates
- [ ] `flux-ssl-mgr search <query>` - Search certificates
- [ ] `flux-ssl-mgr validate <cert>` - Validate certificate chain
- [ ] `flux-ssl-mgr export <name>` - Export certificate bundle
- [ ] Shell completion (bash, zsh, fish)

### Technical Debt
- [ ] Refactor batch processing for better error handling
- [ ] Add retry logic for file operations
- [ ] Improve configuration validation
- [ ] Add more detailed error context
- [ ] Documentation improvements (rustdoc)

---

## Version 2.2 - Certificate Revocation & Multi-CA

**Target Release**: Q3 2025

### Goals

Add certificate revocation support and enable management of multiple intermediate CAs.

### Features

#### Certificate Revocation
- [ ] CRL (Certificate Revocation List) generation
- [ ] `flux-ssl-mgr revoke <name>` command
- [ ] Revocation reason tracking
- [ ] CRL publication to file/HTTP
- [ ] OCSP responder support (optional)

#### Multiple CA Support
- [ ] Support multiple intermediate CAs in config
- [ ] CA selection during certificate generation
- [ ] CA-specific configuration profiles
- [ ] Cross-CA certificate migration
- [ ] CA certificate chain validation

#### Batch Enhancements
- [ ] Progress bar for batch operations (indicatif)
- [ ] Dry-run mode (`--dry-run`)
- [ ] Resume interrupted batch operations
- [ ] Batch operation reports (JSON/CSV)
- [ ] Concurrent batch processing optimization

#### API & Automation
- [ ] Library API for programmatic use
- [ ] Rust API documentation
- [ ] Example code for common operations
- [ ] Webhook notifications (optional)

---

## Version 2.3 - Advanced Features

**Target Release**: Q4 2025

### Features

#### Certificate Templates
- [ ] Predefined certificate templates
- [ ] Template configuration file
- [ ] Quick certificate generation from templates
- [ ] Template inheritance

#### Backup & Restore
- [ ] Encrypted backup of CA keys and certificates
- [ ] Backup verification
- [ ] Point-in-time restore
- [ ] Incremental backups
- [ ] S3/cloud storage support (optional)

#### Enhanced Security
- [ ] Key rotation for CA keys
- [ ] Two-person control for CA operations (optional)
- [ ] Audit log signing
- [ ] Compliance reporting (PCI-DSS, etc.)

#### Integration Features
- [ ] Certificate deployment scripts
- [ ] Service restart hooks
- [ ] Ansible module
- [ ] Terraform provider (stretch goal)

---

## Version 2.4 - Monitoring & Observability

**Target Release**: Q1 2026

### Features

#### Monitoring & Alerting
- [ ] Prometheus metrics endpoint
- [ ] Certificate expiration alerts
- [ ] CA health monitoring
- [ ] Grafana dashboard templates
- [ ] Email/Slack notifications

#### Certificate Discovery
- [ ] Scan filesystem for certificates
- [ ] Import existing certificates to database
- [ ] Detect unmanaged certificates
- [ ] Certificate usage tracking

#### Reporting
- [ ] Certificate inventory reports
- [ ] Compliance reports
- [ ] Usage statistics
- [ ] PDF/HTML report generation

---

## Version 2.5 - Database & Search

**Target Release**: Q2 2026

### Features

#### Database Enhancements
- [ ] PostgreSQL backend support
- [ ] MySQL/MariaDB backend support
- [ ] Database migrations
- [ ] Multi-user support with RBAC
- [ ] Connection pooling

#### Advanced Search
- [ ] Full-text search for certificates
- [ ] Complex query filters
- [ ] Saved searches
- [ ] Search result export

#### Performance
- [ ] Query optimization
- [ ] Caching layer
- [ ] Bulk operations optimization

---

## Version 3.0 - ACME & Web UI

**Target Release**: Q4 2026

### Goals

Major release adding ACME protocol support for Let's Encrypt integration and a web-based management interface.

### Features

#### ACME Protocol Support
- [ ] ACME v2 client implementation
- [ ] Let's Encrypt integration
- [ ] Automatic certificate renewal
- [ ] DNS-01 challenge support
- [ ] HTTP-01 challenge support
- [ ] TLS-ALPN-01 challenge support
- [ ] Multi-domain certificates (SAN)
- [ ] Wildcard certificate support

#### Web UI
- [ ] Modern web interface (React/Svelte)
- [ ] Certificate management dashboard
- [ ] Interactive certificate generation
- [ ] Real-time expiration monitoring
- [ ] Certificate search and filtering
- [ ] User authentication (OAuth2/OIDC)
- [ ] Role-based access control
- [ ] Responsive design (mobile-friendly)
- [ ] Dark mode support

#### REST API
- [ ] RESTful API for all operations
- [ ] OpenAPI 3.0 specification
- [ ] API authentication (JWT)
- [ ] Rate limiting
- [ ] API versioning
- [ ] Webhook support
- [ ] API documentation site

#### Service Discovery Integration
- [ ] Consul integration
- [ ] etcd integration
- [ ] Kubernetes integration
- [ ] Docker Swarm integration
- [ ] Automatic certificate deployment

---

## Version 3.1+ - Enterprise Features

**Target Release**: 2027+

### Features

#### Hardware Security Module (HSM) Support
- [ ] PKCS#11 interface
- [ ] AWS CloudHSM integration
- [ ] Azure Key Vault integration
- [ ] Google Cloud KMS integration
- [ ] YubiHSM2 support

#### High Availability
- [ ] Active-passive HA setup
- [ ] Active-active clustering
- [ ] Distributed certificate database
- [ ] Load balancing
- [ ] Automatic failover

#### Plugin System
- [ ] Plugin API
- [ ] Custom validators
- [ ] Custom storage backends
- [ ] Custom notification channels
- [ ] Plugin marketplace

#### Advanced PKI Features
- [ ] Online Certificate Status Protocol (OCSP)
- [ ] Time-stamping authority (TSA)
- [ ] Code signing certificates
- [ ] S/MIME certificates
- [ ] Client authentication certificates
- [ ] Certificate transparency logging

---

## Community & Contribution Goals

### Documentation
- [ ] Comprehensive user guide
- [ ] Administrator handbook
- [ ] API reference documentation
- [ ] Video tutorials
- [ ] Blog posts and articles
- [ ] Migration guides
- [ ] Best practices guide

### Community Building
- [ ] Regular release schedule
- [ ] Public roadmap updates
- [ ] Community forum/Discord
- [ ] Contribution guidelines
- [ ] Code of conduct
- [ ] Maintainer documentation
- [ ] Regular community calls

### Ecosystem
- [ ] Docker images
- [ ] Helm charts (Kubernetes)
- [ ] Ansible role
- [ ] Puppet module
- [ ] Chef cookbook
- [ ] NixOS package
- [ ] Homebrew formula

---

## Technical Improvements (Ongoing)

### Code Quality
- [ ] Maintain 80%+ test coverage
- [ ] Regular dependency updates
- [ ] Security audit (annual)
- [ ] Performance benchmarking
- [ ] Code review process
- [ ] Automated security scanning

### CI/CD
- [ ] Enhanced GitHub Actions workflows
  - [ ] Automated testing on multiple platforms
  - [ ] Code coverage reporting (codecov)
  - [ ] Clippy linting
  - [ ] Format checking
  - [ ] Security scanning (cargo-audit)
  - [ ] Binary releases
  - [ ] Docker image builds
  - [ ] Documentation deployment
- [ ] Pre-commit hooks
- [ ] Automated changelog generation

### Platform Support
- [ ] Linux (primary)
  - [x] Ubuntu/Debian
  - [x] RHEL/Fedora
  - [ ] Alpine Linux
  - [ ] Arch Linux
- [ ] macOS
  - [x] Intel
  - [ ] Apple Silicon (ARM64)
- [ ] Windows
  - [ ] Native Windows support
  - [ ] WSL2 optimization
- [ ] FreeBSD (community support)

### Performance
- [ ] Benchmark suite
- [ ] Performance regression testing
- [ ] Memory usage optimization
- [ ] CPU usage optimization
- [ ] Parallel processing improvements

---

## Research & Innovation

### Future Exploration

- **Post-Quantum Cryptography**
  - Research NIST PQC standards
  - Hybrid certificate schemes
  - Migration planning

- **Certificate Automation**
  - Machine learning for certificate patterns
  - Predictive renewal recommendations
  - Anomaly detection

- **Zero-Trust PKI**
  - Short-lived certificates
  - Just-in-time certificate issuance
  - Identity-aware certificate management

- **Blockchain Integration**
  - Certificate transparency on blockchain
  - Decentralized CA verification
  - Immutable audit trails

---

## How to Contribute

We welcome contributions to help achieve these roadmap goals!

### Ways to Contribute

1. **Code Contributions**
   - Pick an issue from the roadmap
   - Submit pull requests
   - Review code from other contributors

2. **Documentation**
   - Improve existing docs
   - Write tutorials and guides
   - Create examples

3. **Testing**
   - Write tests for new features
   - Report bugs
   - Test beta releases

4. **Design & UX**
   - UI/UX design for web interface
   - CLI usability improvements
   - Error message improvements

5. **Community**
   - Answer questions in discussions
   - Help other users
   - Share your use cases

### Getting Started

1. Read [CONTRIBUTING.md](CONTRIBUTING.md) (if available)
2. Join our community discussions
3. Pick a "good first issue"
4. Ask questions - we're here to help!

---

## Changelog

### 2025-01-21
- Created initial roadmap
- Defined milestones for v2.1 - v3.1
- Outlined community and technical goals

---

## Feedback

This roadmap is a living document. We welcome feedback and suggestions!

- **GitHub Issues**: https://github.com/ethanbissbort/flux-ssl-mgr/issues
- **GitHub Discussions**: https://github.com/ethanbissbort/flux-ssl-mgr/discussions

---

**Note**: Dates and features are subject to change based on community feedback, contributor availability, and emerging requirements.

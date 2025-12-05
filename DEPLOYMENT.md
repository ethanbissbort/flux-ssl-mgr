# Flux SSL Manager - Web Service Deployment Guide

This guide provides comprehensive instructions for deploying the Flux SSL Manager web service in production environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building from Source](#building-from-source)
- [Configuration](#configuration)
- [Deployment Options](#deployment-options)
  - [Systemd Service](#systemd-service)
  - [Docker Container](#docker-container)
  - [Reverse Proxy (nginx)](#reverse-proxy-nginx)
- [Security Considerations](#security-considerations)
- [Monitoring](#monitoring)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements

- **OS**: Linux (Ubuntu 20.04+, Debian 11+, RHEL 8+, or similar)
- **CPU**: 1+ cores
- **RAM**: 512MB minimum, 1GB recommended
- **Disk**: 100MB for application + space for certificates
- **Network**: Port 8443 (or your chosen port) available

### Software Dependencies

- **Rust**: 1.70+ (for building from source)
- **OpenSSL**: 1.1.1+ or 3.0+
- **CA Infrastructure**: Existing PKI with intermediate CA

---

## Building from Source

### 1. Clone the Repository

```bash
git clone https://github.com/ethanbissbort/flux-ssl-mgr.git
cd flux-ssl-mgr
```

### 2. Build with Web Feature

```bash
# Debug build (for testing)
cargo build --features web

# Release build (for production)
cargo build --release --features web
```

### 3. Verify Build

```bash
./target/release/flux-ssl-mgr --version
```

---

## Configuration

### 1. Create Configuration File

Create `/etc/flux-ssl-mgr/config.toml`:

```toml
# PKI Configuration
working_dir = "/root/ca"
output_dir = "/var/lib/flux-ssl-mgr/output"
csr_input_dir = "/var/lib/flux-ssl-mgr/csr"

# CA Paths
ca_key_path = "/root/ca/intermediate/private/intermediate.key.pem"
ca_cert_path = "/root/ca/intermediate/certs/intermediate.cert.pem"
openssl_config = "/root/ca/intermediate/openssl.cnf"

# Default Settings
[defaults]
key_size = 4096
cert_days = 375
hash_algorithm = "sha256"
owner = "fluxadmin"
group = "root"

# Permissions (octal)
[permissions]
private_key = 0o400
certificate = 0o644
output_dir = 0o755
```

### 2. Set Permissions

```bash
sudo chmod 600 /etc/flux-ssl-mgr/config.toml
sudo chown root:root /etc/flux-ssl-mgr/config.toml
```

### 3. Create Working Directories

```bash
sudo mkdir -p /var/lib/flux-ssl-mgr/{output,csr}
sudo chown fluxadmin:root /var/lib/flux-ssl-mgr/{output,csr}
sudo chmod 755 /var/lib/flux-ssl-mgr/{output,csr}
```

---

## Deployment Options

### Systemd Service

#### 1. Create Service File

Create `/etc/systemd/system/flux-ssl-mgr.service`:

```ini
[Unit]
Description=Flux SSL Manager Web Service
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/flux-ssl-mgr

# Run the web service
ExecStart=/opt/flux-ssl-mgr/flux-ssl-mgr serve --bind 127.0.0.1 --port 8443

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/flux-ssl-mgr

# Restart policy
Restart=on-failure
RestartSec=5s

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=flux-ssl-mgr

[Install]
WantedBy=multi-user.target
```

#### 2. Install and Enable Service

```bash
# Copy binary to system location
sudo mkdir -p /opt/flux-ssl-mgr
sudo cp target/release/flux-ssl-mgr /opt/flux-ssl-mgr/
sudo chmod 755 /opt/flux-ssl-mgr/flux-ssl-mgr

# Copy static files and templates
sudo cp -r static /opt/flux-ssl-mgr/
sudo cp -r templates /opt/flux-ssl-mgr/

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable flux-ssl-mgr
sudo systemctl start flux-ssl-mgr
```

#### 3. Verify Service Status

```bash
sudo systemctl status flux-ssl-mgr
journalctl -u flux-ssl-mgr -f
```

---

### Docker Container

#### 1. Create Dockerfile

Create `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

RUN cargo build --release --features web

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/flux-ssl-mgr /usr/local/bin/
COPY static /app/static
COPY templates /app/templates

# Create non-root user
RUN useradd -m -u 1000 fluxadmin

# Create directories
RUN mkdir -p /var/lib/flux-ssl-mgr/{output,csr} && \
    chown -R fluxadmin:fluxadmin /var/lib/flux-ssl-mgr

USER fluxadmin

EXPOSE 8443

CMD ["flux-ssl-mgr", "serve", "--bind", "0.0.0.0", "--port", "8443"]
```

#### 2. Create docker-compose.yml

```yaml
version: '3.8'

services:
  flux-ssl-mgr:
    build: .
    container_name: flux-ssl-mgr
    restart: unless-stopped
    ports:
      - "127.0.0.1:8443:8443"
    volumes:
      - ./config.toml:/etc/flux-ssl-mgr/config.toml:ro
      - /root/ca:/root/ca:ro
      - flux-data:/var/lib/flux-ssl-mgr
    environment:
      - RUST_LOG=info

volumes:
  flux-data:
```

#### 3. Build and Run

```bash
docker-compose up -d
docker-compose logs -f
```

---

### Reverse Proxy (nginx)

#### 1. Install nginx

```bash
sudo apt-get install nginx
```

#### 2. Configure nginx

Create `/etc/nginx/sites-available/flux-ssl-mgr`:

```nginx
upstream flux_ssl_backend {
    server 127.0.0.1:8443;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name certs.example.com;

    # SSL Configuration
    ssl_certificate /etc/ssl/certs/certs.example.com.pem;
    ssl_certificate_key /etc/ssl/private/certs.example.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Logging
    access_log /var/log/nginx/flux-ssl-mgr-access.log;
    error_log /var/log/nginx/flux-ssl-mgr-error.log;

    # Client body size (for file uploads)
    client_max_body_size 5M;

    # Proxy to Flux SSL Manager
    location / {
        proxy_pass http://flux_ssl_backend;
        proxy_http_version 1.1;

        # Headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Static files (optional: serve directly from nginx for better performance)
    location /static/ {
        alias /opt/flux-ssl-mgr/static/;
        expires 1d;
        add_header Cache-Control "public, immutable";
    }
}

# HTTP to HTTPS redirect
server {
    listen 80;
    listen [::]:80;
    server_name certs.example.com;
    return 301 https://$server_name$request_uri;
}
```

#### 3. Enable and Test

```bash
sudo ln -s /etc/nginx/sites-available/flux-ssl-mgr /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## Security Considerations

### 1. File Permissions

```bash
# Protect CA private key
sudo chmod 400 /root/ca/intermediate/private/intermediate.key.pem
sudo chown root:root /root/ca/intermediate/private/intermediate.key.pem

# Protect configuration
sudo chmod 600 /etc/flux-ssl-mgr/config.toml
```

### 2. Firewall Configuration

```bash
# Allow only from local network
sudo ufw allow from 192.168.1.0/24 to any port 8443
sudo ufw enable
```

### 3. Authentication (Future Enhancement)

Currently, the web service does not include authentication. For production use, consider:

- Adding nginx basic authentication
- Implementing reverse proxy with SSO
- VPN access only
- IP whitelisting

### 4. Rate Limiting

Configure nginx rate limiting:

```nginx
http {
    limit_req_zone $binary_remote_addr zone=flux_limit:10m rate=10r/s;

    server {
        location /api/ {
            limit_req zone=flux_limit burst=20 nodelay;
            # ... rest of config
        }
    }
}
```

---

## Monitoring

### 1. Health Check

```bash
curl http://localhost:8443/api/health
```

Expected response:
```json
{
  "status": "healthy",
  "version": "2.0.0"
}
```

### 2. Systemd Monitoring

```bash
# Service status
sudo systemctl status flux-ssl-mgr

# Live logs
journalctl -u flux-ssl-mgr -f

# Recent errors
journalctl -u flux-ssl-mgr -p err --since today
```

### 3. Resource Usage

```bash
# Memory usage
ps aux | grep flux-ssl-mgr

# Open files/connections
sudo lsof -p $(pgrep flux-ssl-mgr)
```

### 4. Prometheus Metrics (Future Enhancement)

Future versions will expose metrics at `/metrics`:

- Request count
- Response times
- Error rates
- Certificate generation count

---

## Troubleshooting

### Service Won't Start

**Check logs**:
```bash
journalctl -u flux-ssl-mgr -n 50
```

**Common issues**:
1. Port already in use
2. Missing configuration file
3. CA files not accessible
4. Permission denied on working directories

### Cannot Access Web Interface

**Check service is running**:
```bash
sudo systemctl status flux-ssl-mgr
curl http://localhost:8443/api/health
```

**Check firewall**:
```bash
sudo ufw status
```

**Check nginx** (if using reverse proxy):
```bash
sudo nginx -t
sudo systemctl status nginx
```

### Certificate Signing Fails

**Verify CA files**:
```bash
openssl x509 -in /root/ca/intermediate/certs/intermediate.cert.pem -noout -text
openssl rsa -in /root/ca/intermediate/private/intermediate.key.pem -check
```

**Check permissions**:
```bash
ls -l /root/ca/intermediate/private/intermediate.key.pem
ls -l /root/ca/intermediate/certs/intermediate.cert.pem
```

### File Upload Fails

**Check nginx client_max_body_size**:
```nginx
client_max_body_size 5M;
```

**Check disk space**:
```bash
df -h /var/lib/flux-ssl-mgr
```

---

## Performance Tuning

### 1. Increase File Descriptors

Edit `/etc/systemd/system/flux-ssl-mgr.service`:

```ini
[Service]
LimitNOFILE=65536
```

### 2. Optimize nginx Worker Processes

```nginx
worker_processes auto;
worker_connections 1024;
```

### 3. Enable HTTP/2

Already enabled in the nginx config above.

---

## Backup and Recovery

### 1. Backup Configuration

```bash
sudo tar -czf flux-ssl-mgr-backup.tar.gz \
    /etc/flux-ssl-mgr/config.toml \
    /var/lib/flux-ssl-mgr/
```

### 2. Restore

```bash
sudo tar -xzf flux-ssl-mgr-backup.tar.gz -C /
sudo systemctl restart flux-ssl-mgr
```

---

## Support

For issues and questions:

- GitHub Issues: https://github.com/ethanbissbort/flux-ssl-mgr/issues
- Documentation: See `web-roadmap.md` for detailed API documentation
- Logs: Check `journalctl -u flux-ssl-mgr` for service logs

---

**Last Updated**: December 5, 2025
**Version**: 2.0.0

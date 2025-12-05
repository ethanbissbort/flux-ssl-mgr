# Critical Fixes Required - Action Plan

This document provides actionable fixes for the critical issues identified in the audit.

## 🔴 High Priority Fixes

### 1. Fix Parallel Batch Password Protection Bug

**File:** `src/batch.rs:206-216`

**Problem:** When batch processing with password protection in parallel mode, multiple threads try to prompt for password simultaneously, causing race condition.

**Fix Options:**

**Option A - Disable password protection in parallel mode (recommended):**
```rust
// In batch_process function, add check:
if config.batch.parallel && password_protect {
    return Err(FluxError::InvalidConfigValue(
        "batch processing".to_string(),
        "Password protection is not supported in parallel mode. Use --no-parallel or disable password protection.".to_string()
    ));
}
```

**Option B - Prompt for password once before parallel processing:**
```rust
let password = if password_protect {
    use dialoguer::Password;
    Some(Password::new()
        .with_prompt("Enter password for all certificates")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?)
} else {
    None
};

// Pass password to process_certificate calls
```

---

### 2. Implement Proper Password Security

**Files:** `src/crypto/key.rs`, `src/batch.rs`, `src/ca/intermediate.rs`

**Problem:** `secrecy` and `zeroize` crates are imported but not used. Passwords stored as plain `String`.

**Fix:**

```rust
// In src/crypto/key.rs
use secrecy::{Secret, ExposeSecret};
use zeroize::Zeroize;

pub fn save_private_key<P: AsRef<Path>>(
    key: &PKey<openssl::pkey::Private>,
    path: P,
    password: Option<&Secret<String>>, // Changed from &str
) -> Result<()> {
    let pem_bytes = if let Some(pwd) = password {
        key.private_key_to_pem_pkcs8_passphrase(
            Cipher::aes_256_cbc(),
            pwd.expose_secret().as_bytes() // Use expose_secret()
        )?
    } else {
        key.private_key_to_pem_pkcs8()?
    };

    std::fs::write(path.as_ref(), &pem_bytes)
        .map_err(|e| FluxError::FileWriteFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    Ok(())
}

// Update load_private_key similarly
pub fn load_private_key<P: AsRef<Path>>(
    path: P,
    password: Option<&Secret<String>>,
) -> Result<PKey<openssl::pkey::Private>> {
    let pem_bytes = std::fs::read(path.as_ref())
        .map_err(|e| FluxError::FileReadFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    let key = if let Some(pwd) = password {
        PKey::private_key_from_pem_passphrase(&pem_bytes, pwd.expose_secret().as_bytes())?
    } else {
        match PKey::private_key_from_pem(&pem_bytes) {
            Ok(k) => k,
            Err(_) => {
                return Err(FluxError::CaKeyUnlockFailed);
            }
        }
    };

    Ok(key)
}
```

**Update all callers to use `Secret<String>`:**

```rust
// In src/batch.rs:96-105
let password = if password_protect {
    use dialoguer::Password;
    use secrecy::Secret;

    let pwd = Password::new()
        .with_prompt(&format!("Enter password for {}", cert_name))
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;
    Some(Secret::new(pwd))
} else {
    None
};

let key = generate_rsa_key(config.defaults.key_size, None)?;
save_private_key(&key, &key_path, password.as_ref())?;
```

---

### 3. Add CA Certificate Validation

**File:** `src/ca/intermediate.rs`

**Problem:** No validation that CA certificate is valid and matches the CA key.

**Fix:**

```rust
impl IntermediateCA {
    pub fn load(config: &Config) -> Result<Self> {
        // Load CA certificate
        let cert = load_cert(&config.ca_cert_path)?;

        // Check CA certificate expiration
        if is_cert_expired(&cert)? {
            return Err(FluxError::CertParseError(
                "CA certificate is expired".to_string()
            ));
        }

        // Warn if expiring soon
        let days_left = days_until_expiration(&cert)?;
        if days_left < 30 {
            tracing::warn!("CA certificate expires in {} days", days_left);
        }

        // Check if CA key is encrypted
        let is_encrypted = is_key_encrypted(&config.ca_key_path)?;

        let (key, temp_file) = if is_encrypted {
            use dialoguer::Password;
            let password = Password::new()
                .with_prompt("Enter intermediate CA private key password")
                .interact()
                .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

            let (key, temp) = unlock_ca_key(&config.ca_key_path, &password)?;
            (key, Some(temp))
        } else {
            let key = load_private_key(&config.ca_key_path, None)?;
            (key, None)
        };

        // Verify CA key matches CA certificate
        if !cert.verify(&key).map_err(|e| FluxError::CertParseError(e.to_string()))? {
            return Err(FluxError::CaKeyUnlockFailed);
        }

        Ok(Self {
            key,
            cert,
            _temp_file: temp_file,
        })
    }
}
```

**Add missing imports to crypto/mod.rs:**
```rust
pub use cert::{
    sign_csr, save_cert_pem, save_cert_der, load_cert,
    get_cert_info, is_cert_expired, days_until_expiration
};
```

---

## 🟡 Medium Priority Fixes

### 4. Fix Certificate File Permissions

**File:** `src/config.rs:166`

**Problem:** Certificates have execute bit set (0o755).

**Fix:**

```rust
fn default_certificate_perms() -> u32 { 0o644 } // Changed from 0o755
```

---

### 5. Remove/Fix Unused Password Parameter

**File:** `src/crypto/key.rs:12`

**Problem:** `_password` parameter in `generate_rsa_key` is unused and misleading.

**Fix Option A - Remove parameter (recommended):**
```rust
pub fn generate_rsa_key(key_size: u32) -> Result<PKey<openssl::pkey::Private>> {
    // Generate RSA key
    let rsa = Rsa::generate(key_size)
        .map_err(|e| FluxError::KeyGenerationFailed(e.to_string()))?;

    // Convert to PKey
    let pkey = PKey::from_rsa(rsa)
        .map_err(|e| FluxError::KeyGenerationFailed(e.to_string()))?;

    Ok(pkey)
}
```

**Update all callers:**
```rust
// In batch.rs:107
let key = generate_rsa_key(config.defaults.key_size)?;
```

**Fix Option B - Document why it's ignored:**
```rust
/// Generate an RSA private key
///
/// Note: password parameter is intentionally ignored. Encryption is applied
/// when saving the key using save_private_key().
pub fn generate_rsa_key(key_size: u32, _password: Option<&str>) -> Result<PKey<openssl::pkey::Private>> {
    // ...
}
```

---

### 6. Add Input Validation for SANs

**File:** `src/crypto/csr.rs`

**Problem:** No validation of IP address or DNS name formats.

**Add to Cargo.toml:**
```toml
[dependencies]
ipnetwork = "0.20"  # For IP validation
```

**Fix:**

```rust
impl SanEntry {
    /// Parse SAN entry from string with validation
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(FluxError::InvalidSanFormat(s.to_string()));
        }

        let san_type = parts[0].to_uppercase();
        let value = parts[1].to_string();

        match san_type.as_str() {
            "DNS" => {
                // Basic DNS validation
                if value.is_empty() || value.len() > 253 {
                    return Err(FluxError::InvalidSanFormat(
                        format!("Invalid DNS name length: {}", value)
                    ));
                }
                // Check for valid characters
                if !value.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '*') {
                    return Err(FluxError::InvalidSanFormat(
                        format!("Invalid DNS name characters: {}", value)
                    ));
                }
                Ok(SanEntry::Dns(value))
            }
            "IP" => {
                // Validate IP address format
                use std::net::IpAddr;
                value.parse::<IpAddr>()
                    .map_err(|_| FluxError::InvalidSanFormat(
                        format!("Invalid IP address: {}", value)
                    ))?;
                Ok(SanEntry::Ip(value))
            }
            "EMAIL" => {
                // Basic email validation
                if !value.contains('@') {
                    return Err(FluxError::InvalidSanFormat(
                        format!("Invalid email: {}", value)
                    ));
                }
                Ok(SanEntry::Email(value))
            }
            _ => Err(FluxError::InvalidSanFormat(
                format!("Unknown SAN type: {}", san_type)
            )),
        }
    }
}
```

---

## 🟢 Low Priority Fixes

### 7. Remove Unused Imports

**Fix:**
```bash
cargo fix --lib -p flux-ssl-mgr
```

Or manually remove:
- `src/crypto/key.rs:7` - Remove `ExposeSecret` (if not using Secret)
- `src/crypto/key.rs:9` - Remove `zeroize::Zeroize` (if not using)
- `src/crypto/csr.rs:4` - Remove `X509Name`
- `src/ca/intermediate.rs:8` - Remove `Secret, ExposeSecret` (if not using)
- `src/ca/intermediate.rs:9` - Remove `std::path::PathBuf`

---

### 8. Add CSR Directory Validation

**File:** `src/config.rs:214`

**Fix:**

```rust
pub fn validate(&self) -> Result<()> {
    // Check if working directory exists
    if !self.working_dir.exists() {
        return Err(FluxError::WorkingDirNotFound(self.working_dir.clone()));
    }

    // Check if CA key exists
    if !self.ca_key_path.exists() {
        return Err(FluxError::CaKeyNotFound(self.ca_key_path.clone()));
    }

    // Check if CA cert exists
    if !self.ca_cert_path.exists() {
        return Err(FluxError::CaCertNotFound(self.ca_cert_path.clone()));
    }

    // Check if OpenSSL config exists
    if !self.openssl_config.exists() {
        return Err(FluxError::OpenSslConfigNotFound(self.openssl_config.clone()));
    }

    // NEW: Check if CSR input directory exists
    if !self.csr_input_dir.exists() {
        tracing::warn!("CSR input directory does not exist: {}", self.csr_input_dir.display());
        // Don't fail validation, just warn
    }

    Ok(())
}
```

---

### 9. Decision on Ownership Code

**File:** `src/batch.rs:169-182`

**Options:**

**Option A - Implement it:**
```toml
# Add to Cargo.toml
[dependencies]
users = "0.11"
nix = { version = "0.27", features = ["user"] }
```

```rust
// Uncomment and update code in batch.rs
#[cfg(all(unix, not(target_os = "macos")))]
{
    use users::{get_user_by_name, get_group_by_name};
    use nix::unistd::{chown, Uid, Gid};

    if let (Some(user), Some(group)) = (
        get_user_by_name(&config.defaults.owner),
        get_group_by_name(&config.defaults.group),
    ) {
        let uid = Uid::from_raw(user.uid());
        let gid = Gid::from_raw(group.gid());

        let _ = chown(&output_cert_pem, Some(uid), Some(gid));
        let _ = chown(&output_cert_crt, Some(uid), Some(gid));
        let _ = chown(&output_key, Some(uid), Some(gid));
    } else {
        tracing::warn!("Could not find user/group for ownership change");
    }
}
```

**Option B - Remove it:**
```rust
// Delete lines 169-182 in src/batch.rs
```

---

## Testing Additions Needed

### Add Integration Tests

**Create:** `tests/integration_test.rs`

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_config_init() {
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.toml");

    let mut cmd = Command::cargo_bin("flux-ssl-mgr").unwrap();
    cmd.arg("config")
        .arg("--init")
        .arg("--output")
        .arg(&config_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Created default configuration"));

    assert!(config_path.exists());
}

#[test]
fn test_config_show() {
    let mut cmd = Command::cargo_bin("flux-ssl-mgr").unwrap();
    cmd.arg("config").arg("--show");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Current Configuration"));
}
```

---

## Execution Order

1. ✅ Remove unused imports (quick win)
2. ✅ Fix certificate permissions (one line change)
3. ✅ Fix password parameter issue (decide option A or B)
4. ✅ Implement password security with Secret/Zeroize
5. ✅ Add CA validation checks
6. ✅ Fix parallel batch password bug
7. ✅ Add SAN input validation
8. ✅ Add CSR directory validation
9. ✅ Decide on ownership code
10. ✅ Add comprehensive tests

---

## Verification Steps

After each fix:

```bash
# 1. Check compilation
cargo build

# 2. Run tests
cargo test

# 3. Check for warnings
cargo clippy

# 4. Format code
cargo fmt

# 5. Run specific test
cargo test test_name
```

---

**End of Fixes Document**

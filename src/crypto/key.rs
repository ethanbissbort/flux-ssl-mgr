//! Private key generation and management

use crate::error::{FluxError, Result};
use openssl::rsa::Rsa;
use openssl::pkey::PKey;
use openssl::symm::Cipher;
use secrecy::{Secret, ExposeSecret};
use std::path::Path;
use zeroize::Zeroize;

/// Generate an RSA private key
pub fn generate_rsa_key(key_size: u32, _password: Option<&str>) -> Result<PKey<openssl::pkey::Private>> {
    // Generate RSA key
    let rsa = Rsa::generate(key_size)
        .map_err(|e| FluxError::KeyGenerationFailed(e.to_string()))?;

    // Convert to PKey
    let pkey = PKey::from_rsa(rsa)
        .map_err(|e| FluxError::KeyGenerationFailed(e.to_string()))?;

    Ok(pkey)
}

/// Save private key to file
pub fn save_private_key<P: AsRef<Path>>(
    key: &PKey<openssl::pkey::Private>,
    path: P,
    password: Option<&str>,
) -> Result<()> {
    let pem_bytes = if let Some(pwd) = password {
        // Encrypt with AES-256
        key.private_key_to_pem_pkcs8_passphrase(Cipher::aes_256_cbc(), pwd.as_bytes())?
    } else {
        // No encryption
        key.private_key_to_pem_pkcs8()?
    };

    std::fs::write(path.as_ref(), &pem_bytes)
        .map_err(|e| FluxError::FileWriteFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    Ok(())
}

/// Convert private key to PEM bytes (unencrypted)
pub fn to_pem(key: &PKey<openssl::pkey::Private>) -> Result<Vec<u8>> {
    key.private_key_to_pem_pkcs8()
        .map_err(|e| FluxError::KeyGenerationFailed(e.to_string()))
}

/// Convert private key to encrypted PEM bytes
pub fn to_encrypted_pem(key: &PKey<openssl::pkey::Private>, password: &Secret<String>) -> Result<Vec<u8>> {
    key.private_key_to_pem_pkcs8_passphrase(Cipher::aes_256_cbc(), password.expose_secret().as_bytes())
        .map_err(|e| FluxError::KeyGenerationFailed(e.to_string()))
}

/// Load private key from file
pub fn load_private_key<P: AsRef<Path>>(
    path: P,
    password: Option<&str>,
) -> Result<PKey<openssl::pkey::Private>> {
    let pem_bytes = std::fs::read(path.as_ref())
        .map_err(|e| FluxError::FileReadFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    let key = if let Some(pwd) = password {
        PKey::private_key_from_pem_passphrase(&pem_bytes, pwd.as_bytes())?
    } else {
        // Try without password first
        match PKey::private_key_from_pem(&pem_bytes) {
            Ok(k) => k,
            Err(_) => {
                // If it fails, the key might be encrypted
                return Err(FluxError::CaKeyUnlockFailed);
            }
        }
    };

    Ok(key)
}

/// Check if a private key is password protected
pub fn is_key_encrypted<P: AsRef<Path>>(path: P) -> Result<bool> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| FluxError::FileReadFailed(
            path.as_ref().to_path_buf(),
            e.to_string()
        ))?;

    // Check for encryption headers in PEM format
    Ok(content.contains("ENCRYPTED"))
}

/// Securely prompt for password
pub fn prompt_password(prompt: &str) -> Result<Secret<String>> {
    use dialoguer::Password;

    let password = Password::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

    Ok(Secret::new(password))
}

/// Prompt for password with confirmation
pub fn prompt_password_with_confirmation(prompt: &str) -> Result<Secret<String>> {
    use dialoguer::Password;

    let password = Password::new()
        .with_prompt(prompt)
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

    Ok(Secret::new(password))
}

/// Create a temporary unlocked copy of a CA key
pub fn unlock_ca_key<P: AsRef<Path>>(
    key_path: P,
    password: &str,
) -> Result<(PKey<openssl::pkey::Private>, tempfile::NamedTempFile)> {
    // Load the encrypted key
    let key = load_private_key(&key_path, Some(password))?;

    // Create a temporary file
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| FluxError::IoError(e))?;

    // Write unencrypted key to temp file
    let pem_bytes = key.private_key_to_pem_pkcs8()?;
    std::fs::write(temp_file.path(), &pem_bytes)
        .map_err(|e| FluxError::FileWriteFailed(
            temp_file.path().to_path_buf(),
            e.to_string()
        ))?;

    // Set restrictive permissions (600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(temp_file.path())?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(temp_file.path(), perms)?;
    }

    Ok((key, temp_file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_rsa_key() {
        let key = generate_rsa_key(2048, None).unwrap();
        assert!(key.rsa().is_ok());
    }

    #[test]
    fn test_save_and_load_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let key_path = temp_dir.path().join("test.key");

        let key = generate_rsa_key(2048, None).unwrap();
        save_private_key(&key, &key_path, None).unwrap();

        let loaded_key = load_private_key(&key_path, None).unwrap();
        assert!(loaded_key.rsa().is_ok());
    }

    #[test]
    fn test_encrypted_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let key_path = temp_dir.path().join("test_enc.key");

        let key = generate_rsa_key(2048, None).unwrap();
        save_private_key(&key, &key_path, Some("testpass")).unwrap();

        assert!(is_key_encrypted(&key_path).unwrap());

        let loaded_key = load_private_key(&key_path, Some("testpass")).unwrap();
        assert!(loaded_key.rsa().is_ok());
    }
}

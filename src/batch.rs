//! Batch processing module for multiple certificates

use crate::config::Config;
use crate::ca::IntermediateCA;
use crate::crypto::{SanEntry, create_csr, save_csr, sign_csr, save_cert_pem, generate_rsa_key, save_private_key};
use crate::error::{FluxError, Result};
use crate::output::OutputFormatter;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a CSR file to be processed
#[derive(Debug, Clone)]
pub struct CsrFile {
    pub path: PathBuf,
    pub name: String,
}

/// Batch processing result
#[derive(Debug)]
pub struct BatchResult {
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<(String, String)>,
}

/// Find all CSR files in a directory
pub fn find_csr_files<P: AsRef<Path>>(dir: P) -> Result<Vec<CsrFile>> {
    let mut csr_files = Vec::new();

    for entry in WalkDir::new(dir.as_ref())
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "csr" {
                    let name = entry.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    csr_files.push(CsrFile {
                        path: entry.path().to_path_buf(),
                        name,
                    });
                }
            }
        }
    }

    if csr_files.is_empty() {
        return Err(FluxError::NoCsrFilesFound(dir.as_ref().to_path_buf()));
    }

    // Sort by name
    csr_files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(csr_files)
}

/// Filter CSR files by name pattern
pub fn filter_csr_files(files: Vec<CsrFile>, pattern: &str) -> Vec<CsrFile> {
    files.into_iter()
        .filter(|f| f.name.contains(pattern))
        .collect()
}

/// Process a single certificate
pub fn process_certificate(
    cert_name: &str,
    sans: &[SanEntry],
    password_protect: bool,
    config: &Config,
    ca: &IntermediateCA,
    output: &OutputFormatter,
) -> Result<()> {
    output.info(&format!("Processing certificate: {}", cert_name));

    // Create directories if they don't exist
    let working_dir = &config.working_dir.join("intermediate");
    let private_dir = working_dir.join("private");
    let csr_dir = working_dir.join("csr");
    let certs_dir = working_dir.join("certs");

    std::fs::create_dir_all(&private_dir)?;
    std::fs::create_dir_all(&csr_dir)?;
    std::fs::create_dir_all(&certs_dir)?;
    std::fs::create_dir_all(&config.output_dir)?;

    // Generate private key
    output.step("Generating private key...");
    let password = if password_protect {
        use dialoguer::Password;
        let pwd = Password::new()
            .with_prompt(&format!("Enter password for {}", cert_name))
            .with_confirmation("Confirm password", "Passwords do not match")
            .interact()
            .map_err(|e| FluxError::InteractiveError(e.to_string()))?;
        Some(pwd)
    } else {
        None
    };

    let key = generate_rsa_key(config.defaults.key_size, password.as_deref())?;

    let key_path = private_dir.join(format!("{}.key.pem", cert_name));
    save_private_key(&key, &key_path, password.as_deref())?;

    // Set private key permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&key_path)?.permissions();
        perms.set_mode(config.permissions.private_key);
        std::fs::set_permissions(&key_path, perms)?;
    }

    output.success("Private key generated");

    // Generate CSR
    output.step("Generating certificate signing request...");
    let csr = create_csr(cert_name, &key, sans, None)?;
    let csr_path = csr_dir.join(format!("{}.csr.pem", cert_name));
    save_csr(&csr, &csr_path)?;
    output.success("CSR generated");

    // Sign certificate
    output.step("Signing certificate with intermediate CA...");
    let cert = sign_csr(&csr, ca.cert(), ca.key(), config.defaults.cert_days)?;
    output.success("Certificate signed");

    // Save certificate in PEM format
    output.step("Saving certificate...");
    let cert_pem_path = certs_dir.join(format!("{}.cert.pem", cert_name));
    save_cert_pem(&cert, &cert_pem_path)?;

    // Save certificate in CRT format (same as PEM for OpenSSL)
    let cert_crt_path = certs_dir.join(format!("{}.crt", cert_name));
    save_cert_pem(&cert, &cert_crt_path)?;

    // Copy to output directory
    let output_cert_pem = config.output_dir.join(format!("{}.cert.pem", cert_name));
    let output_cert_crt = config.output_dir.join(format!("{}.crt", cert_name));
    let output_key = config.output_dir.join(format!("{}.key.pem", cert_name));

    std::fs::copy(&cert_pem_path, &output_cert_pem)?;
    std::fs::copy(&cert_crt_path, &output_cert_crt)?;
    std::fs::copy(&key_path, &output_key)?;

    // Set permissions on output files
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Certificate permissions
        let mut cert_perms = std::fs::metadata(&output_cert_pem)?.permissions();
        cert_perms.set_mode(config.permissions.certificate);
        std::fs::set_permissions(&output_cert_pem, cert_perms.clone())?;
        std::fs::set_permissions(&output_cert_crt, cert_perms)?;

        // Key permissions
        let mut key_perms = std::fs::metadata(&output_key)?.permissions();
        key_perms.set_mode(config.permissions.private_key);
        std::fs::set_permissions(&output_key, key_perms)?;

        // Set ownership if specified
        // Note: Ownership changes require external crates (users, nix)
        // Uncomment and add dependencies if needed:
        // #[cfg(target_os = "linux")]
        // if let (Ok(uid), Ok(gid)) = (
        //     users::get_user_by_name(&config.defaults.owner).map(|u| u.uid()),
        //     users::get_group_by_name(&config.defaults.group).map(|g| g.gid()),
        // ) {
        //     // Note: This requires root privileges
        //     let _ = nix::unistd::chown(&output_cert_pem, uid, gid);
        //     let _ = nix::unistd::chown(&output_cert_crt, uid, gid);
        //     let _ = nix::unistd::chown(&output_key, uid, gid);
        // }
    }

    output.success(&format!("Certificate {} completed successfully", cert_name));

    Ok(())
}

/// Batch process multiple certificates
pub fn batch_process(
    cert_names: Vec<String>,
    common_sans: Option<Vec<SanEntry>>,
    password_protect: bool,
    config: &Config,
    output: &OutputFormatter,
) -> Result<BatchResult> {
    output.info(&format!("Starting batch processing of {} certificates", cert_names.len()));

    // Load CA once
    let ca = IntermediateCA::load(config)?;

    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    if config.batch.parallel && cert_names.len() > 1 {
        // Parallel processing (without progress bar for simplicity)
        let results: Vec<_> = cert_names.par_iter()
            .map(|name| {
                let sans = common_sans.clone().unwrap_or_default();
                match process_certificate(name, &sans, password_protect, config, &ca, output) {
                    Ok(_) => Ok(name.clone()),
                    Err(e) => Err((name.clone(), e.to_string())),
                }
            })
            .collect();

        for result in results {
            match result {
                Ok(_) => successful += 1,
                Err((name, err)) => {
                    failed += 1;
                    errors.push((name, err));
                }
            }
        }
    } else {
        // Sequential processing with progress bar
        for name in &cert_names {
            let sans = common_sans.clone().unwrap_or_default();
            match process_certificate(name, &sans, password_protect, config, &ca, output) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push((name.clone(), e.to_string()));
                }
            }
        }
    }

    Ok(BatchResult {
        successful,
        failed,
        errors,
    })
}

// Additional dependencies that might need to be added to Cargo.toml
// users = "0.11"  (for user/group lookups)
// nix = { version = "0.27", features = ["user"] }  (for chown)

//! Interactive mode for user prompts

use crate::crypto::SanEntry;
use crate::error::{FluxError, Result};
use crate::batch::CsrFile;
use dialoguer::{Input, Confirm, Select, MultiSelect};

/// Prompt for certificate name
pub fn prompt_cert_name() -> Result<String> {
    let name: String = Input::new()
        .with_prompt("Enter certificate name (e.g., myservice)")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.trim().is_empty() {
                Err("Certificate name cannot be empty")
            } else if input.contains(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.') {
                Err("Certificate name can only contain alphanumeric characters, hyphens, underscores, and dots")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

    Ok(name.trim().to_string())
}

/// Prompt for Subject Alternative Names
pub fn prompt_sans() -> Result<Vec<SanEntry>> {
    println!("\nEnter Subject Alternative Names (DNS and IP addresses)");
    println!("Example: DNS:service.fluxlab.systems,DNS:service.local,IP:10.0.2.100");

    let sans_input: String = Input::new()
        .with_prompt("SANs")
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            if input.trim().is_empty() {
                return Err("Subject Alternative Names are required".to_string());
            }
            match SanEntry::parse_multiple(input) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Invalid SAN format: {}", e)),
            }
        })
        .interact_text()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

    SanEntry::parse_multiple(&sans_input)
}

/// Prompt for password protection
pub fn prompt_password_protection() -> Result<bool> {
    Confirm::new()
        .with_prompt("Password protect the private key?")
        .default(false)
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))
}

/// Prompt for processing mode (single or batch)
pub fn prompt_processing_mode() -> Result<usize> {
    let modes = vec![
        "Single certificate (interactive)",
        "Batch process CSR files from directory",
    ];

    Select::new()
        .with_prompt("Select processing mode")
        .items(&modes)
        .default(0)
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))
}

/// Prompt for CSR directory
pub fn prompt_csr_directory(default: &str) -> Result<String> {
    let dir: String = Input::new()
        .with_prompt("Enter directory containing CSR files")
        .default(default.to_string())
        .interact_text()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

    Ok(dir)
}

/// Prompt for CSR selection
pub fn prompt_csr_selection(files: &[CsrFile]) -> Result<Vec<usize>> {
    // Display all CSR files
    let items: Vec<String> = files.iter()
        .map(|f| format!("{} ({})", f.name, f.path.display()))
        .collect();

    let selection = MultiSelect::new()
        .with_prompt("Select CSRs to process (Space to select, Enter to confirm)")
        .items(&items)
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

    if selection.is_empty() {
        return Err(FluxError::UserCancelled);
    }

    Ok(selection)
}

/// Prompt for common SANs in batch mode
pub fn prompt_use_common_sans() -> Result<bool> {
    println!("\nFor batch processing, you can set common Subject Alternative Names");
    println!("or configure each certificate individually.");

    Confirm::new()
        .with_prompt("Use common SANs for all certificates?")
        .default(false)
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))
}

/// Prompt for common SANs value
pub fn prompt_common_sans() -> Result<Vec<SanEntry>> {
    println!("\nEnter common Subject Alternative Names:");
    println!("Example: DNS:*.fluxlab.systems,IP:10.0.2.100");

    let sans_input: String = Input::new()
        .with_prompt("Common SANs")
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            if input.trim().is_empty() {
                return Ok(()); // Allow empty for no common SANs
            }
            match SanEntry::parse_multiple(input) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Invalid SAN format: {}", e)),
            }
        })
        .interact_text()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

    if sans_input.trim().is_empty() {
        Ok(vec![])
    } else {
        SanEntry::parse_multiple(&sans_input)
    }
}

/// Prompt for confirmation
pub fn prompt_confirm(message: &str) -> Result<bool> {
    Confirm::new()
        .with_prompt(message)
        .default(true)
        .interact()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))
}

/// Prompt for certificate validity days
pub fn prompt_cert_days(default: u32) -> Result<u32> {
    let days: String = Input::new()
        .with_prompt("Certificate validity in days")
        .default(default.to_string())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            match input.parse::<u32>() {
                Ok(d) if d > 0 && d <= 825 => Ok(()), // Max 825 days per CA/B Forum
                Ok(_) => Err("Days must be between 1 and 825"),
                Err(_) => Err("Please enter a valid number"),
            }
        })
        .interact_text()
        .map_err(|e| FluxError::InteractiveError(e.to_string()))?;

    days.parse::<u32>()
        .map_err(|e| FluxError::InvalidConfigValue("cert_days".to_string(), e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Interactive tests would require mocking user input
    // These are placeholder tests
}

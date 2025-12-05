//! Flux SSL Manager - CLI Entry Point

use clap::{Parser, Subcommand};
use flux_ssl_mgr::{Config, IntermediateCA, OutputFormatter, Result, FluxError};
use flux_ssl_mgr::crypto::SanEntry;
use flux_ssl_mgr::batch;
use flux_ssl_mgr::interactive;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "flux-ssl-mgr")]
#[command(version, about = "Certificate management for homelab PKI", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode (suppress non-error output)
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a single certificate
    Single {
        /// Certificate name
        #[arg(short, long)]
        name: Option<String>,

        /// Subject Alternative Names (comma-separated)
        /// Example: DNS:example.com,IP:192.168.1.1
        #[arg(short, long, value_delimiter = ',')]
        sans: Option<Vec<String>>,

        /// Password-protect the private key
        #[arg(short, long)]
        password: bool,

        /// Certificate validity in days
        #[arg(short, long)]
        days: Option<u32>,

        /// RSA key size in bits
        #[arg(short, long)]
        key_size: Option<u32>,
    },

    /// Batch process CSR files
    Batch {
        /// Directory containing CSR files
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// Process all CSRs without prompting
        #[arg(short, long)]
        all: bool,

        /// Filter CSRs by name pattern
        #[arg(short, long)]
        filter: Option<String>,

        /// Common SANs for all certificates (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        sans: Option<Vec<String>>,

        /// Password-protect all private keys
        #[arg(short, long)]
        password: bool,
    },

    /// Show certificate information
    Info {
        /// Certificate file path
        cert: PathBuf,

        /// Show full certificate details
        #[arg(short, long)]
        verbose: bool,
    },

    /// Configuration management
    Config {
        /// Initialize default configuration file
        #[arg(long)]
        init: bool,

        /// Show current configuration
        #[arg(long)]
        show: bool,

        /// Output path for configuration file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Start web service (requires 'web' feature)
    #[cfg(feature = "web")]
    Serve {
        /// Bind address
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,

        /// Port number
        #[arg(short, long, default_value = "8443")]
        port: u16,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = if cli.verbose {
        "debug".to_string()
    } else {
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    };

    tracing_subscriber::fmt()
        .with_env_filter(&log_level)
        .init();

    // Load configuration
    let mut config = if let Some(config_path) = &cli.config {
        Config::from_file(config_path)?
    } else {
        Config::load()?
    };

    // Override output settings from CLI
    if cli.verbose {
        config.output.verbose = true;
    }
    if cli.quiet {
        config.output.quiet = true;
    }

    // Create output formatter
    let output = OutputFormatter::new(&config.output);

    // Execute command
    match cli.command {
        Commands::Single { name, sans, password, days, key_size } => {
            handle_single(name, sans, password, days, key_size, config, output)
        }
        Commands::Batch { dir, all, filter, sans, password } => {
            handle_batch(dir, all, filter, sans, password, config, output)
        }
        Commands::Info { cert, verbose } => {
            handle_info(cert, verbose, output)
        }
        Commands::Config { init, show, output: output_path } => {
            handle_config(init, show, output_path, config)
        }
        #[cfg(feature = "web")]
        Commands::Serve { bind, port } => {
            handle_serve(bind, port, config)
        }
    }
}

fn handle_single(
    name: Option<String>,
    sans: Option<Vec<String>>,
    password: bool,
    days: Option<u32>,
    key_size: Option<u32>,
    mut config: Config,
    output: OutputFormatter,
) -> Result<()> {
    // Override config with CLI args if provided
    if let Some(d) = days {
        config.defaults.cert_days = d;
    }
    if let Some(k) = key_size {
        config.defaults.key_size = k;
    }

    output.header("PKI Certificate Generation");

    // Get certificate name (CLI or interactive)
    let cert_name = if let Some(n) = name {
        n
    } else {
        interactive::prompt_cert_name()?
    };

    // Get SANs (CLI or interactive)
    let san_entries = if let Some(s) = sans {
        let sans_str = s.join(",");
        SanEntry::parse_multiple(&sans_str)?
    } else {
        interactive::prompt_sans()?
    };

    // Get password protection preference (CLI or interactive)
    let use_password = if password {
        true
    } else {
        interactive::prompt_password_protection()?
    };

    // Load CA
    let ca = IntermediateCA::load(&config)?;

    // Process certificate
    batch::process_certificate(
        &cert_name,
        &san_entries,
        use_password,
        &config,
        &ca,
        &output,
    )?;

    output.print_cert_summary(&cert_name, &config.output_dir);
    output.warning("Don't forget to update your service configuration with the new certificate!");

    Ok(())
}

fn handle_batch(
    dir: Option<PathBuf>,
    all: bool,
    filter: Option<String>,
    sans: Option<Vec<String>>,
    password: bool,
    config: Config,
    output: OutputFormatter,
) -> Result<()> {
    output.header("PKI Batch Certificate Processing");

    // Get CSR directory
    let csr_dir = if let Some(d) = dir {
        d
    } else {
        PathBuf::from(interactive::prompt_csr_directory(
            config.csr_input_dir.to_str().unwrap_or("/home/fluxadmin/ssl")
        )?)
    };

    // Find CSR files
    let mut csr_files = batch::find_csr_files(&csr_dir)?;

    // Apply filter if provided
    if let Some(pattern) = filter {
        csr_files = batch::filter_csr_files(csr_files, &pattern);
        if csr_files.is_empty() {
            return Err(FluxError::NoCsrFilesFound(csr_dir));
        }
    }

    output.info(&format!("Found {} CSR files", csr_files.len()));

    // Select CSRs to process
    let selected_indices = if all {
        (0..csr_files.len()).collect()
    } else {
        interactive::prompt_csr_selection(&csr_files)?
    };

    let selected_names: Vec<String> = selected_indices.iter()
        .map(|&i| csr_files[i].name.clone())
        .collect();

    // Get common SANs
    let common_sans = if let Some(s) = sans {
        let sans_str = s.join(",");
        Some(SanEntry::parse_multiple(&sans_str)?)
    } else if interactive::prompt_use_common_sans()? {
        Some(interactive::prompt_common_sans()?)
    } else {
        None
    };

    // Process batch
    let result = batch::batch_process(
        selected_names,
        common_sans,
        password,
        &config,
        &output,
    )?;

    output.print_batch_summary(result.successful, result.failed);

    // Show errors if any
    if !result.errors.is_empty() {
        output.println("\nFailed certificates:");
        for (name, error) in result.errors {
            output.error(&format!("{}: {}", name, error));
        }
    }

    Ok(())
}

fn handle_info(cert_path: PathBuf, verbose: bool, output: OutputFormatter) -> Result<()> {
    use flux_ssl_mgr::crypto::cert::{load_cert, get_cert_info, is_cert_expired, days_until_expiration};

    let cert = load_cert(&cert_path)?;

    output.header(&format!("Certificate Information: {}", cert_path.display()));

    let info = get_cert_info(&cert)?;
    output.println(&info);

    // Check expiration
    let expired = is_cert_expired(&cert)?;
    let days_left = days_until_expiration(&cert)?;

    if expired {
        output.error(&format!("Certificate is EXPIRED (expired {} days ago)", -days_left));
    } else if days_left < 30 {
        output.warning(&format!("Certificate expires in {} days", days_left));
    } else {
        output.success(&format!("Certificate is valid ({} days remaining)", days_left));
    }

    if verbose {
        // Show additional details
        output.println("\nPublic Key Info:");
        let pubkey = cert.public_key()?;
        output.println(&format!("  Algorithm: RSA"));
        if let Ok(rsa) = pubkey.rsa() {
            output.println(&format!("  Key Size: {} bits", rsa.size() * 8));
        }
    }

    Ok(())
}

fn handle_config(init: bool, show: bool, output_path: Option<PathBuf>, config: Config) -> Result<()> {
    if init {
        let config_path = output_path.unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
                .join(".config/flux-ssl-mgr/config.toml")
        });

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create default config
        let default_config = Config::default();
        default_config.save(&config_path)?;

        println!("Created default configuration at: {}", config_path.display());
        println!("\nPlease edit this file to match your PKI setup.");
        return Ok(());
    }

    if show {
        println!("Current Configuration:");
        println!("======================");
        println!("{}", toml::to_string_pretty(&config).unwrap());
        return Ok(());
    }

    println!("Use --init to create a configuration file");
    println!("Use --show to display current configuration");

    Ok(())
}

#[cfg(feature = "web")]
fn handle_serve(bind: String, port: u16, config: Config) -> Result<()> {
    use flux_ssl_mgr::web::{start_server, ServerConfig};
    use std::sync::Arc;

    println!("Starting Flux SSL Manager web service...");
    println!("Bind address: {}:{}", bind, port);

    let server_config = ServerConfig {
        bind_address: bind,
        port,
    };

    // Create a tokio runtime
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| FluxError::IoError(e))?;

    // Run the server
    runtime.block_on(async {
        start_server(Arc::new(config), server_config).await
    })
}

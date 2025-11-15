//! Output formatting module

use console::{Style, Term};
use crate::config::OutputConfig;

/// Output formatter with color support
pub struct OutputFormatter {
    term: Term,
    colored: bool,
    verbose: bool,
    quiet: bool,
    green: Style,
    yellow: Style,
    red: Style,
    blue: Style,
}

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new(config: &OutputConfig) -> Self {
        let green = Style::new().green().bold();
        let yellow = Style::new().yellow().bold();
        let red = Style::new().red().bold();
        let blue = Style::new().blue();

        Self {
            term: Term::stdout(),
            colored: config.colored,
            verbose: config.verbose,
            quiet: config.quiet,
            green,
            yellow,
            red,
            blue,
        }
    }

    /// Print success message
    pub fn success(&self, msg: &str) {
        if self.quiet {
            return;
        }

        if self.colored {
            let _ = self.term.write_line(&format!("{} {}", self.green.apply_to("✓"), msg));
        } else {
            let _ = self.term.write_line(&format!("✓ {}", msg));
        }
    }

    /// Print error message
    pub fn error(&self, msg: &str) {
        if self.colored {
            let _ = self.term.write_line(&format!("{} {}", self.red.apply_to("✗"), msg));
        } else {
            let _ = self.term.write_line(&format!("✗ {}", msg));
        }
    }

    /// Print warning message
    pub fn warning(&self, msg: &str) {
        if self.quiet {
            return;
        }

        if self.colored {
            let _ = self.term.write_line(&format!("{} {}", self.yellow.apply_to("⚠"), msg));
        } else {
            let _ = self.term.write_line(&format!("⚠ {}", msg));
        }
    }

    /// Print info message
    pub fn info(&self, msg: &str) {
        if self.quiet {
            return;
        }

        if self.colored {
            let _ = self.term.write_line(&format!("{} {}", self.blue.apply_to("ℹ"), msg));
        } else {
            let _ = self.term.write_line(&format!("ℹ {}", msg));
        }
    }

    /// Print step message
    pub fn step(&self, msg: &str) {
        if self.quiet {
            return;
        }

        let _ = self.term.write_line(msg);
    }

    /// Print verbose message (only if verbose mode is enabled)
    pub fn verbose(&self, msg: &str) {
        if self.verbose && !self.quiet {
            if self.colored {
                let _ = self.term.write_line(&self.blue.apply_to(msg).to_string());
            } else {
                let _ = self.term.write_line(msg);
            }
        }
    }

    /// Print section header
    pub fn header(&self, msg: &str) {
        if self.quiet {
            return;
        }

        let separator = "=".repeat(msg.len());
        if self.colored {
            let _ = self.term.write_line(&self.green.apply_to(msg).to_string());
            let _ = self.term.write_line(&self.green.apply_to(&separator).to_string());
        } else {
            let _ = self.term.write_line(msg);
            let _ = self.term.write_line(&separator);
        }
    }

    /// Print raw message
    pub fn println(&self, msg: &str) {
        if !self.quiet {
            let _ = self.term.write_line(msg);
        }
    }

    /// Print without newline
    pub fn print(&self, msg: &str) {
        if !self.quiet {
            let _ = self.term.write_str(msg);
        }
    }

    /// Print certificate summary
    pub fn print_cert_summary(&self, cert_name: &str, output_dir: &std::path::Path) {
        if self.quiet {
            return;
        }

        self.println("");
        self.header(&format!("Certificate {} generation complete!", cert_name));
        self.println("Generated files:");
        self.println(&format!("  • Certificate (PEM): {}/{}.cert.pem", output_dir.display(), cert_name));
        self.println(&format!("  • Certificate (CRT): {}/{}.crt", output_dir.display(), cert_name));
        self.println(&format!("  • Private Key:       {}/{}.key.pem", output_dir.display(), cert_name));
    }

    /// Print batch summary
    pub fn print_batch_summary(&self, successful: usize, failed: usize) {
        if self.quiet {
            return;
        }

        self.println("");
        self.header("Batch processing complete!");
        self.success(&format!("Processed: {} certificates", successful));

        if failed > 0 {
            self.error(&format!("Failed: {} certificates", failed));
        }
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self {
            term: Term::stdout(),
            colored: true,
            verbose: false,
            quiet: false,
            green: Style::new().green().bold(),
            yellow: Style::new().yellow().bold(),
            red: Style::new().red().bold(),
            blue: Style::new().blue(),
        }
    }
}

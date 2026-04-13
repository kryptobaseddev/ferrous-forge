//! Template file content strings

/// CLI main content
pub fn cli_main_content() -> String {
    r#"//! Command-line interface for {{workspace_name}}
#![deny(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use {{workspace_name}}_core::{Config, Core};
use {{workspace_name}}_utils::setup_logging;

/// {{workspace_name}} CLI application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// Commands
    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand)]
enum Commands {
    /// Process data
    Process {
        /// Input to process
        #[arg(value_name = "INPUT")]
        input: String,
    },
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    setup_logging(cli.debug)?;

    let config = Config {
        name: "{{workspace_name}}".to_string(),
        debug: cli.debug,
    };

    let core = Core::new(config);

    match cli.command {
        Commands::Process { input } => {
            println!("Processing: {}", input);
            let result = core.process()?;
            println!("Result: {}", result);
        }
        Commands::Version => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
"#
    .to_string()
}

/// Core library content
pub fn core_lib_content() -> String {
    r#"//! Core functionality for {{workspace_name}}
#![deny(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use anyhow::Result;
use thiserror::Error;

/// Core application errors
#[derive(Debug, Error)]
pub enum CoreError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Processing error
    #[error("Processing failed: {0}")]
    Processing(String),
}

/// Main core functionality
pub struct Core {
    config: Config,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Application name
    pub name: String,
    /// Debug mode
    pub debug: bool,
}

impl Core {
    /// Create a new core instance
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Process data with the core functionality
    pub fn process(&self) -> Result<String> {
        // Add your core processing logic here
        Ok(format!("Processed by {}", self.config.name))
    }

    /// Get configuration reference
    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "{{workspace_name}}".to_string(),
            debug: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_creation() {
        let config = Config::default();
        let core = Core::new(config);
        assert_eq!(core.config().name, "{{workspace_name}}");
    }

    #[test]
    fn test_core_processing() {
        let config = Config::default();
        let core = Core::new(config);
        let result = core.process();
        assert!(result.is_ok());
    }
}
"#
    .to_string()
}

/// Utils library content
pub fn utils_lib_content() -> String {
    r#"//! Utility functions for {{workspace_name}}
#![deny(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use anyhow::Result;

/// Setup logging for the application
pub fn setup_logging(debug: bool) -> Result<()> {
    let level = if debug { "debug" } else { "info" };
    
    // In a real application, you'd set up a proper logger here
    // For now, we'll just print the level
    println!("Logging level set to: {}", level);
    
    Ok(())
}

/// Format a message with timestamp
pub fn format_message(message: &str) -> String {
    // In a real application, you'd use proper timestamp formatting
    // Using a simple static timestamp for now
    format!("[2024-01-01 00:00:00] {}", message)
}

/// Utility function for file operations
pub async fn read_file_async(path: &str) -> Result<String> {
    tokio::fs::read_to_string(path).await.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message() {
        let message = "Test message";
        let formatted = format_message(message);
        assert!(formatted.contains(message));
        assert!(formatted.contains('['));
        assert!(formatted.contains(']'));
    }

    #[tokio::test]
    async fn test_read_file_async() {
        // This test would need a real file to work properly
        // In a real application, you'd create a test file or use mocking
        let result = read_file_async("nonexistent.txt").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_setup_logging() {
        let result = setup_logging(true);
        assert!(result.is_ok());
    }
}
"#
    .to_string()
}

//! CLI application template definition

use crate::templates::manifest::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use crate::templates::registry::BuiltinTemplate;
use std::collections::HashMap;
use std::path::PathBuf;

/// Create the CLI application template
pub fn create_cli_template() -> BuiltinTemplate {
    let mut manifest = TemplateManifest::new("cli-app".to_string(), TemplateKind::CliApp);

    manifest.description = "Command-line application with clap and tokio".to_string();
    manifest.author = "Ferrous Forge Team".to_string();

    // Add variables
    manifest.add_variable(TemplateVariable::required(
        "project_name".to_string(),
        "Name of the project".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "author".to_string(),
        "Author name".to_string(),
        "Unknown".to_string(),
    ));

    // Add files
    manifest.add_file(TemplateFile::new(
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/main.rs"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/lib.rs"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from(".ferrous-forge/config.toml"),
        PathBuf::from(".ferrous-forge/config.toml"),
    ));

    // Add post-generate commands
    manifest.add_post_generate("cargo fmt".to_string());
    manifest.add_post_generate("ferrous-forge validate .".to_string());

    // Create file contents
    let mut files = HashMap::new();

    files.insert(
        "Cargo.toml".to_string(),
        r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"
authors = ["{{author}}"]

[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.40", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

[dev-dependencies]
tempfile = "3.10"

[[bin]]
name = "{{project_name}}"
path = "src/main.rs"
"#
        .to_string(),
    );

    files.insert(
        "src/main.rs".to_string(),
        r#"//! {{project_name}} - A Ferrous Forge compliant CLI application
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use clap::Parser;
use {{project_ident}}::{run, Config};

/// Command-line arguments
#[derive(Debug, Parser)]
#[command(author = "{{author}}", version, about)]
struct Args {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Configuration file
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse arguments
    let args = Args::parse();
    
    // Initialize tracing
    let filter = if args.verbose {
        "debug"
    } else {
        "info"
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    // Load configuration
    let config = if let Some(path) = args.config {
        Config::from_file(&path)?
    } else {
        Config::default()
    };
    
    // Run the application
    run(config).await
}
"#
        .to_string(),
    );

    files.insert(
        "src/lib.rs".to_string(),
        r#"//! Core library for {{project_name}}
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application name
    pub name: String,
    /// Debug mode
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "{{project_name}}".to_string(),
            debug: false,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}

/// Main application logic
pub async fn run(config: Config) -> Result<()> {
    tracing::info!("Starting {} with config: {:?}", config.name, config);
    
    // Application logic here
    println!("Hello from {}!", config.name);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.name, "{{project_name}}");
        assert!(!config.debug);
    }
}
"#
        .to_string(),
    );

    files.insert(
        ".ferrous-forge/config.toml".to_string(),
        r#"# Ferrous Forge configuration
[validation]
enabled = true
strict = true

[standards]
edition = "2024"
max_line_length = 100
max_function_lines = 50
max_file_lines = 300

[hooks]
pre_commit = true
pre_push = true
"#
        .to_string(),
    );

    BuiltinTemplate { manifest, files }
}

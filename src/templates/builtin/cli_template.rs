//! CLI application template

use crate::templates::{BuiltinTemplate, TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use std::collections::HashMap;
use std::path::PathBuf;

/// Create the CLI application template
pub fn create_cli_template() -> BuiltinTemplate {
    let manifest = create_cli_manifest();
    let files = create_cli_files();
    
    BuiltinTemplate { manifest, files }
}

/// Create the manifest for CLI template
fn create_cli_manifest() -> TemplateManifest {
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
    
    manifest
}

/// Create the files for CLI template
fn create_cli_files() -> HashMap<String, String> {
    let mut files = HashMap::new();
    
    files.insert("Cargo.toml".to_string(), cargo_toml_content());
    files.insert("src/main.rs".to_string(), main_rs_content());
    files.insert("src/lib.rs".to_string(), lib_rs_content());
    files.insert(".ferrous-forge/config.toml".to_string(), config_toml_content());
    
    files
}

/// Cargo.toml content
fn cargo_toml_content() -> String {
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
"#.to_string()
}

/// src/main.rs content
fn main_rs_content() -> String {
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
"#.to_string()
}

/// src/lib.rs content
fn lib_rs_content() -> String {
    r#"//! {{project_name}} library implementation
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Application configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Verbosity level
    pub verbose: bool,
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

/// Main application logic
pub async fn run(config: Config) -> Result<()> {
    tracing::info!("Running with config: {:?}", config);
    
    // TODO: Implement your application logic here
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.verbose);
    }
}
"#.to_string()
}

/// .ferrous-forge/config.toml content
fn config_toml_content() -> String {
    r#"# Ferrous Forge configuration

[validation]
max_line_length = 100
max_file_length = 300
max_function_length = 50
allow_unwrap = false
allow_expect = false
"#.to_string()
}
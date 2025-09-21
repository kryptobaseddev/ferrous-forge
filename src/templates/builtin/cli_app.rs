//! CLI application template

use super::BuiltinTemplate;
use crate::templates::manifest::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Create CLI application template
pub fn create_template() -> Result<BuiltinTemplate> {
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
"#
        .to_string(),
    );

    files.insert(
        "src/main.rs".to_string(),
        r#"//! {{project_name}} CLI application

use clap::{Parser, Subcommand};
use {{project_name}}::{Error, Result};

#[derive(Parser)]
#[command(name = "{{project_name}}")]
#[command(about = "A Ferrous Forge CLI application")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the application
    Run {
        /// Input file path
        #[arg(short, long)]
        input: Option<String>,
        
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show version information  
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { input, verbose } => {
            if verbose {
                println!("Running {{project_name}} with input: {:?}", input);
            }
            
            let result = {{project_name}}::run(input).await?;
            println!("Result: {}", result);
        }
        Commands::Version => {
            println!("{{project_name}} v{}", env!("CARGO_PKG_VERSION"));
        }
    }
    
    Ok(())
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
    /// Version
    pub version: String,
}

/// Custom error type
pub type Error = anyhow::Error;

/// Main application function
pub async fn run(input: Option<String>) -> Result<String> {
    let input_value = input.unwrap_or_else(|| "default".to_string());
    
    // Simulate some async work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(format!("Processed: {input_value}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_run() {
        let result = run(Some("test".to_string())).await.unwrap();
        assert_eq!(result, "Processed: test");
    }
    
    #[tokio::test]
    async fn test_run_default() {
        let result = run(None).await.unwrap();
        assert_eq!(result, "Processed: default");
    }
}
"#
        .to_string(),
    );

    files.insert(
        ".ferrous-forge/config.toml".to_string(),
        r#"# Ferrous Forge configuration for {{project_name}}

[validation]
enabled = true
max_line_length = 100
max_function_lines = 50
max_file_lines = 300

[safety]
enabled = true
pre_commit = true
pre_push = true

[fix]
conservative_mode = true
backup_files = true
"#
        .to_string(),
    );

    Ok(BuiltinTemplate { manifest, files })
}

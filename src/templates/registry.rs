//! Template registry for managing available templates

use super::manifest::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use crate::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Registry of available templates
pub struct TemplateRegistry {
    /// Built-in templates
    builtin: HashMap<String, BuiltinTemplate>,

    /// Custom templates from filesystem
    custom: HashMap<String, PathBuf>,
}

/// Built-in template definition
pub struct BuiltinTemplate {
    /// Template manifest
    pub manifest: TemplateManifest,

    /// Template files as strings
    pub files: HashMap<String, String>,
}

impl TemplateRegistry {
    /// Create a new registry with built-in templates
    pub fn new() -> Self {
        let mut registry = Self {
            builtin: HashMap::new(),
            custom: HashMap::new(),
        };

        // Register built-in templates
        registry.register_cli_template();
        registry.register_library_template();
        registry.register_web_service_template();

        registry
    }

    /// Register the CLI application template
    fn register_cli_template(&mut self) {
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

        self.builtin
            .insert("cli-app".to_string(), BuiltinTemplate { manifest, files });
    }

    /// Register the library template
    fn register_library_template(&mut self) {
        let mut manifest = TemplateManifest::new("library".to_string(), TemplateKind::Library);

        manifest.description = "Library crate with comprehensive testing".to_string();
        manifest.author = "Ferrous Forge Team".to_string();

        manifest.add_variable(TemplateVariable::required(
            "project_name".to_string(),
            "Name of the library".to_string(),
        ));

        manifest.add_file(TemplateFile::new(
            PathBuf::from("Cargo.toml"),
            PathBuf::from("Cargo.toml"),
        ));

        manifest.add_file(TemplateFile::new(
            PathBuf::from("src/lib.rs"),
            PathBuf::from("src/lib.rs"),
        ));

        manifest.add_file(TemplateFile::new(
            PathBuf::from("benches/benchmarks.rs"),
            PathBuf::from("benches/benchmarks.rs"),
        ));

        let mut files = HashMap::new();

        files.insert(
            "Cargo.toml".to_string(),
            r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"
description = "A Ferrous Forge compliant library"
license = "MIT OR Apache-2.0"

[dependencies]
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
criterion = "0.5"
proptest = "1.5"

[[bench]]
name = "benchmarks"
harness = false
"#
            .to_string(),
        );

        files.insert(
            "src/lib.rs".to_string(),
            r#"//! {{project_name}} - A Ferrous Forge compliant library
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Main error type for this library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Example function
pub fn example(input: &str) -> Result<String> {
    if input.is_empty() {
        return Err(Error::InvalidInput("Input cannot be empty".to_string()));
    }
    
    Ok(format!("Processed: {input}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example() {
        assert_eq!(example("test").unwrap(), "Processed: test");
    }
    
    #[test]
    fn test_empty_input() {
        assert!(example("").is_err());
    }
}
"#
            .to_string(),
        );

        files.insert(
            "benches/benchmarks.rs".to_string(),
            r#"//! Benchmarks for {{project_name}}

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use {{project_ident}}::*;

fn bench_example(c: &mut Criterion) {
    c.bench_function("example", |b| {
        b.iter(|| example(black_box("test input")))
    });
}

criterion_group!(benches, bench_example);
criterion_main!(benches);
"#
            .to_string(),
        );

        self.builtin
            .insert("library".to_string(), BuiltinTemplate { manifest, files });
    }

    /// Register web service template
    fn register_web_service_template(&mut self) {
        let mut manifest =
            TemplateManifest::new("web-service".to_string(), TemplateKind::WebService);

        manifest.description = "Web service with axum and tokio".to_string();
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
axum = "0.7"
tokio = { version = "1.40", features = ["full"] }
tower = "0.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tower-test = "0.4"
"#
            .to_string(),
        );

        files.insert(
            "src/main.rs".to_string(),
            r#"//! {{project_name}} web service

use axum::{
    extract::Query,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, Level};

#[derive(Debug, Deserialize)]
struct QueryParams {
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct Response {
    message: String,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Build our application with a route
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/hello", get(hello_handler))
        .route("/api/v1/echo", post(echo_handler));

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await?;
    
    info!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<Response> {
    Json(Response {
        message: "Service is healthy".to_string(),
        status: "ok".to_string(),
    })
}

async fn hello_handler(Query(params): Query<QueryParams>) -> Json<Response> {
    let name = params.name.unwrap_or_else(|| "World".to_string());
    Json(Response {
        message: format!("Hello, {name}!"),
        status: "ok".to_string(),
    })
}

async fn echo_handler(
    Json(payload): Json<HashMap<String, String>>
) -> Json<HashMap<String, String>> {
    Json(payload)
}

"#
            .to_string(),
        );

        files.insert(
            "src/lib.rs".to_string(),
            r#"//! {{project_name}} library

use serde::{Deserialize, Serialize};
use std::fmt;

/// Custom error type
#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    /// Error message
    pub message: String,
    /// Error code
    pub code: u32,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Example function for library usage
pub fn process_data(input: &str) -> Result<String> {
    if input.is_empty() {
        return Err(Error {
            message: "Input cannot be empty".to_string(),
            code: 400,
        });
    }
    
    Ok(format!("Processed: {input}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_data() {
        let result = process_data("test").unwrap();
        assert_eq!(result, "Processed: test");
    }
    
    #[test]
    fn test_empty_input() {
        let result = process_data("");
        assert!(result.is_err());
    }
}
"#
            .to_string(),
        );

        files.insert(
            ".ferrous-forge/config.toml".to_string(),
            r#"# Ferrous Forge configuration for {{project_name}}

[validation]
# Validation settings
enabled = true
max_line_length = 100
max_function_lines = 50
max_file_lines = 300

[safety]
# Safety pipeline settings  
enabled = true
pre_commit = true
pre_push = true

[fix]
# Auto-fix settings
conservative_mode = true
backup_files = true
"#
            .to_string(),
        );

        self.builtin.insert(
            "web-service".to_string(),
            BuiltinTemplate { manifest, files },
        );
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<(&str, TemplateKind, &str)> {
        let mut templates = Vec::new();

        for (name, template) in &self.builtin {
            templates.push((
                name.as_str(),
                template.manifest.kind,
                template.manifest.description.as_str(),
            ));
        }

        templates.sort_by_key(|t| t.0);
        templates
    }

    /// Get a built-in template
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinTemplate> {
        self.builtin.get(name)
    }

    /// Register a custom template
    pub fn register_custom(&mut self, name: String, path: PathBuf) -> Result<()> {
        if self.builtin.contains_key(&name) {
            return Err(Error::validation(format!(
                "Cannot override built-in template: {}",
                name
            )));
        }

        if !path.exists() {
            return Err(Error::validation(format!(
                "Template path does not exist: {}",
                path.display()
            )));
        }

        self.custom.insert(name, path);
        Ok(())
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

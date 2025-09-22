//! Web service template definition

use crate::templates::manifest::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use crate::templates::registry::BuiltinTemplate;
use std::collections::HashMap;
use std::path::PathBuf;

/// Create the web service template
pub fn create_web_service_template() -> BuiltinTemplate {
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

    BuiltinTemplate { manifest, files }
}
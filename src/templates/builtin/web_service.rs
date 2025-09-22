//! Web service template for Ferrous Forge

use super::BuiltinTemplate;
use crate::templates::manifest::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Create the web service template
pub fn create_template() -> Result<BuiltinTemplate> {
    let mut manifest = TemplateManifest::new("web-service".to_string(), TemplateKind::WebService);

    manifest.description = "Web service with axum and tokio".to_string();
    manifest.author = "Ferrous Forge Team".to_string();

    manifest.add_variable(TemplateVariable::required(
        "project_name".to_string(),
        "Name of the project".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "author".to_string(),
        "Author name".to_string(),
        "Unknown".to_string(),
    ));

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

    manifest.add_post_generate("cargo fmt".to_string());
    manifest.add_post_generate("ferrous-forge validate .".to_string());

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
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
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

async fn health() -> Json<Response> {
    Json(Response {
        message: "Service is healthy".to_string(),
        status: "ok".to_string(),
    })
}

async fn hello(Query(params): Query<QueryParams>) -> Json<Response> {
    let name = params.name.unwrap_or_else(|| "World".to_string());
    Json(Response {
        message: format!("Hello, {name}!"),
        status: "ok".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/hello", get(hello));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
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

/// Main error type for this service
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid request data
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = Error::InvalidRequest("test".to_string());
        assert_eq!(err.to_string(), "Invalid request: test");
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

    Ok(BuiltinTemplate { manifest, files })
}

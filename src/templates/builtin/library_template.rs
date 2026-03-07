//! Library template

use crate::templates::{
    BuiltinTemplate, TemplateFile, TemplateKind, TemplateManifest, TemplateVariable,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Create the library template
pub fn create_library_template() -> BuiltinTemplate {
    let manifest = create_library_manifest();
    let files = create_library_files();

    BuiltinTemplate { manifest, files }
}

/// Create the manifest for library template
fn create_library_manifest() -> TemplateManifest {
    let mut manifest = TemplateManifest::new("library".to_string(), TemplateKind::Library);

    manifest.description = "Rust library with comprehensive testing setup".to_string();
    manifest.author = "Ferrous Forge Team".to_string();

    // Add variables
    manifest.add_variable(TemplateVariable::required(
        "project_name".to_string(),
        "Name of the library".to_string(),
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
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/lib.rs"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from(".ferrous-forge/config.toml"),
        PathBuf::from(".ferrous-forge/config.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("rustfmt.toml"),
        PathBuf::from("rustfmt.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("clippy.toml"),
        PathBuf::from("clippy.toml"),
    ));

    // Add post-generate commands
    manifest.add_post_generate("cargo fmt".to_string());
    manifest.add_post_generate("cargo test".to_string());

    manifest
}

/// Create the files for library template
fn create_library_files() -> HashMap<String, String> {
    let mut files = HashMap::new();

    files.insert("Cargo.toml".to_string(), cargo_toml_content());
    files.insert("src/lib.rs".to_string(), lib_rs_content());
    files.insert(
        ".ferrous-forge/config.toml".to_string(),
        config_toml_content(),
    );
    files.insert("rustfmt.toml".to_string(), rustfmt_toml_content());
    files.insert("clippy.toml".to_string(), clippy_toml_content());

    files
}

/// Cargo.toml content
fn cargo_toml_content() -> String {
    r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"
rust-version = "1.85.0"
authors = ["{{author}}"]
description = "A Ferrous Forge compliant Rust library"
license = "MIT OR Apache-2.0"
repository = "https://github.com/{{author}}/{{project_name}}"
keywords = []
categories = []

[dependencies]
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
tempfile = "3.10"
tokio = { version = "1.40", features = ["test-util"] }

[lints.rust]
missing_docs = "warn"
unsafe_code = "forbid"

[lints.rustdoc]
broken_intra_doc_links = "deny"
invalid_html_tags = "deny"
missing_crate_level_docs = "warn"
bare_urls = "warn"
redundant_explicit_links = "warn"
unescaped_backticks = "warn"

[lints.clippy]
missing_safety_doc = "deny"
missing_errors_doc = "warn"
missing_panics_doc = "warn"
empty_docs = "warn"
doc_markdown = "warn"
needless_doctest_main = "warn"
suspicious_doc_comments = "warn"
too_long_first_doc_paragraph = "warn"
unwrap_used = "deny"
expect_used = "deny"
"#
    .to_string()
}

/// src/lib.rs content
fn lib_rs_content() -> String {
    r#"//! `{{project_name}}` — A Ferrous Forge compliant library.
//!
//! ## Overview
//!
//! Describe what this library does and its primary use cases.
//!
//! ## Quick Start
//!
//! ```rust
//! // Example usage here
//! ```
#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use anyhow::Result;
use thiserror::Error;

/// Library-specific errors
#[derive(Debug, Error)]
pub enum {{project_pascal}}Error {
    /// An example error variant
    #[error("Example error: {0}")]
    Example(String),
}

/// Main library functionality
pub struct {{project_pascal}} {
    // Add fields here
}

impl {{project_pascal}} {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            // Initialize fields
        }
    }
}

impl Default for {{project_pascal}} {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let instance = {{project_pascal}}::new();
        // Add assertions
    }
}
"#
    .to_string()
}

/// .ferrous-forge/config.toml content
fn config_toml_content() -> String {
    r#"# Ferrous Forge project configuration
# These values are LOCKED — LLM agents must not modify them without human approval.

[validation]
max_file_lines = 300
max_function_lines = 50
required_edition = "2024"
required_rust_version = "1.85.0"
ban_underscore_bandaid = true
require_documentation = true
"#
    .to_string()
}

/// rustfmt.toml content
fn rustfmt_toml_content() -> String {
    r#"# Ferrous Forge project rustfmt configuration
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
edition = "2024"
"#
    .to_string()
}

/// clippy.toml content
fn clippy_toml_content() -> String {
    r#"# Ferrous Forge project clippy configuration
too-many-lines-threshold = 50
cognitive-complexity-threshold = 25
"#
    .to_string()
}

//! Library template for Ferrous Forge

use super::BuiltinTemplate;
use crate::templates::manifest::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Create the library template
pub fn create_template() -> Result<BuiltinTemplate> {
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

    Ok(BuiltinTemplate { manifest, files })
}

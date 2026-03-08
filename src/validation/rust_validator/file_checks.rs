//! File-level validation checks

mod cargo_validation;
mod doc_validation;
mod pattern_validation;
mod size_validation;
mod test_utils;

use cargo_validation::validate_cargo_toml_content;
use doc_validation::{validate_cargo_doc_config, validate_doc_presence};
use pattern_validation::validate_patterns;
use size_validation::validate_file_size;
use test_utils::is_test_file;

use super::patterns::ValidationPatterns;
use crate::Result;
use crate::validation::Violation;
use std::path::Path;
use tokio::fs;

// Re-export for tests
pub use cargo_validation::validate_cargo_toml;

/// Validates a Rust source file for standards compliance
///
/// # Errors
///
/// Returns an error if the file cannot be read or a validation check fails.
pub async fn validate_rust_file(
    rust_file: &Path,
    violations: &mut Vec<Violation>,
    patterns: &ValidationPatterns,
    max_file_lines: usize,
    max_function_lines: usize,
) -> Result<()> {
    let content = fs::read_to_string(rust_file).await?;
    let lines: Vec<&str> = content.lines().collect();

    let _ = is_test_file(rust_file); // retained for potential future use

    // Validate file size (config-driven)
    validate_file_size(rust_file, &lines, violations, max_file_lines)?;

    // Validate code patterns (function size, underscore bandaid)
    validate_patterns(rust_file, &lines, patterns, violations, max_function_lines)?;

    // Validate documentation presence for module roots
    validate_doc_presence(rust_file, &lines, violations)?;

    Ok(())
}

/// Validates a `Cargo.toml` file: edition/version locks + doc config
///
/// # Errors
///
/// Returns an error if the file cannot be read or doc config validation fails.
pub async fn validate_cargo_toml_full(
    cargo_file: &Path,
    violations: &mut Vec<Violation>,
    required_edition: &str,
    required_rust_version: &str,
) -> Result<()> {
    let content = fs::read_to_string(cargo_file).await?;
    let lines: Vec<&str> = content.lines().collect();

    // Validate edition/version (locked settings)
    validate_cargo_toml_content(
        cargo_file,
        &lines,
        violations,
        required_edition,
        required_rust_version,
    );

    // Validate rustdoc lint config presence
    validate_cargo_doc_config(cargo_file, &content, violations)?;

    Ok(())
}

//! File-level validation checks

mod cargo_validation;
mod pattern_validation;
mod size_validation;
mod test_utils;

// Re-export functions
pub use cargo_validation::validate_cargo_toml;
use pattern_validation::validate_patterns;
use size_validation::validate_file_size;
use test_utils::{check_allow_attributes, is_test_file};

use super::patterns::ValidationPatterns;
use crate::validation::Violation;
use crate::Result;
use std::path::Path;
use tokio::fs;

/// Validates a Rust source file for standards compliance
pub async fn validate_rust_file(
    rust_file: &Path,
    violations: &mut Vec<Violation>,
    patterns: &ValidationPatterns,
) -> Result<()> {
    let content = fs::read_to_string(rust_file).await?;
    let lines: Vec<&str> = content.lines().collect();

    let is_test_file = is_test_file(rust_file);
    let (allow_unwrap, allow_expect) = check_allow_attributes(&lines);

    // Validate file size
    validate_file_size(rust_file, &lines, violations)?;

    // Validate code patterns (functions, unwrap/expect, etc.)
    validate_patterns(
        rust_file,
        &lines,
        patterns,
        violations,
        is_test_file,
        allow_unwrap,
        allow_expect,
    )?;

    Ok(())
}

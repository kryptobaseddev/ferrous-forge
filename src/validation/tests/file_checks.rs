//! Tests for file validation checks

use crate::validation::rust_validator::{
    file_checks::{validate_cargo_toml, validate_rust_file},
    patterns::ValidationPatterns,
};
use crate::validation::violation::*;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_validate_cargo_toml_correct_edition() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");

    let content = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2024"

[dependencies]
"#;

    fs::write(&cargo_toml_path, content)
        .await
        .expect("Failed to write Cargo.toml");

    let mut violations = Vec::new();
    validate_cargo_toml(&cargo_toml_path, &mut violations)
        .await
        .expect("Validation should succeed");

    // Should have no violations for correct edition
    assert!(violations.is_empty());
}

#[tokio::test]
async fn test_validate_cargo_toml_wrong_edition() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");

    let content = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

    fs::write(&cargo_toml_path, content)
        .await
        .expect("Failed to write Cargo.toml");

    let mut violations = Vec::new();
    validate_cargo_toml(&cargo_toml_path, &mut violations)
        .await
        .expect("Validation should succeed");

    // Should have violation for wrong edition
    assert!(!violations.is_empty());
    assert!(violations
        .iter()
        .any(|v| matches!(v.violation_type, ViolationType::WrongEdition)));
}

#[tokio::test]
async fn test_validate_cargo_toml_missing_edition() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");

    let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
"#;

    fs::write(&cargo_toml_path, content)
        .await
        .expect("Failed to write Cargo.toml");

    let mut violations = Vec::new();
    validate_cargo_toml(&cargo_toml_path, &mut violations)
        .await
        .expect("Validation should succeed");

    // Should have violation for missing edition
    assert!(!violations.is_empty());
    assert!(violations
        .iter()
        .any(|v| matches!(v.violation_type, ViolationType::WrongEdition)));
}

#[tokio::test]
async fn test_validate_rust_file_size_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("test.rs");

    // Create a file with many lines (over 300)
    let content: String = (0..350)
        .map(|i| format!("// Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let mut violations = Vec::new();
    let patterns = ValidationPatterns::new().expect("Failed to create patterns");
    validate_rust_file(&rust_file, &mut violations, &patterns)
        .await
        .expect("Validation should succeed");

    // Should have violation for file too large
    assert!(!violations.is_empty());
    assert!(violations
        .iter()
        .any(|v| matches!(v.violation_type, ViolationType::FileTooLarge)));
}

#[tokio::test]
async fn test_validate_rust_file_line_length() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("test.rs");

    let content = "// ".to_string() + &"a".repeat(150); // Create a very long line

    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let mut violations = Vec::new();
    let patterns = ValidationPatterns::new().expect("Failed to create patterns");
    validate_rust_file(&rust_file, &mut violations, &patterns)
        .await
        .expect("Validation should succeed");

    // Should have violation for line too long
    assert!(!violations.is_empty());
    assert!(violations
        .iter()
        .any(|v| matches!(v.violation_type, ViolationType::LineTooLong)));
}

#[tokio::test]
async fn test_validate_rust_file_underscore_bandaid() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("test.rs");

    let content = "fn test_function(_unused: String) {\n    println!(\"test\");\n}\n";
    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let mut violations = Vec::new();
    let patterns = ValidationPatterns::new().expect("Failed to create patterns");
    validate_rust_file(&rust_file, &mut violations, &patterns)
        .await
        .expect("Validation should succeed");

    // Should have violations for underscore bandaid
    assert!(!violations.is_empty());
    assert!(violations
        .iter()
        .any(|v| matches!(v.violation_type, ViolationType::UnderscoreBandaid)));
}

#[tokio::test]
async fn test_validate_rust_file_unwrap_in_production() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("test.rs");

    let content = r#"
#[test]
fn test_code() {
    let value = Some(42);
    value.unwrap(); // This should NOT be flagged in test code
}

fn production_code() {
    let value = Some(42);
    value.unwrap(); // This SHOULD be flagged
    value.expect("error"); // This SHOULD also be flagged
}
"#;

    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let mut violations = Vec::new();
    let patterns = ValidationPatterns::new().expect("Failed to create patterns");
    validate_rust_file(&rust_file, &mut violations, &patterns)
        .await
        .expect("Validation should succeed");

    // Should have violations for unwrap in production code, but not in test code
    let unwrap_violations: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v.violation_type, ViolationType::UnwrapInProduction))
        .collect();

    // Should find unwrap and expect in production function but not in test function
    assert!(!unwrap_violations.is_empty());
    assert!(unwrap_violations.len() >= 2); // unwrap and expect
}

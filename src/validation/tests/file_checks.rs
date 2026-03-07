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
    validate_cargo_toml(&cargo_toml_path, &mut violations, "2024", "1.85.0")
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
edition = "2018"

[dependencies]
"#;

    fs::write(&cargo_toml_path, content)
        .await
        .expect("Failed to write Cargo.toml");

    let mut violations = Vec::new();
    validate_cargo_toml(&cargo_toml_path, &mut violations, "2024", "1.85.0")
        .await
        .expect("Validation should succeed");

    // Should have violation for wrong edition
    assert!(!violations.is_empty());
    assert!(
        violations
            .iter()
            .any(|v| matches!(v.violation_type, ViolationType::WrongEdition))
    );
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
    validate_cargo_toml(&cargo_toml_path, &mut violations, "2024", "1.85.0")
        .await
        .expect("Validation should succeed");

    // Should have violation for missing edition
    assert!(!violations.is_empty());
    assert!(
        violations
            .iter()
            .any(|v| matches!(v.violation_type, ViolationType::WrongEdition))
    );
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
    validate_rust_file(&rust_file, &mut violations, &patterns, 300, 50)
        .await
        .expect("Validation should succeed");

    // Should have violation for file too large
    assert!(!violations.is_empty());
    assert!(
        violations
            .iter()
            .any(|v| matches!(v.violation_type, ViolationType::FileTooLarge))
    );
}

#[tokio::test]
async fn test_validate_rust_file_respects_config_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("test.rs");

    // 150 lines — above default limit of 100 but below 300
    let content: String = (0..150)
        .map(|i| format!("// Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let mut violations_strict = Vec::new();
    let mut violations_lenient = Vec::new();
    let patterns = ValidationPatterns::new().expect("Failed to create patterns");

    // With limit=100 (strict), should flag
    validate_rust_file(&rust_file, &mut violations_strict, &patterns, 100, 50)
        .await
        .expect("Validation should succeed");

    // With limit=300 (lenient), should NOT flag
    validate_rust_file(&rust_file, &mut violations_lenient, &patterns, 300, 50)
        .await
        .expect("Validation should succeed");

    assert!(
        violations_strict
            .iter()
            .any(|v| matches!(v.violation_type, ViolationType::FileTooLarge))
    );
    assert!(
        !violations_lenient
            .iter()
            .any(|v| matches!(v.violation_type, ViolationType::FileTooLarge))
    );
}

#[tokio::test]
async fn test_validate_rust_file_underscore_bandaid() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("test.rs");

    let content = "fn process_data(_unused: String) {\n    println!(\"test\");\n}\n";
    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let mut violations = Vec::new();
    let patterns = ValidationPatterns::new().expect("Failed to create patterns");
    validate_rust_file(&rust_file, &mut violations, &patterns, 300, 50)
        .await
        .expect("Validation should succeed");

    // Should have violations for underscore bandaid
    assert!(!violations.is_empty());
    assert!(
        violations
            .iter()
            .any(|v| matches!(v.violation_type, ViolationType::UnderscoreBandaid))
    );
}

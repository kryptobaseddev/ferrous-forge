//! Tests for the Rust validation module

use super::rust_validator::{
    file_checks::{validate_cargo_toml, validate_rust_file},
    *,
};
use super::violation::*;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

#[test]
fn test_violation_type_variants() {
    let types = [
        ViolationType::UnderscoreBandaid,
        ViolationType::WrongEdition,
        ViolationType::FileTooLarge,
        ViolationType::FunctionTooLarge,
        ViolationType::LineTooLong,
        ViolationType::UnwrapInProduction,
        ViolationType::MissingDocs,
        ViolationType::MissingDependencies,
        ViolationType::OldRustVersion,
    ];

    // Test that variants are distinct
    for (i, type1) in types.iter().enumerate() {
        for (j, type2) in types.iter().enumerate() {
            if i != j {
                assert_ne!(type1, type2);
            }
        }
    }
}

#[test]
fn test_severity_variants() {
    let error = Severity::Error;
    let warning = Severity::Warning;

    // Just test that we can create instances
    assert!(matches!(error, Severity::Error));
    assert!(matches!(warning, Severity::Warning));
}

#[test]
fn test_violation_creation() {
    let violation = Violation {
        violation_type: ViolationType::UnderscoreBandaid,
        file: PathBuf::from("test.rs"),
        line: 10,
        message: "Test violation".to_string(),
        severity: Severity::Error,
    };

    assert_eq!(violation.violation_type, ViolationType::UnderscoreBandaid);
    assert_eq!(violation.file, PathBuf::from("test.rs"));
    assert_eq!(violation.line, 10);
    assert_eq!(violation.message, "Test violation");
    matches!(violation.severity, Severity::Error);
}

#[test]
fn test_clippy_result() {
    let result = ClippyResult {
        success: true,
        output: "All checks passed".to_string(),
    };

    assert!(result.success);
    assert_eq!(result.output, "All checks passed");
}

#[tokio::test]
async fn test_rust_validator_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let validator = RustValidator::new(temp_dir.path().to_path_buf());

    assert!(validator.is_ok());
}

#[tokio::test]
async fn test_generate_report_no_violations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let violations = vec![];
    let report = validator.generate_report(&violations);

    assert!(report.contains("✅"));
    assert!(report.contains("All Rust validation checks passed"));
}

#[tokio::test]
async fn test_generate_report_with_violations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let violations = vec![
        Violation {
            violation_type: ViolationType::UnderscoreBandaid,
            file: PathBuf::from("test.rs"),
            line: 10,
            message: "Underscore parameter".to_string(),
            severity: Severity::Error,
        },
        Violation {
            violation_type: ViolationType::WrongEdition,
            file: PathBuf::from("Cargo.toml"),
            line: 5,
            message: "Wrong edition".to_string(),
            severity: Severity::Error,
        },
    ];

    let report = validator.generate_report(&violations);

    assert!(report.contains("❌"));
    assert!(report.contains("Found 2 violations"));
    assert!(report.contains("UNDERSCOREBANDAID"));
    assert!(report.contains("WRONGEDITION"));
    assert!(report.contains("test.rs:11"));
    assert!(report.contains("Cargo.toml:6"));
}

#[tokio::test]
async fn test_validate_cargo_toml_correct_edition() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"
[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#,
    )
    .await
    .expect("Failed to write Cargo.toml");

    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let mut violations = Vec::new();
    validate_cargo_toml(&cargo_toml, &mut violations)
        .await
        .expect("Should validate");

    assert!(violations.is_empty());
}

#[tokio::test]
async fn test_validate_cargo_toml_wrong_edition() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
    )
    .await
    .expect("Failed to write Cargo.toml");

    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let mut violations = Vec::new();
    validate_cargo_toml(&cargo_toml, &mut violations)
        .await
        .expect("Should validate");

    assert_eq!(violations.len(), 0); // 2021 is now valid
}

#[tokio::test]
async fn test_validate_cargo_toml_missing_edition() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"
[package]
name = "test"
version = "0.1.0"
"#,
    )
    .await
    .expect("Failed to write Cargo.toml");

    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let mut violations = Vec::new();
    validate_cargo_toml(&cargo_toml, &mut violations)
        .await
        .expect("Should validate");

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].violation_type, ViolationType::WrongEdition);
    assert!(violations[0].message.contains("Missing edition"));
}

#[tokio::test]
async fn test_validate_rust_file_size_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rust_file = temp_dir.path().join("test.rs");

    // Create a file with over 300 lines
    let content = (0..350)
        .map(|i| format!("// Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let mut violations = Vec::new();
    validate_rust_file(&rust_file, &mut violations, validator.patterns())
        .await
        .expect("Should validate");

    let file_size_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.violation_type == ViolationType::FileTooLarge)
        .collect();

    assert_eq!(file_size_violations.len(), 1);
    assert!(file_size_violations[0].message.contains("350 lines"));
}

#[tokio::test]
async fn test_validate_rust_file_line_length() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rust_file = temp_dir.path().join("test.rs");

    let long_line = "// ".to_string() + &"x".repeat(150);
    fs::write(&rust_file, long_line)
        .await
        .expect("Failed to write Rust file");

    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let mut violations = Vec::new();
    validate_rust_file(&rust_file, &mut violations, validator.patterns())
        .await
        .expect("Should validate");

    let line_length_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.violation_type == ViolationType::LineTooLong)
        .collect();

    assert_eq!(line_length_violations.len(), 1);
    assert!(line_length_violations[0].message.contains("153 characters"));
}

#[tokio::test]
async fn test_validate_rust_file_underscore_bandaid() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rust_file = temp_dir.path().join("test.rs");

    let content = r"
fn test_function(_param: String) {
    let _ = some_operation();
}
";
    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let mut violations = Vec::new();
    validate_rust_file(&rust_file, &mut violations, validator.patterns())
        .await
        .expect("Should validate");

    let bandaid_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.violation_type == ViolationType::UnderscoreBandaid)
        .collect();

    assert_eq!(bandaid_violations.len(), 2); // One for param, one for let
    assert!(bandaid_violations
        .iter()
        .any(|v| v.message.contains("parameter")));
    assert!(bandaid_violations
        .iter()
        .any(|v| v.message.contains("assignment")));
}

#[tokio::test]
async fn test_validate_rust_file_unwrap_in_production() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rust_file = temp_dir.path().join("test.rs");

    let content = r#"
fn production_code() {
    let value = some_result.unwrap();
    let other = another_result.expect("message");
}

#[test]
fn test_code() {
    let value = some_result.unwrap(); // This should be allowed
}
"#;
    fs::write(&rust_file, content)
        .await
        .expect("Failed to write Rust file");

    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

    let mut violations = Vec::new();
    validate_rust_file(&rust_file, &mut violations, validator.patterns())
        .await
        .expect("Should validate");

    let unwrap_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.violation_type == ViolationType::UnwrapInProduction)
        .collect();

    // Should find 2 violations in production code, but none in test code
    assert_eq!(unwrap_violations.len(), 2);
    assert!(unwrap_violations
        .iter()
        .any(|v| v.message.contains("unwrap")));
    assert!(unwrap_violations
        .iter()
        .any(|v| v.message.contains("expect")));
}

#[test]
fn test_serialization() {
    let violation = Violation {
        violation_type: ViolationType::UnderscoreBandaid,
        file: PathBuf::from("test.rs"),
        line: 10,
        message: "Test violation".to_string(),
        severity: Severity::Error,
    };

    let serialized = serde_json::to_string(&violation).expect("Should serialize");
    let deserialized: Violation = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(violation.violation_type, deserialized.violation_type);
    assert_eq!(violation.file, deserialized.file);
    assert_eq!(violation.line, deserialized.line);
    assert_eq!(violation.message, deserialized.message);
}

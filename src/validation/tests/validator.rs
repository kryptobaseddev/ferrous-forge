//! Tests for the RustValidator

use crate::validation::rust_validator::*;
use crate::validation::violation::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_rust_validator_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let validator = RustValidator::new(temp_dir.path().to_path_buf());

    assert!(validator.is_ok());
}

#[tokio::test]
async fn test_generate_report_no_violations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Failed to create validator");

    let violations = vec![];
    let report = validator.generate_report(&violations);

    assert!(report.contains("✅ No violations found"));
}

#[tokio::test]
async fn test_generate_report_with_violations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let validator =
        RustValidator::new(temp_dir.path().to_path_buf()).expect("Failed to create validator");

    let violations = vec![
        Violation {
            violation_type: ViolationType::UnderscoreBandaid,
            file: PathBuf::from("test.rs"),
            line: 10,
            message: "Test violation".to_string(),
            severity: Severity::Error,
        },
        Violation {
            violation_type: ViolationType::LineTooLong,
            file: PathBuf::from("other.rs"),
            line: 20,
            message: "Line too long".to_string(),
            severity: Severity::Warning,
        },
    ];

    let report = validator.generate_report(&violations);

    // Check that report contains violation information
    assert!(report.contains("2 violations"));
    assert!(report.contains("UNDERSCOREBANDAID"));
    assert!(report.contains("LINETOOLONG"));
    assert!(report.contains("test.rs"));
    assert!(report.contains("other.rs"));
}

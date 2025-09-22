//! Tests for validation types

use crate::validation::violation::*;

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
    let severities = [Severity::Error, Severity::Warning, Severity::Info];

    for severity in severities {
        // Test that Debug formatting works
        let _debug_str = format!("{:?}", severity);
    }
}

#[test]
fn test_violation_creation() {
    let violation = Violation {
        violation_type: ViolationType::UnderscoreBandaid,
        file: std::path::PathBuf::from("test.rs"),
        line: 42,
        column: Some(10),
        message: "Test violation".to_string(),
        severity: Severity::Error,
    };

    assert_eq!(violation.violation_type, ViolationType::UnderscoreBandaid);
    assert_eq!(violation.file, std::path::PathBuf::from("test.rs"));
    assert_eq!(violation.line, 42);
    assert_eq!(violation.column, Some(10));
    assert_eq!(violation.message, "Test violation");
    assert_eq!(violation.severity, Severity::Error);
}

#[test]
fn test_clippy_result() {
    let result = super::super::ClippyResult {
        success: false,
        output: "Some clippy warnings".to_string(),
    };

    assert!(!result.success);
    assert_eq!(result.output, "Some clippy warnings");
}

#[test]
fn test_serialization() {
    let violation = Violation {
        violation_type: ViolationType::UnderscoreBandaid,
        file: std::path::PathBuf::from("test.rs"),
        line: 42,
        column: Some(10),
        message: "Test violation".to_string(),
        severity: Severity::Error,
    };

    // Test that we can serialize and deserialize
    let json = serde_json::to_string(&violation).expect("Serialization should work");
    let deserialized: Violation = serde_json::from_str(&json).expect("Deserialization should work");

    assert_eq!(violation.violation_type, deserialized.violation_type);
    assert_eq!(violation.file, deserialized.file);
    assert_eq!(violation.line, deserialized.line);
}

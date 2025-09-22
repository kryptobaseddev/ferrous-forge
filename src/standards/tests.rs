//! Tests for coding standards

use super::*;
use tempfile::TempDir;
use tokio::fs;

#[test]
fn test_coding_standards_default() {
    let standards = CodingStandards::default();

    // Test edition standards
    assert_eq!(standards.edition.required_edition, "2024");
    assert_eq!(standards.edition.min_rust_version, "1.82.0");
    assert!(!standards.edition.auto_upgrade);

    // Test file limits
    assert_eq!(standards.file_limits.max_lines, 400);
    assert_eq!(standards.file_limits.max_line_length, 100);

    // Test function limits
    assert_eq!(standards.function_limits.max_lines, 230);
    assert_eq!(standards.function_limits.max_complexity, 10);

    // Test documentation standards
    assert!(standards.documentation.require_public_docs);
    assert!(!standards.documentation.require_private_docs);
    assert_eq!(standards.documentation.min_coverage, 80.0);

    // Test banned patterns
    assert!(standards.banned_patterns.ban_unwrap);
    assert!(standards.banned_patterns.ban_expect);
    assert!(standards.banned_patterns.ban_panic);
    assert!(standards.banned_patterns.ban_underscore_bandaid);
}

#[test]
fn test_get_clippy_rules() {
    let standards = CodingStandards::default();
    let rules = standards.get_clippy_rules();

    assert!(rules.contains(&"-D warnings".to_string()));
    assert!(rules.contains(&"-D clippy::unwrap_used".to_string()));
    assert!(rules.contains(&"-D clippy::expect_used".to_string()));
    assert!(rules.contains(&"-D clippy::panic".to_string()));
    assert!(rules.contains(&"-D missing_docs".to_string()));
    assert!(rules.contains(&"-F unsafe_code".to_string()));
}

#[tokio::test]
async fn test_check_compliance() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#,
    )
    .await
    .expect("Failed to write Cargo.toml");

    let standards = CodingStandards::default();
    let violations = standards
        .check_compliance(temp_dir.path())
        .await
        .expect("Check should succeed");

    assert!(violations.is_empty());
}

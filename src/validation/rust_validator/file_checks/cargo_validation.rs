//! Cargo.toml validation functions

use crate::Result;
use crate::validation::{Severity, Violation, ViolationType};
use std::path::Path;
use tokio::fs;

/// Validates a Cargo.toml file for standards compliance (reads file itself).
/// Used by tests and legacy callers.
pub async fn validate_cargo_toml(
    cargo_file: &Path,
    violations: &mut Vec<Violation>,
    required_edition: &str,
    required_rust_version: &str,
) -> Result<()> {
    let content = fs::read_to_string(cargo_file).await?;
    let lines: Vec<&str> = content.lines().collect();
    validate_cargo_toml_content(cargo_file, &lines, violations, required_edition, required_rust_version);
    Ok(())
}

/// Validates Cargo.toml content that has already been read into lines.
/// Used by file_checks.rs to avoid double file reads.
pub fn validate_cargo_toml_content(
    cargo_file: &Path,
    lines: &[&str],
    violations: &mut Vec<Violation>,
    required_edition: &str,
    required_rust_version: &str,
) {
    validate_edition(lines, cargo_file, violations, required_edition);
    validate_rust_version_field(lines, cargo_file, violations, required_rust_version);
}

/// Check edition field against locked required value
fn validate_edition(
    lines: &[&str],
    cargo_file: &Path,
    violations: &mut Vec<Violation>,
    required_edition: &str,
) {
    let mut edition_found = false;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("edition") && line.contains('=') {
            edition_found = true;
            let found_edition = extract_quoted_value(line);

            if found_edition.as_deref() != Some(required_edition) {
                let found_display = found_edition.as_deref().unwrap_or("<unknown>");
                violations.push(Violation {
                    violation_type: ViolationType::WrongEdition,
                    file: cargo_file.to_path_buf(),
                    line: i + 1,
                    message: format!(
                        "FERROUS FORGE [LOCKED SETTING] — Edition Violation\n\
                         ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                         \n\
                           Found:    edition = \"{found_display}\"\n\
                           Required: edition = \"{required_edition}\"\n\
                           Lock:     .ferrous-forge/config.toml → required_edition = \"{required_edition}\"\n\
                         \n\
                           ⚠ AI AGENT NOTICE: This edition is locked by project configuration.\n\
                           DO NOT change required_edition without human approval.\n\
                           DO NOT downgrade Cargo.toml to match a lower edition.\n\
                         \n\
                           To fix: Update edition = \"{required_edition}\" in Cargo.toml, then run:\n\
                             cargo fix --edition"
                    ),
                    severity: Severity::Error,
                });
            }
            break;
        }
    }

    if !edition_found {
        violations.push(Violation {
            violation_type: ViolationType::WrongEdition,
            file: cargo_file.to_path_buf(),
            line: 0,
            message: format!(
                "FERROUS FORGE [LOCKED SETTING] — Missing Edition\n\
                 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                 \n\
                   Missing edition specification in Cargo.toml.\n\
                   Required: edition = \"{required_edition}\"\n\
                 \n\
                   ⚠ AI AGENT NOTICE: Add edition = \"{required_edition}\" to [package] section."
            ),
            severity: Severity::Error,
        });
    }
}

/// Check rust-version field against locked required value
fn validate_rust_version_field(
    lines: &[&str],
    cargo_file: &Path,
    violations: &mut Vec<Violation>,
    required_rust_version: &str,
) {
    if required_rust_version.is_empty() {
        return;
    }

    for (i, line) in lines.iter().enumerate() {
        if line.contains("rust-version") && line.contains('=') {
            let found_version = extract_quoted_value(line);

            if found_version.as_deref() != Some(required_rust_version) {
                let found_display = found_version.as_deref().unwrap_or("<unknown>");
                violations.push(Violation {
                    violation_type: ViolationType::OldRustVersion,
                    file: cargo_file.to_path_buf(),
                    line: i + 1,
                    message: format!(
                        "FERROUS FORGE [LOCKED SETTING] — Rust Version Violation\n\
                         ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                         \n\
                           Found:    rust-version = \"{found_display}\"\n\
                           Required: rust-version = \"{required_rust_version}\"\n\
                           Lock:     .ferrous-forge/config.toml → required_rust_version = \"{required_rust_version}\"\n\
                         \n\
                           ⚠ AI AGENT NOTICE: This rust-version is locked by project configuration.\n\
                           DO NOT change required_rust_version without human approval.\n\
                           DO NOT downgrade rust-version to resolve compilation errors."
                    ),
                    severity: Severity::Error,
                });
            }
            return;
        }
    }
    // rust-version absent from Cargo.toml is acceptable
}

/// Extract a quoted string value from a TOML line like: key = "value"
fn extract_quoted_value(line: &str) -> Option<String> {
    let after_eq = line.split('=').nth(1)?.trim();
    let inner = after_eq.trim_matches('"').trim_matches('\'').trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_quoted_value() {
        assert_eq!(
            extract_quoted_value("edition = \"2024\""),
            Some("2024".to_string())
        );
        assert_eq!(
            extract_quoted_value("rust-version = \"1.85.0\""),
            Some("1.85.0".to_string())
        );
    }
}

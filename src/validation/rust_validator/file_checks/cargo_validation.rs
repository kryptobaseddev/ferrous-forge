//! Cargo.toml validation functions

use crate::validation::{Severity, Violation, ViolationType};
use crate::Result;
use std::path::Path;
use tokio::fs;

/// Validates a Cargo.toml file for standards compliance
pub async fn validate_cargo_toml(cargo_file: &Path, violations: &mut Vec<Violation>) -> Result<()> {
    let content = fs::read_to_string(cargo_file).await?;
    let lines: Vec<&str> = content.lines().collect();

    // Check for Edition 2021 or 2024
    let mut edition_found = false;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("edition") {
            if !line.contains("2021") && !line.contains("2024") {
                violations.push(Violation {
                    violation_type: ViolationType::WrongEdition,
                    file: cargo_file.to_path_buf(),
                    line: i + 1,
                    message: "Must use Edition 2021 or 2024".to_string(),
                    severity: Severity::Error,
                });
            }
            edition_found = true;
            break;
        }
    }

    if !edition_found {
        violations.push(Violation {
            violation_type: ViolationType::WrongEdition,
            file: cargo_file.to_path_buf(),
            line: 0,
            message: "Missing edition specification - must be '2021' or '2024'".to_string(),
            severity: Severity::Error,
        });
    }

    Ok(())
}
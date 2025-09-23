//! Ferrous Forge standards checking

use crate::Result;
use std::path::Path;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// Standards check implementation
pub struct StandardsCheck;

impl SafetyCheck for StandardsCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "standards"
    }

    fn description() -> &'static str {
        "Validates Ferrous Forge coding standards"
    }
}

/// Run Ferrous Forge standards validation
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Standards);

    // Use the existing validation module
    let validator = crate::validation::RustValidator::new(project_path.to_path_buf())?;
    match validator.validate_project().await {
        Ok(violations) => {
            result.set_duration(start.elapsed());

            if violations.is_empty() {
                result.add_context("All Ferrous Forge standards met");
            } else {
                result.add_error(format!("Found {} standards violations", violations.len()));

                // Add specific violations (limit to first 5)
                for violation in violations.iter().take(5) {
                    result.add_error(format!(
                        "{:?}: {}",
                        violation.violation_type, violation.message
                    ));
                }

                if violations.len() > 5 {
                    result.add_error(format!("... and {} more violations", violations.len() - 5));
                }

                result.add_suggestion("Run 'ferrous-forge validate' for detailed report");
                result.add_suggestion("Fix standards violations before proceeding");
            }
        }
        Err(e) => {
            result.set_duration(start.elapsed());
            result.add_error(format!("Standards check failed: {}", e));
            result.add_suggestion("Ensure project has valid Cargo.toml and Rust files");
        }
    }

    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_standards_check_struct() {
        assert_eq!(StandardsCheck::name(), "standards");
        assert!(!StandardsCheck::description().is_empty());
    }
}

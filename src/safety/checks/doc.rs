//! Documentation checking - placeholder for now, using standards module

use crate::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{CheckType, report::CheckResult};

/// Doc check implementation
pub struct DocCheck;

impl SafetyCheck for DocCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "doc"
    }

    fn description() -> &'static str {
        "Builds project documentation"
    }
}

/// Run documentation build check
///
/// # Errors
///
/// Returns an error if the check result cannot be constructed.
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Doc);

    match Command::new("cargo")
        .args(["doc", "--no-deps"])
        .current_dir(project_path)
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                result.add_error(&format!("cargo doc failed: {stderr}"));
                result.fail();
            } else {
                result.add_context("Documentation built successfully");
            }
        }
        Err(e) => {
            result.add_error(&format!("Failed to execute cargo doc: {e}"));
            result.fail();
        }
    }

    result.set_duration(start.elapsed());
    Ok(result)
}

/// Check documentation coverage
///
/// # Errors
///
/// Returns an error if the documentation coverage check fails to run.
pub async fn coverage_check(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::DocCoverage);

    let coverage = crate::doc_coverage::check_documentation_coverage(project_path).await?;
    let threshold = 50.0;

    if !coverage.meets_threshold(threshold) {
        result.add_error(&format!(
            "Documentation coverage {:.1}% is below the {:.0}% threshold",
            coverage.coverage_percent, threshold
        ));
        result.fail();
    }

    result.add_context(&format!(
        "Documentation coverage: {:.1}% ({}/{} items documented)",
        coverage.coverage_percent, coverage.documented_items, coverage.total_items
    ));

    result.set_duration(start.elapsed());
    Ok(result)
}

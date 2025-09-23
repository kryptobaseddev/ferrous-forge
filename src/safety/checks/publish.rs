//! Publish validation checking

use crate::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// Publish dry run check implementation
pub struct PublishCheck;

impl SafetyCheck for PublishCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "publish"
    }

    fn description() -> &'static str {
        "Validates crates.io publication readiness"
    }
}

/// Run cargo publish --dry-run
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::PublishDryRun);

    // Run cargo publish --dry-run
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["publish", "--dry-run"])
        .output()?;

    result.set_duration(start.elapsed());

    if !output.status.success() {
        result.add_error("Publish dry run failed");
        result.add_suggestion("Fix publish issues before attempting real publish");

        // Parse publish errors
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut error_count = 0;

        for line in stderr.lines() {
            if line.starts_with("error:") && error_count < 3 {
                result.add_error(format!("Publish: {}", line.trim()));
                error_count += 1;
            } else if line.contains("warning:") && line.contains("ignoring") {
                result.add_context(line.trim().to_string());
            }
        }

        if error_count >= 3 {
            result.add_error("... and more publish errors (showing first 3)");
        }

        result.add_suggestion("Run 'cargo publish --dry-run' to see detailed output");
        result.add_suggestion("Check Cargo.toml metadata and file inclusions");
    } else {
        result.add_context("Ready for crates.io publication");

        // Check for warnings in successful dry run
        let stderr = String::from_utf8_lossy(&output.stderr);
        let warning_count = stderr
            .lines()
            .filter(|line| line.starts_with("warning:"))
            .count();

        if warning_count > 0 {
            result.add_context(format!(
                "Publish dry run completed with {} warnings",
                warning_count
            ));
        }
    }

    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_check_struct() {
        assert_eq!(PublishCheck::name(), "publish");
        assert!(!PublishCheck::description().is_empty());
    }
}

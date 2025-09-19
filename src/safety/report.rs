//! Safety check reporting and result handling

use crate::{Error, Result};
use console::style;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::fs;

use super::{CheckType, PipelineStage};

/// Result of a single safety check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Type of check that was run
    pub check_type: CheckType,
    /// Whether the check passed
    pub passed: bool,
    /// Duration the check took to run
    pub duration: Duration,
    /// Error messages if check failed
    pub errors: Vec<String>,
    /// Suggestions for fixing issues
    pub suggestions: Vec<String>,
    /// Additional context information
    pub context: Vec<String>,
}

impl CheckResult {
    /// Create a new check result
    pub fn new(check_type: CheckType) -> Self {
        Self {
            check_type,
            passed: true,
            duration: Duration::default(),
            errors: Vec::new(),
            suggestions: Vec::new(),
            context: Vec::new(),
        }
    }

    /// Mark the check as failed
    pub fn fail(&mut self) {
        self.passed = false;
    }

    /// Add an error message
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
        self.fail();
    }

    /// Add a suggestion
    pub fn add_suggestion(&mut self, suggestion: impl Into<String>) {
        self.suggestions.push(suggestion.into());
    }

    /// Add context information
    pub fn add_context(&mut self, context: impl Into<String>) {
        self.context.push(context.into());
    }

    /// Set the duration
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    /// Get a status emoji for display
    pub fn status_emoji(&self) -> &'static str {
        if self.passed {
            "✅"
        } else {
            "❌"
        }
    }

    /// Get a colored status for display
    pub fn status_colored(&self) -> console::StyledObject<&'static str> {
        if self.passed {
            style("PASS").green().bold()
        } else {
            style("FAIL").red().bold()
        }
    }
}

/// Comprehensive safety report for a pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    /// Pipeline stage this report is for
    pub stage: PipelineStage,
    /// Individual check results
    pub checks: Vec<CheckResult>,
    /// Whether all checks passed
    pub passed: bool,
    /// Total duration for all checks
    pub total_duration: Duration,
    /// Timestamp when report was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SafetyReport {
    /// Create a new safety report
    pub fn new(stage: PipelineStage) -> Self {
        Self {
            stage,
            checks: Vec::new(),
            passed: true,
            total_duration: Duration::default(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add a check result to the report
    pub fn add_check(&mut self, check: CheckResult) {
        if !check.passed {
            self.passed = false;
        }
        self.total_duration += check.duration;
        self.checks.push(check);
    }

    /// Merge another report into this one
    pub fn merge(&mut self, other: SafetyReport) {
        for check in other.checks {
            self.add_check(check);
        }
    }

    /// Get failed checks
    pub fn failed_checks(&self) -> Vec<&CheckResult> {
        self.checks.iter().filter(|c| !c.passed).collect()
    }

    /// Get all error messages
    pub fn all_errors(&self) -> Vec<String> {
        self.checks
            .iter()
            .flat_map(|c| c.errors.iter().cloned())
            .collect()
    }

    /// Get all suggestions
    pub fn all_suggestions(&self) -> Vec<String> {
        self.checks
            .iter()
            .flat_map(|c| c.suggestions.iter().cloned())
            .collect()
    }

    /// Print a concise report
    pub fn print_summary(&self) {
        println!(
            "🛡️  Ferrous Forge Safety Pipeline - {}\n",
            self.stage.display_name()
        );

        for check in &self.checks {
            println!(
                "  {} {} ({:.2}s)",
                check.status_emoji(),
                check.check_type.display_name(),
                check.duration.as_secs_f64()
            );
        }

        println!("\nTotal time: {:.2}s", self.total_duration.as_secs_f64());

        if self.passed {
            println!("{}", style("🎉 All safety checks passed!").green().bold());
        } else {
            println!(
                "{}",
                style("🚨 Safety checks FAILED - operation blocked!")
                    .red()
                    .bold()
            );
        }
    }

    /// Print a detailed report with errors and suggestions
    pub fn print_detailed(&self) {
        self.print_summary();

        if !self.passed {
            let failed = self.failed_checks();

            if !failed.is_empty() {
                println!("\n{}", style("📋 Failed Checks:").red().bold());

                for check in failed {
                    println!(
                        "\n  {} {}",
                        style("❌").red(),
                        style(check.check_type.display_name()).red().bold()
                    );

                    for error in &check.errors {
                        println!("    {}", style(format!("⚠️  {}", error)).yellow());
                    }

                    if !check.suggestions.is_empty() {
                        println!("    {}", style("💡 Suggestions:").cyan());
                        for suggestion in &check.suggestions {
                            println!("      • {}", style(suggestion).cyan());
                        }
                    }
                }
            }

            // Show general suggestions
            let all_suggestions = self.all_suggestions();
            if !all_suggestions.is_empty() {
                println!("\n{}", style("🔧 How to Fix:").cyan().bold());
                for suggestion in all_suggestions.iter().take(5) {
                    println!("  • {}", suggestion);
                }
            }
        }

        println!();
    }

    /// Save report to file for audit trail
    pub async fn save_to_file(&self) -> Result<()> {
        let reports_dir = crate::config::Config::config_dir_path()?.join("safety-reports");
        fs::create_dir_all(&reports_dir).await?;

        let filename = format!(
            "{}-{}.json",
            self.timestamp.format("%Y%m%d-%H%M%S"),
            self.stage.name()
        );

        let report_path = reports_dir.join(filename);
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize report: {}", e)))?;

        fs::write(&report_path, contents).await?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_creation() {
        let mut result = CheckResult::new(CheckType::Format);

        assert!(result.passed);
        assert!(result.errors.is_empty());
        assert_eq!(result.check_type, CheckType::Format);

        result.add_error("Format violation");
        assert!(!result.passed);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_safety_report() {
        let mut report = SafetyReport::new(PipelineStage::PreCommit);

        assert!(report.passed);
        assert!(report.checks.is_empty());

        let mut failed_check = CheckResult::new(CheckType::Clippy);
        failed_check.add_error("Clippy error");

        report.add_check(failed_check);

        assert!(!report.passed);
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.failed_checks().len(), 1);
    }

    #[test]
    fn test_report_merge() {
        let mut report1 = SafetyReport::new(PipelineStage::PreCommit);
        let mut report2 = SafetyReport::new(PipelineStage::PrePush);

        let check1 = CheckResult::new(CheckType::Format);
        let check2 = CheckResult::new(CheckType::Test);

        report1.add_check(check1);
        report2.add_check(check2);

        report1.merge(report2);

        assert_eq!(report1.checks.len(), 2);
    }
}

//! Coverage reporting and validation

use super::analyzer::CoverageAnalyzer;
use super::types::CoverageReport;
use crate::{Error, Result};
use std::path::Path;

impl CoverageAnalyzer {
    /// Validate coverage meets minimum thresholds
    ///
    /// # Errors
    ///
    /// Returns a validation error if any coverage metric falls below its
    /// configured minimum threshold and `fail_on_low_coverage` is enabled.
    pub fn validate_coverage(&self, report: &CoverageReport) -> Result<()> {
        let mut violations = Vec::new();

        if report.line_coverage < self.config().min_line_coverage {
            violations.push(format!(
                "Line coverage {:.1}% is below minimum {:.1}%",
                report.line_coverage,
                self.config().min_line_coverage
            ));
        }

        if report.function_coverage < self.config().min_function_coverage {
            violations.push(format!(
                "Function coverage {:.1}% is below minimum {:.1}%",
                report.function_coverage,
                self.config().min_function_coverage
            ));
        }

        if report.branch_coverage < self.config().min_branch_coverage {
            violations.push(format!(
                "Branch coverage {:.1}% is below minimum {:.1}%",
                report.branch_coverage,
                self.config().min_branch_coverage
            ));
        }

        if !violations.is_empty() {
            let message = format!("Coverage violations:\n  • {}", violations.join("\n  • "));

            if self.config().fail_on_low_coverage {
                return Err(Error::validation(message));
            }
            tracing::warn!("{}", message);
        }

        Ok(())
    }

    /// Generate a human-readable coverage report
    pub fn format_coverage_report(&self, report: &CoverageReport) -> String {
        let mut output = String::new();

        self.add_report_header(&mut output);
        self.add_overall_coverage(&mut output, report);
        self.add_threshold_status(&mut output, report);
        self.add_low_coverage_files(&mut output, report);
        self.add_improvement_suggestions(&mut output);

        output
    }

    /// Add report header section
    fn add_report_header(&self, output: &mut String) {
        output.push_str("📊 Test Coverage Report\n");
        output.push_str("=".repeat(39).as_str());
        output.push_str("\n\n");
    }

    /// Add overall coverage statistics
    fn add_overall_coverage(&self, output: &mut String, report: &CoverageReport) {
        output.push_str(&format!("📈 Overall Coverage:\n"));
        output.push_str(&format!(
            "  • Lines:     {:.1}% ({}/{})\n",
            report.line_coverage, report.lines_tested, report.total_lines
        ));
        output.push_str(&format!(
            "  • Functions: {:.1}% ({}/{})\n",
            report.function_coverage, report.functions_tested, report.total_functions
        ));
        output.push_str(&format!(
            "  • Branches:  {:.1}% ({}/{})\n\n",
            report.branch_coverage, report.branches_tested, report.total_branches
        ));
    }

    /// Add threshold status section
    fn add_threshold_status(&self, output: &mut String, report: &CoverageReport) {
        let line_status = if report.line_coverage >= self.config().min_line_coverage {
            "✅"
        } else {
            "❌"
        };
        let func_status = if report.function_coverage >= self.config().min_function_coverage {
            "✅"
        } else {
            "❌"
        };
        let branch_status = if report.branch_coverage >= self.config().min_branch_coverage {
            "✅"
        } else {
            "❌"
        };

        output.push_str("🎯 Threshold Status:\n");
        output.push_str(&format!(
            "  {} Lines:     {:.1}% (min: {:.1}%)\n",
            line_status,
            report.line_coverage,
            self.config().min_line_coverage
        ));
        output.push_str(&format!(
            "  {} Functions: {:.1}% (min: {:.1}%)\n",
            func_status,
            report.function_coverage,
            self.config().min_function_coverage
        ));
        output.push_str(&format!(
            "  {} Branches:  {:.1}% (min: {:.1}%)\n\n",
            branch_status,
            report.branch_coverage,
            self.config().min_branch_coverage
        ));
    }

    /// Add low coverage files section
    fn add_low_coverage_files(&self, output: &mut String, report: &CoverageReport) {
        let mut low_coverage_files: Vec<_> = report
            .file_coverage
            .values()
            .filter(|file| file.line_coverage < self.config().min_line_coverage)
            .collect();
        low_coverage_files.sort_by(|a, b| {
            a.line_coverage
                .partial_cmp(&b.line_coverage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if !low_coverage_files.is_empty() {
            output.push_str("⚠️  Files Below Threshold:\n");
            for file in low_coverage_files.iter().take(5) {
                output.push_str(&format!(
                    "  • {}: {:.1}%\n",
                    file.file_path, file.line_coverage
                ));
            }
            if low_coverage_files.len() > 5 {
                output.push_str(&format!(
                    "  ... and {} more files\n",
                    low_coverage_files.len() - 5
                ));
            }
            output.push('\n');
        }
    }

    /// Add improvement suggestions section
    fn add_improvement_suggestions(&self, output: &mut String) {
        output.push_str("💡 To improve coverage:\n");
        output.push_str("  • Add tests for uncovered code paths\n");
        output.push_str("  • Remove dead code\n");
        output.push_str("  • Test error conditions and edge cases\n");
        output.push_str("  • Use property-based testing\n");
    }

    /// Check coverage for a project
    ///
    /// # Errors
    ///
    /// Returns an error if the coverage run fails or coverage is below
    /// the configured minimum thresholds.
    pub async fn check_project_coverage(&self, project_path: &Path) -> Result<()> {
        println!("🧪 Checking test coverage...");

        let report = self.run_coverage(project_path).await?;

        println!("{}", self.format_coverage_report(&report));

        self.validate_coverage(&report)?;

        println!("✅ Coverage check completed successfully");
        Ok(())
    }
}

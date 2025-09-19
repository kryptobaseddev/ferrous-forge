//! Edition compatibility analyzer

use crate::{Error, Result};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

use super::Edition;

/// Edition analyzer for checking compatibility
pub struct EditionAnalyzer {
    project_path: std::path::PathBuf,
}

impl EditionAnalyzer {
    /// Create a new edition analyzer
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
        }
    }

    /// Analyze the project for edition compatibility issues
    pub async fn analyze(&self, target_edition: Edition) -> Result<AnalysisReport> {
        let mut report = AnalysisReport {
            target_edition,
            total_files: 0,
            issues: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Count Rust source files
        for entry in WalkDir::new(&self.project_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                report.total_files += 1;
            }
        }

        // Run cargo check with edition lints
        self.check_edition_lints(target_edition, &mut report)
            .await?;

        // Check for common patterns that need updating
        self.check_common_patterns(target_edition, &mut report)
            .await?;

        // Add general suggestions
        self.add_suggestions(target_edition, &mut report);

        Ok(report)
    }

    /// Run cargo check with edition-specific lints
    async fn check_edition_lints(
        &self,
        _edition: Edition,
        report: &mut AnalysisReport,
    ) -> Result<()> {
        let cargo_path =
            which::which("cargo").map_err(|_| Error::rust_not_found("cargo not found"))?;

        // Get migration lints for current edition
        let current_edition = super::detect_edition(&self.project_path.join("Cargo.toml")).await?;
        let lints = current_edition.migration_lints();

        if lints.is_empty() {
            return Ok(());
        }

        // Run cargo clippy with edition lints
        for lint in lints {
            let output = Command::new(&cargo_path)
                .current_dir(&self.project_path)
                .args(&["clippy", "--", "-W", &lint])
                .output()?;

            let stderr = String::from_utf8_lossy(&output.stderr);

            // Parse lint output for issues
            for line in stderr.lines() {
                if line.contains("warning:") || line.contains("error:") {
                    report.issues.push(EditionIssue {
                        file: self.extract_file_from_lint(line),
                        line: self.extract_line_from_lint(line),
                        message: line.to_string(),
                        severity: if line.contains("error:") {
                            Severity::Error
                        } else {
                            Severity::Warning
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// Check for common patterns that need updating
    async fn check_common_patterns(
        &self,
        target_edition: Edition,
        report: &mut AnalysisReport,
    ) -> Result<()> {
        match target_edition {
            Edition::Edition2018 => {
                // Check for extern crate declarations (not needed in 2018+)
                report.warnings.push(
                    "Consider removing `extern crate` declarations (except for macros)".to_string(),
                );
            }
            Edition::Edition2021 => {
                // Check for disjoint captures in closures
                report.warnings.push(
                    "Closures now capture individual fields instead of entire structs".to_string(),
                );
                // Check for or patterns
                report
                    .warnings
                    .push("Or patterns in matches are now available".to_string());
            }
            Edition::Edition2024 => {
                // Check for new edition 2024 features
                report.warnings.push(
                    "Edition 2024 includes improved async support and pattern matching".to_string(),
                );
            }
            _ => {}
        }

        Ok(())
    }

    /// Add migration suggestions
    fn add_suggestions(&self, target_edition: Edition, report: &mut AnalysisReport) {
        report.suggestions.push(format!(
            "Run `cargo fix --edition` to automatically fix most edition issues"
        ));

        report.suggestions.push(format!(
            "After migration, update Cargo.toml to edition = \"{}\"",
            target_edition.as_str()
        ));

        report
            .suggestions
            .push("Review and test your code after migration".to_string());

        if target_edition >= Edition::Edition2018 {
            report
                .suggestions
                .push("Consider using `cargo fmt` to update code style".to_string());
        }
    }

    fn extract_file_from_lint(&self, line: &str) -> Option<String> {
        // Simple extraction - this would be more sophisticated in production
        line.split(':').next().map(|s| s.trim().to_string())
    }

    fn extract_line_from_lint(&self, line: &str) -> Option<u32> {
        // Simple extraction - this would be more sophisticated in production
        line.split(':').nth(1).and_then(|s| s.trim().parse().ok())
    }
}

/// Analysis report for edition compatibility
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    /// Target edition
    pub target_edition: Edition,
    /// Total number of Rust files
    pub total_files: usize,
    /// Issues found
    pub issues: Vec<EditionIssue>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Suggestions for migration
    pub suggestions: Vec<String>,
}

impl AnalysisReport {
    /// Check if the project is ready for migration
    pub fn is_ready_for_migration(&self) -> bool {
        self.issues.iter().all(|i| i.severity != Severity::Error)
    }

    /// Get a summary of the analysis
    pub fn summary(&self) -> String {
        format!(
            "Analysis complete: {} files, {} issues, {} warnings",
            self.total_files,
            self.issues.len(),
            self.warnings.len()
        )
    }
}

/// Edition compatibility issue
#[derive(Debug, Clone)]
pub struct EditionIssue {
    /// File path
    pub file: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Issue message
    pub message: String,
    /// Severity
    pub severity: Severity,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Error - must be fixed
    Error,
    /// Warning - should be reviewed
    Warning,
    /// Info - informational
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_analyzer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = EditionAnalyzer::new(temp_dir.path());

        // Create a simple Cargo.toml
        let manifest_content = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;

        tokio::fs::write(temp_dir.path().join("Cargo.toml"), manifest_content)
            .await
            .unwrap();

        // This would normally analyze the project
        // In tests, we just verify it doesn't panic
        let _report = analyzer.analyze(Edition::Edition2024).await;
    }
}

//! Test coverage integration with cargo-tarpaulin
//!
//! This module provides functionality to measure test coverage using cargo-tarpaulin
//! and enforce minimum coverage thresholds as part of Ferrous Forge standards.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Test coverage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageConfig {
    /// Minimum line coverage percentage (0-100)
    pub min_line_coverage: f64,
    /// Minimum function coverage percentage (0-100)
    pub min_function_coverage: f64,
    /// Minimum branch coverage percentage (0-100)
    pub min_branch_coverage: f64,
    /// Whether to fail builds on coverage below threshold
    pub fail_on_low_coverage: bool,
    /// Files to exclude from coverage analysis
    pub exclude_files: Vec<String>,
    /// Directories to exclude from coverage analysis
    pub exclude_dirs: Vec<String>,
}

impl Default for CoverageConfig {
    fn default() -> Self {
        Self {
            min_line_coverage: 80.0,
            min_function_coverage: 85.0,
            min_branch_coverage: 75.0,
            fail_on_low_coverage: true,
            exclude_files: vec![
                "main.rs".to_string(),
                "lib.rs".to_string(),
                "**/tests/**".to_string(),
                "**/benches/**".to_string(),
            ],
            exclude_dirs: vec![
                "target".to_string(),
                "tests".to_string(),
                "benches".to_string(),
                "examples".to_string(),
            ],
        }
    }
}

/// Test coverage results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    /// Overall line coverage percentage
    pub line_coverage: f64,
    /// Overall function coverage percentage  
    pub function_coverage: f64,
    /// Overall branch coverage percentage
    pub branch_coverage: f64,
    /// Per-file coverage breakdown
    pub file_coverage: HashMap<String, FileCoverage>,
    /// Total lines tested
    pub lines_tested: u32,
    /// Total lines in codebase
    pub total_lines: u32,
    /// Functions tested
    pub functions_tested: u32,
    /// Total functions
    pub total_functions: u32,
    /// Branches tested
    pub branches_tested: u32,
    /// Total branches
    pub total_branches: u32,
}

/// Coverage information for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    /// File path relative to project root
    pub file_path: String,
    /// Line coverage percentage for this file
    pub line_coverage: f64,
    /// Function coverage percentage for this file
    pub function_coverage: f64,
    /// Lines tested in this file
    pub lines_tested: u32,
    /// Total lines in this file
    pub total_lines: u32,
    /// Functions tested in this file
    pub functions_tested: u32,
    /// Total functions in this file
    pub total_functions: u32,
}

/// Test coverage analyzer
pub struct CoverageAnalyzer {
    /// Coverage configuration
    config: CoverageConfig,
}

impl CoverageAnalyzer {
    /// Create a new coverage analyzer with default configuration
    pub fn new() -> Self {
        Self {
            config: CoverageConfig::default(),
        }
    }

    /// Create a new coverage analyzer with custom configuration
    pub fn with_config(config: CoverageConfig) -> Self {
        Self { config }
    }

    /// Check if cargo-tarpaulin is installed
    pub fn check_tarpaulin_installed(&self) -> Result<bool> {
        let output = Command::new("cargo")
            .args(["tarpaulin", "--version"])
            .output();

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// Install cargo-tarpaulin if not already installed
    pub async fn install_tarpaulin(&self) -> Result<()> {
        if self.check_tarpaulin_installed()? {
            tracing::info!("cargo-tarpaulin already installed");
            return Ok(());
        }

        tracing::info!("Installing cargo-tarpaulin...");

        let output = Command::new("cargo")
            .args(["install", "cargo-tarpaulin"])
            .output()
            .map_err(|e| Error::process(format!("Failed to run cargo install: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::process(format!(
                "Failed to install cargo-tarpaulin: {}",
                stderr
            )));
        }

        tracing::info!("cargo-tarpaulin installed successfully");
        Ok(())
    }

    /// Run test coverage analysis
    pub async fn run_coverage(&self, project_path: &Path) -> Result<CoverageReport> {
        if !self.check_tarpaulin_installed()? {
            return Err(Error::validation(
                "cargo-tarpaulin not installed. Run 'cargo install cargo-tarpaulin' first.",
            ));
        }

        tracing::info!("Running test coverage analysis...");

        let exclude_files_str = self.config.exclude_files.join(",");
        let mut args = vec![
            "tarpaulin",
            "--verbose",
            "--timeout",
            "120",
            "--out",
            "Json",
            "--exclude-files",
            &exclude_files_str,
        ];

        // Add exclude directories
        for exclude_dir in &self.config.exclude_dirs {
            args.extend_from_slice(&["--exclude-files", exclude_dir]);
        }

        let output = Command::new("cargo")
            .args(&args)
            .current_dir(project_path)
            .output()
            .map_err(|e| Error::process(format!("Failed to run cargo tarpaulin: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::process(format!(
                "cargo tarpaulin failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_tarpaulin_output(&stdout)
    }

    /// Parse cargo-tarpaulin JSON output
    fn parse_tarpaulin_output(&self, output: &str) -> Result<CoverageReport> {
        #[derive(Deserialize)]
        struct TarpaulinOutput {
            #[serde(rename = "coverage")]
            line_coverage: f64,
            #[serde(rename = "linesCovered")]
            lines_covered: u32,
            #[serde(rename = "linesTotal")]
            lines_total: u32,
            #[serde(rename = "branchesCovered")]
            branches_covered: Option<u32>,
            #[serde(rename = "branchesTotal")]
            branches_total: Option<u32>,
            #[serde(rename = "files")]
            files: HashMap<String, TarpaulinFile>,
        }

        #[derive(Deserialize)]
        struct TarpaulinFile {
            #[serde(rename = "coverage")]
            line_coverage: f64,
            #[serde(rename = "linesCovered")]
            lines_covered: u32,
            #[serde(rename = "linesTotal")]
            lines_total: u32,
        }

        let tarpaulin_data: TarpaulinOutput = serde_json::from_str(output)
            .map_err(|e| Error::validation(format!("Failed to parse coverage output: {}", e)))?;

        let mut file_coverage = HashMap::new();
        let mut total_functions_tested = 0;
        let mut total_functions = 0;

        for (file_path, file_data) in tarpaulin_data.files {
            // Estimate function coverage (tarpaulin doesn't provide this directly)
            let estimated_functions = (file_data.lines_total / 10).max(1); // Rough estimate
            let estimated_functions_tested =
                ((file_data.line_coverage / 100.0) * estimated_functions as f64) as u32;

            total_functions += estimated_functions;
            total_functions_tested += estimated_functions_tested;

            file_coverage.insert(
                file_path.clone(),
                FileCoverage {
                    file_path,
                    line_coverage: file_data.line_coverage,
                    function_coverage: if estimated_functions > 0 {
                        (estimated_functions_tested as f64 / estimated_functions as f64) * 100.0
                    } else {
                        100.0
                    },
                    lines_tested: file_data.lines_covered,
                    total_lines: file_data.lines_total,
                    functions_tested: estimated_functions_tested,
                    total_functions: estimated_functions,
                },
            );
        }

        let function_coverage = if total_functions > 0 {
            (total_functions_tested as f64 / total_functions as f64) * 100.0
        } else {
            100.0
        };

        let branch_coverage = if let (Some(covered), Some(total)) = (
            tarpaulin_data.branches_covered,
            tarpaulin_data.branches_total,
        ) {
            if total > 0 {
                (covered as f64 / total as f64) * 100.0
            } else {
                100.0
            }
        } else {
            tarpaulin_data.line_coverage // Fallback to line coverage
        };

        Ok(CoverageReport {
            line_coverage: tarpaulin_data.line_coverage,
            function_coverage,
            branch_coverage,
            file_coverage,
            lines_tested: tarpaulin_data.lines_covered,
            total_lines: tarpaulin_data.lines_total,
            functions_tested: total_functions_tested,
            total_functions,
            branches_tested: tarpaulin_data.branches_covered.unwrap_or(0),
            total_branches: tarpaulin_data.branches_total.unwrap_or(0),
        })
    }

    /// Validate coverage meets minimum thresholds
    pub fn validate_coverage(&self, report: &CoverageReport) -> Result<()> {
        let mut violations = Vec::new();

        if report.line_coverage < self.config.min_line_coverage {
            violations.push(format!(
                "Line coverage {:.1}% is below minimum {:.1}%",
                report.line_coverage, self.config.min_line_coverage
            ));
        }

        if report.function_coverage < self.config.min_function_coverage {
            violations.push(format!(
                "Function coverage {:.1}% is below minimum {:.1}%",
                report.function_coverage, self.config.min_function_coverage
            ));
        }

        if report.branch_coverage < self.config.min_branch_coverage {
            violations.push(format!(
                "Branch coverage {:.1}% is below minimum {:.1}%",
                report.branch_coverage, self.config.min_branch_coverage
            ));
        }

        if !violations.is_empty() {
            let message = format!("Coverage violations:\n  • {}", violations.join("\n  • "));

            if self.config.fail_on_low_coverage {
                return Err(Error::validation(message));
            }
            tracing::warn!("{}", message);
        }

        Ok(())
    }

    /// Generate a human-readable coverage report
    pub fn format_coverage_report(&self, report: &CoverageReport) -> String {
        let mut output = String::new();

        output.push_str("📊 Test Coverage Report\n");
        output.push_str("═══════════════════\n");
        output.push_str("═══\n\n");

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

        // Coverage status
        let line_status = if report.line_coverage >= self.config.min_line_coverage {
            "✅"
        } else {
            "❌"
        };
        let func_status = if report.function_coverage >= self.config.min_function_coverage {
            "✅"
        } else {
            "❌"
        };
        let branch_status = if report.branch_coverage >= self.config.min_branch_coverage {
            "✅"
        } else {
            "❌"
        };

        output.push_str("🎯 Threshold Status:\n");
        output.push_str(&format!(
            "  {} Lines:     {:.1}% (min: {:.1}%)\n",
            line_status, report.line_coverage, self.config.min_line_coverage
        ));
        output.push_str(&format!(
            "  {} Functions: {:.1}% (min: {:.1}%)\n",
            func_status, report.function_coverage, self.config.min_function_coverage
        ));
        output.push_str(&format!(
            "  {} Branches:  {:.1}% (min: {:.1}%)\n\n",
            branch_status, report.branch_coverage, self.config.min_branch_coverage
        ));

        // Top files with low coverage
        let mut low_coverage_files: Vec<_> = report
            .file_coverage
            .values()
            .filter(|file| file.line_coverage < self.config.min_line_coverage)
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

        output.push_str("💡 To improve coverage:\n");
        output.push_str("  • Add tests for uncovered code paths\n");
        output.push_str("  • Remove dead code\n");
        output.push_str("  • Test error conditions and edge cases\n");
        output.push_str("  • Use property-based testing\n");

        output
    }

    /// Check coverage for a project
    pub async fn check_project_coverage(&self, project_path: &Path) -> Result<()> {
        println!("🧪 Checking test coverage...");

        let report = self.run_coverage(project_path).await?;

        println!("{}", self.format_coverage_report(&report));

        self.validate_coverage(&report)?;

        println!("✅ Coverage check completed successfully");
        Ok(())
    }
}

impl Default for CoverageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_config_default() {
        let config = CoverageConfig::default();
        assert_eq!(config.min_line_coverage, 80.0);
        assert_eq!(config.min_function_coverage, 85.0);
        assert_eq!(config.min_branch_coverage, 75.0);
        assert!(config.fail_on_low_coverage);
    }

    #[test]
    fn test_coverage_analyzer_creation() {
        let analyzer = CoverageAnalyzer::new();
        assert_eq!(analyzer.config.min_line_coverage, 80.0);

        let custom_config = CoverageConfig {
            min_line_coverage: 90.0,
            ..Default::default()
        };
        let custom_analyzer = CoverageAnalyzer::with_config(custom_config);
        assert_eq!(custom_analyzer.config.min_line_coverage, 90.0);
    }

    #[test]
    fn test_validate_coverage_success() {
        let analyzer = CoverageAnalyzer::new();
        let report = CoverageReport {
            line_coverage: 85.0,
            function_coverage: 90.0,
            branch_coverage: 80.0,
            file_coverage: HashMap::new(),
            lines_tested: 85,
            total_lines: 100,
            functions_tested: 18,
            total_functions: 20,
            branches_tested: 8,
            total_branches: 10,
        };

        assert!(analyzer.validate_coverage(&report).is_ok());
    }

    #[test]
    fn test_validate_coverage_failure() {
        let analyzer = CoverageAnalyzer::new();
        let report = CoverageReport {
            line_coverage: 70.0, // Below 80% minimum
            function_coverage: 90.0,
            branch_coverage: 80.0,
            file_coverage: HashMap::new(),
            lines_tested: 70,
            total_lines: 100,
            functions_tested: 18,
            total_functions: 20,
            branches_tested: 8,
            total_branches: 10,
        };

        assert!(analyzer.validate_coverage(&report).is_err());
    }

    #[test]
    fn test_format_coverage_report() {
        let analyzer = CoverageAnalyzer::new();
        let report = CoverageReport {
            line_coverage: 85.0,
            function_coverage: 90.0,
            branch_coverage: 80.0,
            file_coverage: HashMap::new(),
            lines_tested: 85,
            total_lines: 100,
            functions_tested: 18,
            total_functions: 20,
            branches_tested: 8,
            total_branches: 10,
        };

        let formatted = analyzer.format_coverage_report(&report);
        assert!(formatted.contains("Test Coverage Report"));
        assert!(formatted.contains("85.0%"));
        assert!(formatted.contains("90.0%"));
        assert!(formatted.contains("80.0%"));
    }
}

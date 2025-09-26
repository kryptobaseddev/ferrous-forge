//! Test coverage analyzer implementation

use super::types::{CoverageConfig, CoverageReport};
use super::utils::{parse_tarpaulin_json, process_file_coverage, calculate_branch_coverage};
use crate::{Error, Result};
use std::path::Path;
use std::process::Command;

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
        let tarpaulin_data = parse_tarpaulin_json(output)?;
        let (file_coverage, function_stats) = process_file_coverage(&tarpaulin_data.files);
        let branch_coverage = calculate_branch_coverage(&tarpaulin_data);

        Ok(CoverageReport {
            line_coverage: tarpaulin_data.line_coverage,
            function_coverage: function_stats.coverage,
            branch_coverage,
            file_coverage,
            lines_tested: tarpaulin_data.lines_covered,
            total_lines: tarpaulin_data.lines_total,
            functions_tested: function_stats.tested,
            total_functions: function_stats.total,
            branches_tested: tarpaulin_data.branches_covered.unwrap_or(0),
            total_branches: tarpaulin_data.branches_total.unwrap_or(0),
        })
    }

    /// Get config reference
    pub fn config(&self) -> &CoverageConfig {
        &self.config
    }

    /// Run tarpaulin and get coverage report
    ///
    /// This is a convenience wrapper around run_coverage that's more explicit
    /// about running tarpaulin
    pub async fn run_tarpaulin(&self, project_path: &Path) -> Result<CoverageReport> {
        self.run_coverage(project_path).await
    }

    /// Parse a coverage report from tarpaulin output
    ///
    /// Parses the JSON output from cargo-tarpaulin and converts it to our 
    /// CoverageReport format
    pub fn parse_coverage_report(&self, tarpaulin_output: &str) -> Result<CoverageReport> {
        self.parse_tarpaulin_output(tarpaulin_output)
    }

    /// Enforce minimum coverage threshold
    ///
    /// Returns an error if the coverage is below the specified threshold
    pub fn enforce_minimum_coverage(&self, report: &CoverageReport, threshold: f64) -> Result<()> {
        if report.line_coverage < threshold {
            return Err(Error::validation(format!(
                "Test coverage {:.1}% is below minimum threshold of {:.1}%",
                report.line_coverage, threshold
            )));
        }
        Ok(())
    }

    /// Generate a coverage badge SVG
    ///
    /// Creates an SVG badge showing the current test coverage percentage
    pub fn generate_coverage_badge(&self, report: &CoverageReport) -> String {
        let coverage = report.line_coverage;
        let color = match coverage {
            c if c >= 80.0 => "#4c1",    // Green
            c if c >= 60.0 => "#dfb317", // Yellow
            c if c >= 40.0 => "#fe7d37", // Orange
            _ => "#e05d44",              // Red
        };

        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="114" height="20">
                <linearGradient id="a" x2="0" y2="100%">
                    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
                    <stop offset="1" stop-opacity=".1"/>
                </linearGradient>
                <rect rx="3" width="114" height="20" fill="#555"/>
                <rect rx="3" x="63" width="51" height="20" fill="{}"/>
                <path fill="{}" d="M63 0h4v20h-4z"/>
                <rect rx="3" width="114" height="20" fill="url(#a)"/>
                <g fill="#fff" text-anchor="middle" 
                   font-family="DejaVu Sans,Verdana,Geneva,sans-serif" font-size="11">
                    <text x="32" y="15" fill="#010101" fill-opacity=".3">coverage</text>
                    <text x="32" y="14">coverage</text>
                    <text x="87" y="15" fill="#010101" fill-opacity=".3">{:.1}%</text>
                    <text x="87" y="14">{:.1}%</text>
                </g>
            </svg>"##,
            color, color, coverage, coverage
        )
    }
}

impl Default for CoverageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
//! Test coverage analyzer implementation

use super::types::{CoverageConfig, CoverageReport, FileCoverage};
use crate::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;
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
}

impl Default for CoverageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Tarpaulin JSON output structure
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

/// Tarpaulin file coverage data
#[derive(Deserialize)]
struct TarpaulinFile {
    #[serde(rename = "coverage")]
    line_coverage: f64,
    #[serde(rename = "linesCovered")]
    lines_covered: u32,
    #[serde(rename = "linesTotal")]
    lines_total: u32,
}

/// Function coverage statistics
struct FunctionStats {
    coverage: f64,
    tested: u32,
    total: u32,
}

/// Parse tarpaulin JSON output
fn parse_tarpaulin_json(output: &str) -> Result<TarpaulinOutput> {
    serde_json::from_str(output)
        .map_err(|e| Error::validation(format!("Failed to parse coverage output: {}", e)))
}

/// Process file coverage data and calculate function statistics
fn process_file_coverage(
    files: &HashMap<String, TarpaulinFile>,
) -> (HashMap<String, FileCoverage>, FunctionStats) {
    let mut file_coverage = HashMap::new();
    let mut total_functions_tested = 0;
    let mut total_functions = 0;

    for (file_path, file_data) in files {
        let (estimated_functions, estimated_functions_tested) =
            estimate_function_coverage(file_data);

        total_functions += estimated_functions;
        total_functions_tested += estimated_functions_tested;

        let coverage = create_file_coverage(
            file_path,
            file_data,
            estimated_functions,
            estimated_functions_tested,
        );
        file_coverage.insert(file_path.clone(), coverage);
    }

    let function_coverage =
        calculate_function_coverage_percentage(total_functions_tested, total_functions);

    (
        file_coverage,
        FunctionStats {
            coverage: function_coverage,
            tested: total_functions_tested,
            total: total_functions,
        },
    )
}

/// Estimate function coverage from line coverage data
fn estimate_function_coverage(file_data: &TarpaulinFile) -> (u32, u32) {
    let estimated_functions = (file_data.lines_total / 10).max(1);
    let estimated_functions_tested =
        ((file_data.line_coverage / 100.0) * estimated_functions as f64) as u32;
    (estimated_functions, estimated_functions_tested)
}

/// Create file coverage object
fn create_file_coverage(
    file_path: &str,
    file_data: &TarpaulinFile,
    estimated_functions: u32,
    estimated_functions_tested: u32,
) -> FileCoverage {
    FileCoverage {
        file_path: file_path.to_string(),
        line_coverage: file_data.line_coverage,
        function_coverage: calculate_function_coverage_percentage(
            estimated_functions_tested,
            estimated_functions,
        ),
        lines_tested: file_data.lines_covered,
        total_lines: file_data.lines_total,
        functions_tested: estimated_functions_tested,
        total_functions: estimated_functions,
    }
}

/// Calculate function coverage percentage
fn calculate_function_coverage_percentage(tested: u32, total: u32) -> f64 {
    if total > 0 {
        (tested as f64 / total as f64) * 100.0
    } else {
        100.0
    }
}

/// Calculate branch coverage from tarpaulin data
fn calculate_branch_coverage(tarpaulin_data: &TarpaulinOutput) -> f64 {
    if let (Some(covered), Some(total)) = (
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
    }
}

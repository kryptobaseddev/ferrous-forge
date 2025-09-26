//! Test coverage utility functions

use super::types::FileCoverage;
use crate::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;

/// Tarpaulin JSON output structure
#[derive(Deserialize)]
pub struct TarpaulinOutput {
    /// Line coverage percentage
    #[serde(rename = "coverage")]
    pub line_coverage: f64,
    /// Number of lines covered
    #[serde(rename = "linesCovered")]
    pub lines_covered: u32,
    /// Total number of lines
    #[serde(rename = "linesTotal")]
    pub lines_total: u32,
    /// Number of branches covered
    #[serde(rename = "branchesCovered")]
    pub branches_covered: Option<u32>,
    /// Total number of branches
    #[serde(rename = "branchesTotal")]
    pub branches_total: Option<u32>,
    /// Per-file coverage data
    #[serde(rename = "files")]
    pub files: HashMap<String, TarpaulinFile>,
}

/// Tarpaulin file coverage data
#[derive(Deserialize)]
pub struct TarpaulinFile {
    /// Line coverage percentage
    #[serde(rename = "coverage")]
    pub line_coverage: f64,
    /// Number of lines covered
    #[serde(rename = "linesCovered")]
    pub lines_covered: u32,
    /// Total number of lines
    #[serde(rename = "linesTotal")]
    pub lines_total: u32,
}

/// Function coverage statistics
pub struct FunctionStats {
    /// Function coverage percentage
    pub coverage: f64,
    /// Number of functions tested
    pub tested: u32,
    /// Total number of functions
    pub total: u32,
}

/// Parse tarpaulin JSON output
pub fn parse_tarpaulin_json(output: &str) -> Result<TarpaulinOutput> {
    serde_json::from_str(output).map_err(|e| {
        Error::process(format!("Failed to parse tarpaulin output: {}", e))
    })
}

/// Process file coverage data
pub fn process_file_coverage(
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
pub fn calculate_function_coverage_percentage(tested: u32, total: u32) -> f64 {
    if total == 0 {
        0.0
    } else {
        (tested as f64 / total as f64) * 100.0
    }
}

/// Calculate branch coverage
pub fn calculate_branch_coverage(data: &TarpaulinOutput) -> f64 {
    match (data.branches_covered, data.branches_total) {
        (Some(covered), Some(total)) if total > 0 => (covered as f64 / total as f64) * 100.0,
        _ => 0.0,
    }
}
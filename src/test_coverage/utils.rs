//! Test coverage utility functions

use super::types::FileCoverage;
use crate::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;

/// Tarpaulin JSON output structure (tarpaulin >= 0.18 format).
///
/// tarpaulin --out Json produces:
/// ```json
/// { "files": [ { "path": [...], "covered": N, "coverable": N, ... } ],
///   "coverage": 82.6, "covered": 2121, "coverable": 2567 }
/// ```
#[derive(Deserialize)]
pub struct TarpaulinOutput {
    /// Line coverage percentage (0–100)
    pub coverage: f64,
    /// Number of lines covered across all files
    pub covered: u32,
    /// Total coverable lines across all files
    pub coverable: u32,
    /// Per-file coverage entries
    pub files: Vec<TarpaulinFile>,
    /// Unused — branches not reported in this format
    #[serde(skip)]
    pub branches_covered: Option<u32>,
    /// Unused — branches not reported in this format
    #[serde(skip)]
    pub branches_total: Option<u32>,
    /// Convenience alias used by the rest of the codebase
    #[serde(skip)]
    pub lines_covered: u32,
    /// Convenience alias used by the rest of the codebase
    #[serde(skip)]
    pub lines_total: u32,
    /// Convenience alias used by the rest of the codebase
    #[serde(skip)]
    pub line_coverage: f64,
}

/// Tarpaulin per-file coverage data (tarpaulin >= 0.18 format).
#[derive(Deserialize)]
pub struct TarpaulinFile {
    /// File path split into components (e.g. `["/", "src", "lib.rs"]`)
    pub path: Vec<String>,
    /// Number of covered lines in this file
    pub covered: u32,
    /// Total coverable lines in this file
    pub coverable: u32,
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
///
/// Supports the tarpaulin >= 0.18 array-of-files format:
/// `{ "files": [...], "coverage": f64, "covered": u32, "coverable": u32 }`
///
/// # Errors
///
/// Returns an error if the output string is not valid JSON or does not
/// match the expected tarpaulin format.
pub fn parse_tarpaulin_json(output: &str) -> Result<TarpaulinOutput> {
    let mut data: TarpaulinOutput = serde_json::from_str(output)
        .map_err(|e| Error::process(format!("Failed to parse tarpaulin output: {}", e)))?;

    // Populate convenience aliases so callers that reference the old field names still work.
    data.line_coverage = data.coverage;
    data.lines_covered = data.covered;
    data.lines_total = data.coverable;

    Ok(data)
}

/// Process file coverage data
pub fn process_file_coverage(
    files: &[TarpaulinFile],
) -> (HashMap<String, FileCoverage>, FunctionStats) {
    let mut file_coverage = HashMap::new();
    let mut total_functions_tested = 0;
    let mut total_functions = 0;

    for file_data in files {
        // Reconstruct the file path from its component array, skipping the leading "/".
        let file_path = file_data
            .path
            .join("/")
            .trim_start_matches("//")
            .to_string();
        let (estimated_functions, estimated_functions_tested) =
            estimate_function_coverage(file_data);

        total_functions += estimated_functions;
        total_functions_tested += estimated_functions_tested;

        let coverage = create_file_coverage(
            &file_path,
            file_data,
            estimated_functions,
            estimated_functions_tested,
        );
        file_coverage.insert(file_path, coverage);
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
    let estimated_functions = (file_data.coverable / 10).max(1);
    let line_coverage_pct = if file_data.coverable == 0 {
        0.0
    } else {
        file_data.covered as f64 / file_data.coverable as f64
    };
    let estimated_functions_tested = (line_coverage_pct * estimated_functions as f64) as u32;
    (estimated_functions, estimated_functions_tested)
}

/// Create file coverage object
fn create_file_coverage(
    file_path: &str,
    file_data: &TarpaulinFile,
    estimated_functions: u32,
    estimated_functions_tested: u32,
) -> FileCoverage {
    let line_coverage = if file_data.coverable == 0 {
        0.0
    } else {
        (file_data.covered as f64 / file_data.coverable as f64) * 100.0
    };
    FileCoverage {
        file_path: file_path.to_string(),
        line_coverage,
        function_coverage: calculate_function_coverage_percentage(
            estimated_functions_tested,
            estimated_functions,
        ),
        lines_tested: file_data.covered,
        total_lines: file_data.coverable,
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

//! Test coverage types and configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

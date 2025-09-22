//! Type definitions for the fix command

use std::collections::HashSet;

/// Context information about a file being fixed
#[derive(Debug)]
pub struct FileContext {
    /// Whether this is a test file
    pub is_test_file: bool,
    /// Whether this is a binary file
    pub is_bin_file: bool,
    /// Whether this is an example file
    pub is_example_file: bool,
    /// Function signatures in the file
    pub function_signatures: Vec<FunctionSignature>,
}

/// Represents a function signature found in the file
#[derive(Debug)]
pub struct FunctionSignature {
    /// Name of the function
    pub name: String,
    /// Starting line number
    pub line_start: usize,
    /// Ending line number
    pub line_end: usize,
    /// Whether function returns Result
    pub returns_result: bool,
    /// Whether function returns Option
    pub returns_option: bool,
}

/// Result of attempting to fix a violation
#[derive(Debug)]
pub enum FixResult {
    /// Fix was successfully applied with description
    Fixed(String),
    /// Fix was skipped with reason
    Skipped(String),
    /// Fix is not applicable to this violation
    NotApplicable,
}

/// Configuration for fixing violations
pub struct FixConfig {
    /// Only fix these violation types
    pub only_types: Option<HashSet<String>>,
    /// Skip these violation types
    pub skip_types: Option<HashSet<String>>,
    /// Run in dry-run mode
    pub dry_run: bool,
    /// Limit number of fixes
    pub limit: Option<usize>,
}

/// Configuration for filter options
#[derive(Debug)]
pub struct FilterOptions {
    pub only_types: Option<HashSet<String>>,
    pub skip_types: Option<HashSet<String>>,
}

/// Statistics for fix operations
#[derive(Debug)]
pub struct FixStats {
    pub total_fixed: usize,
    pub total_skipped: usize,
    pub files_modified: usize,
}

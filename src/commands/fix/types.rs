//! Type definitions for the fix command

use std::collections::HashSet;

/// Context information about a file being fixed
#[derive(Debug)]
pub struct FileContext {
    pub is_test_file: bool,
    pub is_bin_file: bool,
    pub is_example_file: bool,
    pub function_signatures: Vec<FunctionSignature>,
}

/// Represents a function signature found in the file
#[derive(Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub returns_result: bool,
    pub returns_option: bool,
}

/// Result of attempting to fix a violation
#[derive(Debug)]
pub enum FixResult {
    Fixed(String),
    Skipped(String),
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
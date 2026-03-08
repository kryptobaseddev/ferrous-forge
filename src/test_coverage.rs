//! Test coverage module - now modularized for better organization

/// Coverage analysis engine and metrics computation.
pub mod analyzer;
/// Coverage report generation and output formatting.
pub mod reporting;
/// Unit tests for the test coverage module.
pub mod tests;
/// Data types for coverage configuration and reports.
pub mod types;
/// Shared utility functions for coverage analysis.
pub mod utils;

pub use analyzer::CoverageAnalyzer;
pub use types::{CoverageConfig, CoverageReport, FileCoverage};

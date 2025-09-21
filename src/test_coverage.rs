//! Test coverage module - now modularized for better organization

pub mod analyzer;
pub mod reporting;
pub mod tests;
pub mod types;

pub use analyzer::CoverageAnalyzer;
pub use types::{CoverageConfig, CoverageReport, FileCoverage};

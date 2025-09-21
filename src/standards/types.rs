//! Type definitions for Rust coding standards

use serde::{Deserialize, Serialize};

/// Rust coding standards enforced by Ferrous Forge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingStandards {
    /// Rust edition requirements
    pub edition: EditionStandards,
    /// File size limits
    pub file_limits: FileLimits,
    /// Function size limits
    pub function_limits: FunctionLimits,
    /// Documentation requirements
    pub documentation: DocumentationStandards,
    /// Banned patterns and practices
    pub banned_patterns: BannedPatterns,
    /// Dependency requirements
    pub dependencies: DependencyStandards,
    /// Security requirements
    pub security: SecurityStandards,
}

/// Rust edition requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditionStandards {
    /// Required Rust edition
    pub required_edition: String,
    /// Minimum Rust version
    pub min_rust_version: String,
    /// Whether to automatically upgrade projects
    pub auto_upgrade: bool,
}

/// File size limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLimits {
    /// Maximum lines per file
    pub max_lines: usize,
    /// Maximum characters per line
    pub max_line_length: usize,
    /// Files that are exempt from size limits
    pub exempt_files: Vec<String>,
}

/// Function size limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionLimits {
    /// Maximum lines per function
    pub max_lines: usize,
    /// Maximum cyclomatic complexity
    pub max_complexity: usize,
    /// Functions that are exempt from size limits
    pub exempt_functions: Vec<String>,
}

/// Documentation requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationStandards {
    /// Require documentation for public items
    pub require_public_docs: bool,
    /// Require documentation for private items
    pub require_private_docs: bool,
    /// Minimum documentation coverage percentage
    pub min_coverage: f64,
    /// Require examples in documentation
    pub require_examples: bool,
}

/// Banned patterns and practices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedPatterns {
    /// Ban unwrap() in production code
    pub ban_unwrap: bool,
    /// Ban expect() in production code
    pub ban_expect: bool,
    /// Ban panic!() in production code
    pub ban_panic: bool,
    /// Ban todo!() in production code
    pub ban_todo: bool,
    /// Ban unimplemented!() in production code
    pub ban_unimplemented: bool,
    /// Ban underscore variable names as error handling bandaids
    pub ban_underscore_bandaid: bool,
    /// Custom banned patterns
    pub custom_patterns: Vec<BannedPattern>,
}

/// A custom banned pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedPattern {
    /// Pattern name
    pub name: String,
    /// Regular expression to match
    pub pattern: String,
    /// Error message to display
    pub message: String,
    /// Severity level (error, warning, info)
    pub severity: String,
}

/// Dependency requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStandards {
    /// Maximum number of dependencies
    pub max_dependencies: usize,
    /// Require license checking
    pub require_license_check: bool,
    /// Banned licenses
    pub banned_licenses: Vec<String>,
    /// Required MSRV compatibility
    pub require_msrv_compatible: bool,
}

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStandards {
    /// Ban unsafe code
    pub ban_unsafe: bool,
    /// Require security audit
    pub require_audit: bool,
    /// Maximum allowed CVE score
    pub max_cve_score: f64,
}

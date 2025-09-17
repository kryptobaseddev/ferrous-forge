//! Rust coding standards definitions and enforcement
//!
//! This module defines the specific standards that Ferrous Forge enforces
//! and provides utilities for checking compliance.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Maximum parameters per function
    pub max_parameters: usize,
    /// Maximum cyclomatic complexity
    pub max_complexity: usize,
}

/// Documentation requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationStandards {
    /// Require documentation for all public items
    pub require_public_docs: bool,
    /// Require documentation for private items
    pub require_private_docs: bool,
    /// Require examples in documentation
    pub require_examples: bool,
    /// Minimum documentation length (words)
    pub min_doc_length: usize,
}

/// Banned patterns and practices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedPatterns {
    /// Ban underscore parameter naming (_param)
    pub ban_underscore_params: bool,
    /// Ban underscore let assignments (let _ =)
    pub ban_underscore_let: bool,
    /// Ban .unwrap() calls in production
    pub ban_unwrap: bool,
    /// Ban .expect() calls in production
    pub ban_expect: bool,
    /// Ban panic! macro in production
    pub ban_panic: bool,
    /// Ban todo! macro in production
    pub ban_todo: bool,
    /// Ban unimplemented! macro in production
    pub ban_unimplemented: bool,
    /// Custom banned patterns (regex)
    pub custom_banned: Vec<BannedPattern>,
}

/// A custom banned pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedPattern {
    /// Name of the pattern
    pub name: String,
    /// Regular expression to match
    pub pattern: String,
    /// Error message to show
    pub message: String,
    /// Whether this applies to test code
    pub applies_to_tests: bool,
}

/// Dependency requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStandards {
    /// Required dependencies for all projects
    pub required: Vec<String>,
    /// Recommended dependencies
    pub recommended: Vec<String>,
    /// Banned dependencies
    pub banned: Vec<String>,
    /// Version requirements
    pub version_requirements: HashMap<String, String>,
}

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStandards {
    /// Ban unsafe code blocks
    pub ban_unsafe: bool,
    /// Require security audit dependencies
    pub require_audit: bool,
    /// Minimum security audit frequency (days)
    pub audit_frequency_days: u32,
    /// Security-sensitive patterns to flag
    pub security_patterns: Vec<BannedPattern>,
}

impl Default for CodingStandards {
    fn default() -> Self {
        Self {
            edition: EditionStandards {
                required_edition: "2024".to_string(),
                min_rust_version: "1.85.0".to_string(),
                auto_upgrade: true,
            },
            file_limits: FileLimits {
                max_lines: 300,
                max_line_length: 100,
                exempt_files: vec![
                    "build.rs".to_string(),
                    "tests/".to_string(),
                    "benches/".to_string(),
                ],
            },
            function_limits: FunctionLimits {
                max_lines: 50,
                max_parameters: 5,
                max_complexity: 10,
            },
            documentation: DocumentationStandards {
                require_public_docs: true,
                require_private_docs: false,
                require_examples: false,
                min_doc_length: 10,
            },
            banned_patterns: BannedPatterns {
                ban_underscore_params: true,
                ban_underscore_let: true,
                ban_unwrap: true,
                ban_expect: true,
                ban_panic: true,
                ban_todo: true,
                ban_unimplemented: true,
                custom_banned: vec![],
            },
            dependencies: DependencyStandards {
                required: vec![
                    "thiserror".to_string(),
                    "anyhow".to_string(),
                ],
                recommended: vec![
                    "tokio".to_string(),
                    "tracing".to_string(),
                    "serde".to_string(),
                ],
                banned: vec![
                    "failure".to_string(), // Deprecated
                    "error-chain".to_string(), // Deprecated
                ],
                version_requirements: HashMap::new(),
            },
            security: SecurityStandards {
                ban_unsafe: true,
                require_audit: true,
                audit_frequency_days: 30,
                security_patterns: vec![
                    BannedPattern {
                        name: "hardcoded_secret".to_string(),
                        pattern: r#"(?i)(password|secret|key|token)\s*=\s*["'][^"']+["']"#.to_string(),
                        message: "Potential hardcoded secret detected".to_string(),
                        applies_to_tests: false,
                    },
                ],
            },
        }
    }
}

impl CodingStandards {
    /// Load standards from configuration
    pub fn load() -> Result<Self> {
        // For now, return defaults
        // TODO: Load from configuration file or remote source
        Ok(Self::default())
    }

    /// Save standards to configuration
    pub fn save(&self) -> Result<()> {
        // TODO: Save to configuration file
        Ok(())
    }

    /// Get all clippy rules based on these standards
    pub fn get_clippy_rules(&self) -> Vec<String> {
        let mut rules = vec![
            "-D warnings".to_string(),
        ];

        if self.banned_patterns.ban_unwrap {
            rules.push("-D clippy::unwrap_used".to_string());
        }
        
        if self.banned_patterns.ban_expect {
            rules.push("-D clippy::expect_used".to_string());
        }
        
        if self.banned_patterns.ban_panic {
            rules.push("-D clippy::panic".to_string());
        }
        
        if self.banned_patterns.ban_todo {
            rules.push("-D clippy::todo".to_string());
        }
        
        if self.banned_patterns.ban_unimplemented {
            rules.push("-D clippy::unimplemented".to_string());
        }

        if self.documentation.require_public_docs {
            rules.push("-D missing_docs".to_string());
        }

        if self.security.ban_unsafe {
            rules.push("-F unsafe_code".to_string());
        }

        // Add performance and style lints
        rules.extend([
            "-W clippy::pedantic".to_string(),
            "-W clippy::nursery".to_string(),
            "-W clippy::cargo".to_string(),
            "-D clippy::dbg_macro".to_string(),
            "-D clippy::print_stdout".to_string(),
            "-D clippy::print_stderr".to_string(),
        ]);

        rules
    }

    /// Generate clippy.toml configuration
    pub fn generate_clippy_config(&self) -> String {
        format!(
            r#"# Ferrous Forge - Rust Standards Enforcement
# Generated automatically - do not edit manually

msrv = "{}"
max-fn-params-bools = 2
max-struct-bools = 2
max-trait-bounds = 2
max-include-file-size = {}
min-ident-chars-threshold = 2
literal-representation-threshold = 1000
check-private-items = {}
missing-docs-allow-unused = false
allow-comparison-to-zero = false
allow-mixed-uninlined-format-args = false
allow-one-hash-in-raw-strings = false
allow-useless-vec-in-tests = false
allow-indexing-slicing-in-tests = false
allowed-idents-below-min-chars = ["i", "j", "x", "y", "z"]
allowed-wildcard-imports = []
allow-exact-repetitions = false
allow-private-module-inception = false
too-large-for-stack = 100
upper-case-acronyms-aggressive = true
allowed-scripts = ["Latin"]
disallowed-names = ["foo", "bar", "baz", "qux", "quux", "test", "tmp", "temp"]
unreadable-literal-lint-fractions = true
semicolon-inside-block-ignore-singleline = false
semicolon-outside-block-ignore-multiline = false
arithmetic-side-effects-allowed = []
"#,
            self.edition.min_rust_version,
            self.file_limits.max_lines * 1000, // Convert to bytes approximation
            self.documentation.require_private_docs,
        )
    }

    /// Check if a project complies with these standards
    pub async fn check_compliance(&self, project_path: &std::path::Path) -> Result<Vec<String>> {
        let mut violations = Vec::new();
        
        // Check Cargo.toml for edition
        let cargo_toml = project_path.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = tokio::fs::read_to_string(&cargo_toml).await?;
            if !content.contains(&format!(r#"edition = "{}""#, self.edition.required_edition)) {
                violations.push(format!("Project must use Rust Edition {}", self.edition.required_edition));
            }
        }

        // Additional compliance checks would go here
        
        Ok(violations)
    }
}
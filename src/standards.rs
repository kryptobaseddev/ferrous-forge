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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[test]
    fn test_coding_standards_default() {
        let standards = CodingStandards::default();
        
        // Test edition standards
        assert_eq!(standards.edition.required_edition, "2024");
        assert_eq!(standards.edition.min_rust_version, "1.85.0");
        assert!(standards.edition.auto_upgrade);
        
        // Test file limits
        assert_eq!(standards.file_limits.max_lines, 300);
        assert_eq!(standards.file_limits.max_line_length, 100);
        assert!(!standards.file_limits.exempt_files.is_empty());
        
        // Test function limits
        assert_eq!(standards.function_limits.max_lines, 50);
        assert_eq!(standards.function_limits.max_parameters, 5);
        assert_eq!(standards.function_limits.max_complexity, 10);
        
        // Test documentation standards
        assert!(standards.documentation.require_public_docs);
        assert!(!standards.documentation.require_private_docs);
        assert!(!standards.documentation.require_examples);
        assert_eq!(standards.documentation.min_doc_length, 10);
        
        // Test banned patterns
        assert!(standards.banned_patterns.ban_underscore_params);
        assert!(standards.banned_patterns.ban_underscore_let);
        assert!(standards.banned_patterns.ban_unwrap);
        assert!(standards.banned_patterns.ban_expect);
        assert!(standards.banned_patterns.ban_panic);
        assert!(standards.banned_patterns.ban_todo);
        assert!(standards.banned_patterns.ban_unimplemented);
        
        // Test dependencies
        assert!(!standards.dependencies.required.is_empty());
        assert!(!standards.dependencies.recommended.is_empty());
        assert!(!standards.dependencies.banned.is_empty());
        
        // Test security standards
        assert!(standards.security.ban_unsafe);
        assert!(standards.security.require_audit);
        assert_eq!(standards.security.audit_frequency_days, 30);
        assert!(!standards.security.security_patterns.is_empty());
    }

    #[test]
    fn test_edition_standards() {
        let standards = EditionStandards {
            required_edition: "2021".to_string(),
            min_rust_version: "1.70.0".to_string(),
            auto_upgrade: false,
        };
        
        assert_eq!(standards.required_edition, "2021");
        assert_eq!(standards.min_rust_version, "1.70.0");
        assert!(!standards.auto_upgrade);
    }

    #[test]
    fn test_file_limits() {
        let limits = FileLimits {
            max_lines: 500,
            max_line_length: 120,
            exempt_files: vec!["test.rs".to_string()],
        };
        
        assert_eq!(limits.max_lines, 500);
        assert_eq!(limits.max_line_length, 120);
        assert_eq!(limits.exempt_files.len(), 1);
        assert_eq!(limits.exempt_files[0], "test.rs");
    }

    #[test]
    fn test_function_limits() {
        let limits = FunctionLimits {
            max_lines: 100,
            max_parameters: 8,
            max_complexity: 15,
        };
        
        assert_eq!(limits.max_lines, 100);
        assert_eq!(limits.max_parameters, 8);
        assert_eq!(limits.max_complexity, 15);
    }

    #[test]
    fn test_documentation_standards() {
        let docs = DocumentationStandards {
            require_public_docs: false,
            require_private_docs: true,
            require_examples: true,
            min_doc_length: 20,
        };
        
        assert!(!docs.require_public_docs);
        assert!(docs.require_private_docs);
        assert!(docs.require_examples);
        assert_eq!(docs.min_doc_length, 20);
    }

    #[test]
    fn test_banned_patterns() {
        let patterns = BannedPatterns {
            ban_underscore_params: false,
            ban_underscore_let: false,
            ban_unwrap: false,
            ban_expect: false,
            ban_panic: false,
            ban_todo: false,
            ban_unimplemented: false,
            custom_banned: vec![],
        };
        
        assert!(!patterns.ban_underscore_params);
        assert!(!patterns.ban_underscore_let);
        assert!(!patterns.ban_unwrap);
        assert!(!patterns.ban_expect);
        assert!(!patterns.ban_panic);
        assert!(!patterns.ban_todo);
        assert!(!patterns.ban_unimplemented);
        assert!(patterns.custom_banned.is_empty());
    }

    #[test]
    fn test_banned_pattern() {
        let pattern = BannedPattern {
            name: "test_pattern".to_string(),
            pattern: r"test_.*".to_string(),
            message: "Test pattern found".to_string(),
            applies_to_tests: true,
        };
        
        assert_eq!(pattern.name, "test_pattern");
        assert_eq!(pattern.pattern, r"test_.*");
        assert_eq!(pattern.message, "Test pattern found");
        assert!(pattern.applies_to_tests);
    }

    #[test]
    fn test_dependency_standards() {
        let mut version_reqs = HashMap::new();
        version_reqs.insert("serde".to_string(), "1.0".to_string());
        
        let deps = DependencyStandards {
            required: vec!["anyhow".to_string()],
            recommended: vec!["serde".to_string()],
            banned: vec!["failure".to_string()],
            version_requirements: version_reqs,
        };
        
        assert_eq!(deps.required.len(), 1);
        assert_eq!(deps.recommended.len(), 1);
        assert_eq!(deps.banned.len(), 1);
        assert_eq!(deps.version_requirements.get("serde"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_security_standards() {
        let security = SecurityStandards {
            ban_unsafe: false,
            require_audit: false,
            audit_frequency_days: 60,
            security_patterns: vec![],
        };
        
        assert!(!security.ban_unsafe);
        assert!(!security.require_audit);
        assert_eq!(security.audit_frequency_days, 60);
        assert!(security.security_patterns.is_empty());
    }

    #[test]
    fn test_load_standards() {
        let result = CodingStandards::load();
        assert!(result.is_ok());
        
        let standards = result.expect("Should load standards");
        assert_eq!(standards.edition.required_edition, "2024");
    }

    #[test]
    fn test_save_standards() {
        let standards = CodingStandards::default();
        let result = standards.save();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_clippy_rules_all_enabled() {
        let standards = CodingStandards::default();
        let rules = standards.get_clippy_rules();
        
        assert!(!rules.is_empty());
        assert!(rules.contains(&"-D warnings".to_string()));
        assert!(rules.contains(&"-D clippy::unwrap_used".to_string()));
        assert!(rules.contains(&"-D clippy::expect_used".to_string()));
        assert!(rules.contains(&"-D clippy::panic".to_string()));
        assert!(rules.contains(&"-D clippy::todo".to_string()));
        assert!(rules.contains(&"-D clippy::unimplemented".to_string()));
        assert!(rules.contains(&"-D missing_docs".to_string()));
        assert!(rules.contains(&"-F unsafe_code".to_string()));
        assert!(rules.contains(&"-W clippy::pedantic".to_string()));
        assert!(rules.contains(&"-W clippy::nursery".to_string()));
        assert!(rules.contains(&"-W clippy::cargo".to_string()));
    }

    #[test]
    fn test_get_clippy_rules_disabled() {
        let mut standards = CodingStandards::default();
        standards.banned_patterns.ban_unwrap = false;
        standards.banned_patterns.ban_expect = false;
        standards.banned_patterns.ban_panic = false;
        standards.banned_patterns.ban_todo = false;
        standards.banned_patterns.ban_unimplemented = false;
        standards.documentation.require_public_docs = false;
        standards.security.ban_unsafe = false;
        
        let rules = standards.get_clippy_rules();
        
        assert!(rules.contains(&"-D warnings".to_string()));
        assert!(!rules.contains(&"-D clippy::unwrap_used".to_string()));
        assert!(!rules.contains(&"-D clippy::expect_used".to_string()));
        assert!(!rules.contains(&"-D clippy::panic".to_string()));
        assert!(!rules.contains(&"-D clippy::todo".to_string()));
        assert!(!rules.contains(&"-D clippy::unimplemented".to_string()));
        assert!(!rules.contains(&"-D missing_docs".to_string()));
        assert!(!rules.contains(&"-F unsafe_code".to_string()));
    }

    #[test]
    fn test_generate_clippy_config() {
        let standards = CodingStandards::default();
        let config = standards.generate_clippy_config();
        
        assert!(config.contains("Ferrous Forge"));
        assert!(config.contains(&standards.edition.min_rust_version));
        assert!(config.contains("max-fn-params-bools"));
        assert!(config.contains("check-private-items"));
        assert!(config.contains("disallowed-names"));
        assert!(config.contains("allowed-idents-below-min-chars"));
    }

    #[test]
    fn test_generate_clippy_config_private_docs() {
        let mut standards = CodingStandards::default();
        standards.documentation.require_private_docs = true;
        
        let config = standards.generate_clippy_config();
        assert!(config.contains("check-private-items = true"));
        
        standards.documentation.require_private_docs = false;
        let config = standards.generate_clippy_config();
        assert!(config.contains("check-private-items = false"));
    }

    #[tokio::test]
    async fn test_check_compliance_edition_2024() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        
        // Write Cargo.toml with Edition 2024
        fs::write(&cargo_toml, r#"
[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#).await.expect("Failed to write Cargo.toml");
        
        let standards = CodingStandards::default();
        let violations = standards.check_compliance(temp_dir.path()).await.expect("Check should succeed");
        
        assert!(violations.is_empty());
    }

    #[tokio::test]
    async fn test_check_compliance_wrong_edition() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        
        // Write Cargo.toml with wrong edition
        fs::write(&cargo_toml, r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#).await.expect("Failed to write Cargo.toml");
        
        let standards = CodingStandards::default();
        let violations = standards.check_compliance(temp_dir.path()).await.expect("Check should succeed");
        
        assert!(!violations.is_empty());
        assert!(violations[0].contains("Edition 2024"));
    }

    #[tokio::test]
    async fn test_check_compliance_no_cargo_toml() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        let standards = CodingStandards::default();
        let violations = standards.check_compliance(temp_dir.path()).await.expect("Check should succeed");
        
        // Should not fail if no Cargo.toml exists
        assert!(violations.is_empty());
    }

    // Integration tests for serialization/deserialization
    #[test]
    fn test_serialization_roundtrip() {
        let original = CodingStandards::default();
        
        let serialized = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: CodingStandards = serde_json::from_str(&serialized).expect("Should deserialize");
        
        assert_eq!(original.edition.required_edition, deserialized.edition.required_edition);
        assert_eq!(original.file_limits.max_lines, deserialized.file_limits.max_lines);
        assert_eq!(original.function_limits.max_lines, deserialized.function_limits.max_lines);
        assert_eq!(original.documentation.require_public_docs, deserialized.documentation.require_public_docs);
        assert_eq!(original.banned_patterns.ban_unwrap, deserialized.banned_patterns.ban_unwrap);
        assert_eq!(original.security.ban_unsafe, deserialized.security.ban_unsafe);
    }

    #[test]
    fn test_banned_pattern_serialization() {
        let pattern = BannedPattern {
            name: "test".to_string(),
            pattern: r"test_.*".to_string(),
            message: "Test message".to_string(),
            applies_to_tests: true,
        };
        
        let serialized = serde_json::to_string(&pattern).expect("Should serialize");
        let deserialized: BannedPattern = serde_json::from_str(&serialized).expect("Should deserialize");
        
        assert_eq!(pattern.name, deserialized.name);
        assert_eq!(pattern.pattern, deserialized.pattern);
        assert_eq!(pattern.message, deserialized.message);
        assert_eq!(pattern.applies_to_tests, deserialized.applies_to_tests);
    }

    // Property-based tests
    #[cfg(feature = "proptest")]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_file_limits_properties(
                max_lines in 1usize..10000,
                max_line_length in 50usize..500,
            ) {
                let limits = FileLimits {
                    max_lines,
                    max_line_length,
                    exempt_files: vec![],
                };
                
                prop_assert!(limits.max_lines > 0);
                prop_assert!(limits.max_line_length >= 50);
                prop_assert_eq!(limits.max_lines, max_lines);
                prop_assert_eq!(limits.max_line_length, max_line_length);
            }

            #[test]
            fn test_function_limits_properties(
                max_lines in 1usize..1000,
                max_parameters in 1usize..20,
                max_complexity in 1usize..50,
            ) {
                let limits = FunctionLimits {
                    max_lines,
                    max_parameters,
                    max_complexity,
                };
                
                prop_assert!(limits.max_lines > 0);
                prop_assert!(limits.max_parameters > 0);
                prop_assert!(limits.max_complexity > 0);
            }
        }
    }
}
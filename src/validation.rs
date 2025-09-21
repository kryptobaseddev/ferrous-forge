//! Rust code validation engine
//!
//! This module contains the core validation logic ported from the original Python implementation.
//! It enforces Ferrous Forge standards including:
//! - Zero underscore bandaid coding
//! - Edition 2024 enforcement
//! - File and function size limits
//! - Documentation requirements
//! - Security best practices

use crate::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// Types of violations that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationType {
    /// Underscore parameter or let assignment bandaid
    UnderscoreBandaid,
    /// Wrong Rust edition (not 2024)
    WrongEdition,
    /// File exceeds size limit
    FileTooLarge,
    /// Function exceeds size limit
    FunctionTooLarge,
    /// Line exceeds length limit
    LineTooLong,
    /// Use of .unwrap() or .expect() in production code
    UnwrapInProduction,
    /// Missing documentation
    MissingDocs,
    /// Missing required dependencies
    MissingDependencies,
    /// Rust version too old
    OldRustVersion,
}

/// A single standards violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// File where violation occurred
    pub file: PathBuf,
    /// Line number (0-based)
    pub line: usize,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: Severity,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    /// Error - must be fixed
    Error,
    /// Warning - should be fixed
    Warning,
    /// Info - good to know
    Info,
}

/// Result of running clippy
#[derive(Debug)]
pub struct ClippyResult {
    /// Whether clippy passed without errors
    pub success: bool,
    /// Combined stdout and stderr output
    pub output: String,
}

/// Rust project validator
pub struct RustValidator {
    /// Root directory of the project being validated
    project_root: PathBuf,
    /// Compiled regex patterns for efficient matching
    patterns: ValidationPatterns,
    /// Required crates for Context7 integration
    _required_crates: Vec<String>,
}

/// Compiled regex patterns for validation
struct ValidationPatterns {
    underscore_param: Regex,
    underscore_let: Regex,
    unwrap_call: Regex,
    expect_call: Regex,
    function_def: Regex,
}

/// Check if a pattern match is inside a string literal
fn is_in_string_literal(line: &str, pattern: &str) -> bool {
    let mut in_string = false;
    let mut escaped = false;
    let mut quote_char = '"';
    let bytes = line.as_bytes();
    
    // Find the pattern position first
    if let Some(pattern_pos) = line.find(pattern) {
        // Check if any quote comes before the pattern position
        for (i, &byte) in bytes.iter().enumerate() {
            if i >= pattern_pos {
                break;
            }
            
            if escaped {
                escaped = false;
                continue;
            }
            
            if byte == b'\\' {
                escaped = true;
                continue;
            }
            
            if byte == b'"' || byte == b'\'' {
                if !in_string {
                    in_string = true;
                    quote_char = byte as char;
                } else if byte as char == quote_char {
                    in_string = false;
                }
            }
        }
        in_string
    } else {
        false
    }
}

impl RustValidator {
    /// Create a new validator for the given project
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let patterns = ValidationPatterns {
            underscore_param: Regex::new(r"fn\s+\w+\([^)]*_\w+\s*:[^)]*\)")
                .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?,
            underscore_let: Regex::new(r"let\s+_\s*=")
                .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?,
            unwrap_call: Regex::new(r"\.unwrap\(\)")
                .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?,
            expect_call: Regex::new(r"\.expect\(")
                .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?,
            function_def: Regex::new(r"^(\s*)(?:pub\s+)?(?:async\s+)?fn\s+\w+")
                .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?,
        };

        let required_crates = vec![
            "tokio".to_string(),
            "thiserror".to_string(),
            "anyhow".to_string(),
            "tracing".to_string(),
        ];

        Ok(Self {
            project_root,
            patterns,
            _required_crates: required_crates,
        })
    }

    /// Validate the entire project
    pub async fn validate_project(&self) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Check Rust version first
        self.check_rust_version(&mut violations).await?;

        // Find all relevant files
        let rust_files = self.find_rust_files().await?;
        let cargo_files = self.find_cargo_files().await?;

        tracing::info!(
            "Found {} Rust files and {} Cargo.toml files",
            rust_files.len(),
            cargo_files.len()
        );

        // Validate Cargo.toml files
        for cargo_file in cargo_files {
            self.validate_cargo_toml(&cargo_file, &mut violations)
                .await?;
        }

        // Validate Rust source files
        for rust_file in rust_files {
            // Skip target directory
            if rust_file.to_string_lossy().contains("target/") {
                continue;
            }

            self.validate_rust_file(&rust_file, &mut violations).await?;
        }

        Ok(violations)
    }

    /// Generate a human-readable report from violations
    pub fn generate_report(&self, violations: &[Violation]) -> String {
        if violations.is_empty() {
            return "✅ All Rust validation checks passed! Code meets Ferrous Forge standards."
                .to_string();
        }

        let mut report = format!(
            "❌ Found {} violations of Ferrous Forge standards:\n\n",
            violations.len()
        );

        // Group by violation type
        let mut by_type = std::collections::HashMap::new();
        for violation in violations {
            by_type
                .entry(&violation.violation_type)
                .or_insert_with(Vec::new)
                .push(violation);
        }

        for (violation_type, violations) in by_type {
            let type_name = format!("{:?}", violation_type)
                .to_uppercase()
                .replace('_', " ");

            report.push_str(&format!(
                "🚨 {} ({} violations):\n",
                type_name,
                violations.len()
            ));

            for violation in violations.iter().take(10) {
                report.push_str(&format!(
                    "  {}:{} - {}\n",
                    violation.file.display(),
                    violation.line + 1,
                    violation.message
                ));
            }

            if violations.len() > 10 {
                report.push_str(&format!("  ... and {} more\n", violations.len() - 10));
            }

            report.push('\n');
        }

        report
    }

    /// Run clippy with strict configuration
    pub async fn run_clippy(&self) -> Result<ClippyResult> {
        let output = Command::new("cargo")
            .args(&[
                "clippy",
                "--all-features",
                "--",
                "-D",
                "warnings",
                "-D",
                "clippy::unwrap_used",
                "-D",
                "clippy::expect_used",
                "-D",
                "clippy::panic",
                "-D",
                "clippy::unimplemented",
                "-D",
                "clippy::todo",
            ])
            .current_dir(&self.project_root)
            .output()
            .map_err(|e| Error::process(format!("Failed to run clippy: {}", e)))?;

        Ok(ClippyResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string()
                + &String::from_utf8_lossy(&output.stderr),
        })
    }

    async fn check_rust_version(&self, violations: &mut Vec<Violation>) -> Result<()> {
        let output = Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(|_| Error::validation("Rust compiler not found"))?;

        let version_line = String::from_utf8_lossy(&output.stdout);

        // Extract version (e.g., "rustc 1.85.0" -> "1.85.0")
        let version_regex = Regex::new(r"rustc (\d+)\.(\d+)\.(\d+)")
            .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?;

        if let Some(captures) = version_regex.captures(&version_line) {
            let major: u32 = captures[1].parse().unwrap_or(0);
            let minor: u32 = captures[2].parse().unwrap_or(0);

            if major < 1 || (major == 1 && minor < 82) {
                violations.push(Violation {
                    violation_type: ViolationType::OldRustVersion,
                    file: PathBuf::from("<system>"),
                    line: 0,
                    message: format!(
                        "Rust version {}.{} is too old. Minimum required: 1.82.0",
                        major, minor
                    ),
                    severity: Severity::Error,
                });
            }
        } else {
            violations.push(Violation {
                violation_type: ViolationType::OldRustVersion,
                file: PathBuf::from("<system>"),
                line: 0,
                message: "Could not parse Rust version".to_string(),
                severity: Severity::Error,
            });
        }

        Ok(())
    }

    async fn find_rust_files(&self) -> Result<Vec<PathBuf>> {
        let mut rust_files = Vec::new();
        self.collect_rust_files_recursive(&self.project_root, &mut rust_files)?;
        Ok(rust_files)
    }

    fn collect_rust_files_recursive(
        &self,
        path: &Path,
        rust_files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "rs" {
                    rust_files.push(path.to_path_buf());
                }
            }
        } else if path.is_dir() {
            let entries = std::fs::read_dir(path)?;
            for entry in entries {
                let entry = entry?;
                self.collect_rust_files_recursive(&entry.path(), rust_files)?;
            }
        }

        Ok(())
    }

    async fn find_cargo_files(&self) -> Result<Vec<PathBuf>> {
        let mut cargo_files = Vec::new();
        self.collect_cargo_files_recursive(&self.project_root, &mut cargo_files)?;
        Ok(cargo_files)
    }

    fn collect_cargo_files_recursive(
        &self,
        path: &Path,
        cargo_files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if path.is_file() {
            if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
                cargo_files.push(path.to_path_buf());
            }
        } else if path.is_dir() && !path.to_string_lossy().contains("target/") {
            let entries = std::fs::read_dir(path)?;
            for entry in entries {
                let entry = entry?;
                self.collect_cargo_files_recursive(&entry.path(), cargo_files)?;
            }
        }

        Ok(())
    }

    /// Validates a Cargo.toml file for standards compliance
    ///
    /// Checks that the Cargo.toml uses Edition 2021 or 2024.
    pub async fn validate_cargo_toml(
        &self,
        cargo_file: &Path,
        violations: &mut Vec<Violation>,
    ) -> Result<()> {
        let content = fs::read_to_string(cargo_file).await?;
        let lines: Vec<&str> = content.lines().collect();

        // Check for Edition 2021 or 2024
        let mut edition_found = false;
        for (i, line) in lines.iter().enumerate() {
            if line.contains("edition") {
                if !line.contains("2021") && !line.contains("2024") {
                    violations.push(Violation {
                        violation_type: ViolationType::WrongEdition,
                        file: cargo_file.to_path_buf(),
                        line: i + 1,
                        message: "Must use Edition 2021 or 2024".to_string(),
                        severity: Severity::Error,
                    });
                }
                edition_found = true;
                break;
            }
        }

        if !edition_found {
            violations.push(Violation {
                violation_type: ViolationType::WrongEdition,
                file: cargo_file.to_path_buf(),
                line: 0,
                message: "Missing edition specification - must be '2021' or '2024'".to_string(),
                severity: Severity::Error,
            });
        }

        Ok(())
    }

    /// Validates a Rust source file for standards compliance
    ///
    /// Checks for file size limits, line length, function size,
    /// underscore bandaids, and unwrap/expect usage.
    pub async fn validate_rust_file(
        &self,
        rust_file: &Path,
        violations: &mut Vec<Violation>,
    ) -> Result<()> {
        let content = fs::read_to_string(rust_file).await?;
        let lines: Vec<&str> = content.lines().collect();
        
        // Check if this is a test or benchmark file
        let path_str = rust_file.to_string_lossy();
        let is_test_file = path_str.contains("/tests/") 
            || path_str.contains("/benches/") 
            || path_str.contains("/test_")
            || path_str.ends_with("_test.rs")
            || path_str.ends_with("_bench.rs");

        // Check for allow attributes at the top of the file
        let mut allow_unwrap = false;
        let mut allow_expect = false;

        for line in &lines {
            let line_stripped = line.trim();
            if line_stripped.starts_with("#![allow(") {
                if line_stripped.contains("clippy::unwrap_used") {
                    allow_unwrap = true;
                }
                if line_stripped.contains("clippy::expect_used") {
                    allow_expect = true;
                }
            }
            // Also check for allow attributes that might be split across lines
            if line_stripped.contains("clippy::unwrap_used") {
                allow_unwrap = true;
            }
            if line_stripped.contains("clippy::expect_used") {
                allow_expect = true;
            }
            // Stop checking after we hit the first non-attribute line (but allow doc comments)
            if !line_stripped.starts_with("#")
                && !line_stripped.starts_with("//!")
                && !line_stripped.is_empty()
            {
                break;
            }
        }

        // Check file size limit (300 lines)
        if lines.len() > 300 {
            violations.push(Violation {
                violation_type: ViolationType::FileTooLarge,
                file: rust_file.to_path_buf(),
                line: lines.len() - 1,
                message: format!("File has {} lines, maximum allowed is 300", lines.len()),
                severity: Severity::Error,
            });
        }

        // Check line lengths (100 character limit)
        for (i, line) in lines.iter().enumerate() {
            if line.len() > 100 {
                violations.push(Violation {
                    violation_type: ViolationType::LineTooLong,
                    file: rust_file.to_path_buf(),
                    line: i + 1,
                    message: format!("Line has {} characters, maximum allowed is 100", line.len()),
                    severity: Severity::Warning,
                });
            }
        }

        let mut in_test_block = false;
        let mut current_function_start: Option<usize> = None;

        for (i, line) in lines.iter().enumerate() {
            let line_stripped = line.trim();

            // Track test blocks
            if line_stripped.contains("[test]") || line_stripped.contains("[cfg(test)]") {
                in_test_block = true;
            }

            // Track function boundaries
            if self.patterns.function_def.is_match(line) {
                // Check previous function size
                if let Some(start) = current_function_start {
                    let func_lines = i - start;
                    if func_lines > 50 {
                        violations.push(Violation {
                            violation_type: ViolationType::FunctionTooLarge,
                            file: rust_file.to_path_buf(),
                            line: start + 1,
                            message: format!(
                                "Function has {} lines, maximum allowed is 50",
                                func_lines
                            ),
                            severity: Severity::Error,
                        });
                    }
                }
                current_function_start = Some(i);
            }

            // Check for underscore bandaid coding
            if self.patterns.underscore_param.is_match(line) {
                violations.push(Violation {
                    violation_type: ViolationType::UnderscoreBandaid,
                    file: rust_file.to_path_buf(),
                    line: i + 1,
                    message: "BANNED: Underscore parameter (_param) - \
                             fix the design instead of hiding warnings".to_string(),
                    severity: Severity::Error,
                });
            }

            if self.patterns.underscore_let.is_match(line) {
                violations.push(Violation {
                    violation_type: ViolationType::UnderscoreBandaid,
                    file: rust_file.to_path_buf(),
                    line: i + 1,
                    message: "BANNED: Underscore assignment (let _ =) - handle errors properly"
                        .to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for .unwrap() in production code (not in tests or if allowed)
            if !in_test_block && !is_test_file && !allow_unwrap 
                && self.patterns.unwrap_call.is_match(line)
                && !is_in_string_literal(line, ".unwrap()") {
                violations.push(Violation {
                    violation_type: ViolationType::UnwrapInProduction,
                    file: rust_file.to_path_buf(),
                    line: i + 1,
                    message:
                        "BANNED: .unwrap() in production code - use proper error handling with ?"
                            .to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for .expect() in production code (not in tests or if allowed)
            if !in_test_block && !is_test_file && !allow_expect 
                && self.patterns.expect_call.is_match(line)
                && !is_in_string_literal(line, ".expect(") {
                violations.push(Violation {
                    violation_type: ViolationType::UnwrapInProduction,
                    file: rust_file.to_path_buf(),
                    line: i + 1,
                    message:
                        "BANNED: .expect() in production code - use proper error handling with ?"
                            .to_string(),
                    severity: Severity::Error,
                });
            }

            // Reset test block tracking
            if line_stripped.starts_with('}') && in_test_block {
                in_test_block = false;
            }
        }

        // Check the last function if any
        if let Some(start) = current_function_start {
            let func_lines = lines.len() - start;
            if func_lines > 50 {
                violations.push(Violation {
                    violation_type: ViolationType::FunctionTooLarge,
                    file: rust_file.to_path_buf(),
                    line: start,
                    message: format!("Function has {} lines, maximum allowed is 50", func_lines),
                    severity: Severity::Error,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)] // expect() is fine in tests
#[allow(clippy::unwrap_used)] // unwrap() is fine in tests
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[test]
    fn test_violation_type_variants() {
        let types = [
            ViolationType::UnderscoreBandaid,
            ViolationType::WrongEdition,
            ViolationType::FileTooLarge,
            ViolationType::FunctionTooLarge,
            ViolationType::LineTooLong,
            ViolationType::UnwrapInProduction,
            ViolationType::MissingDocs,
            ViolationType::MissingDependencies,
            ViolationType::OldRustVersion,
        ];

        // Test that variants are distinct
        for (i, type1) in types.iter().enumerate() {
            for (j, type2) in types.iter().enumerate() {
                if i != j {
                    assert_ne!(type1, type2);
                }
            }
        }
    }

    #[test]
    fn test_severity_variants() {
        let error = Severity::Error;
        let warning = Severity::Warning;
        let info = Severity::Info;

        // Just test that we can create instances
        assert!(matches!(error, Severity::Error));
        assert!(matches!(warning, Severity::Warning));
        assert!(matches!(info, Severity::Info));
    }

    #[test]
    fn test_violation_creation() {
        let violation = Violation {
            violation_type: ViolationType::UnderscoreBandaid,
            file: PathBuf::from("test.rs"),
            line: 10,
            message: "Test violation".to_string(),
            severity: Severity::Error,
        };

        assert_eq!(violation.violation_type, ViolationType::UnderscoreBandaid);
        assert_eq!(violation.file, PathBuf::from("test.rs"));
        assert_eq!(violation.line, 10);
        assert_eq!(violation.message, "Test violation");
        matches!(violation.severity, Severity::Error);
    }

    #[test]
    fn test_clippy_result() {
        let result = ClippyResult {
            success: true,
            output: "All checks passed".to_string(),
        };

        assert!(result.success);
        assert_eq!(result.output, "All checks passed");
    }

    #[tokio::test]
    async fn test_rust_validator_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let validator = RustValidator::new(temp_dir.path().to_path_buf());

        assert!(validator.is_ok());
        let validator = validator.expect("Should create validator");
        assert_eq!(validator.project_root, temp_dir.path());
    }

    #[tokio::test]
    async fn test_generate_report_no_violations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let violations = vec![];
        let report = validator.generate_report(&violations);

        assert!(report.contains("✅"));
        assert!(report.contains("All Rust validation checks passed"));
    }

    #[tokio::test]
    async fn test_generate_report_with_violations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let violations = vec![
            Violation {
                violation_type: ViolationType::UnderscoreBandaid,
                file: PathBuf::from("test.rs"),
                line: 10,
                message: "Underscore parameter".to_string(),
                severity: Severity::Error,
            },
            Violation {
                violation_type: ViolationType::WrongEdition,
                file: PathBuf::from("Cargo.toml"),
                line: 5,
                message: "Wrong edition".to_string(),
                severity: Severity::Error,
            },
        ];

        let report = validator.generate_report(&violations);

        assert!(report.contains("❌"));
        assert!(report.contains("Found 2 violations"));
        assert!(report.contains("UNDERSCOREBANDAID"));
        assert!(report.contains("WRONGEDITION"));
        assert!(report.contains("test.rs:11"));
        assert!(report.contains("Cargo.toml:6"));
    }

    #[tokio::test]
    async fn test_validate_cargo_toml_correct_edition() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        fs::write(
            &cargo_toml,
            r#"
[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#,
        )
        .await
        .expect("Failed to write Cargo.toml");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_cargo_toml(&cargo_toml, &mut violations)
            .await
            .expect("Should validate");

        assert!(violations.is_empty());
    }

    #[tokio::test]
    async fn test_validate_cargo_toml_wrong_edition() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        fs::write(
            &cargo_toml,
            r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .await
        .expect("Failed to write Cargo.toml");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_cargo_toml(&cargo_toml, &mut violations)
            .await
            .expect("Should validate");

        assert_eq!(violations.len(), 0); // 2021 is now valid
    }

    #[tokio::test]
    async fn test_validate_cargo_toml_missing_edition() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        fs::write(
            &cargo_toml,
            r#"
[package]
name = "test"
version = "0.1.0"
"#,
        )
        .await
        .expect("Failed to write Cargo.toml");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_cargo_toml(&cargo_toml, &mut violations)
            .await
            .expect("Should validate");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].violation_type, ViolationType::WrongEdition);
        assert!(violations[0].message.contains("Missing edition"));
    }

    #[tokio::test]
    async fn test_validate_rust_file_size_limit() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let rust_file = temp_dir.path().join("test.rs");

        // Create a file with over 300 lines
        let content = (0..350)
            .map(|i| format!("// Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&rust_file, content)
            .await
            .expect("Failed to write Rust file");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_rust_file(&rust_file, &mut violations)
            .await
            .expect("Should validate");

        let file_size_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::FileTooLarge)
            .collect();

        assert_eq!(file_size_violations.len(), 1);
        assert!(file_size_violations[0].message.contains("350 lines"));
    }

    #[tokio::test]
    async fn test_validate_rust_file_line_length() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let rust_file = temp_dir.path().join("test.rs");

        let long_line = "// ".to_string() + &"x".repeat(150);
        fs::write(&rust_file, long_line)
            .await
            .expect("Failed to write Rust file");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_rust_file(&rust_file, &mut violations)
            .await
            .expect("Should validate");

        let line_length_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::LineTooLong)
            .collect();

        assert_eq!(line_length_violations.len(), 1);
        assert!(line_length_violations[0].message.contains("153 characters"));
    }

    #[tokio::test]
    async fn test_validate_rust_file_underscore_bandaid() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let rust_file = temp_dir.path().join("test.rs");

        let content = r"
fn test_function(_param: String) {
    let _ = some_operation();
}
";
        fs::write(&rust_file, content)
            .await
            .expect("Failed to write Rust file");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_rust_file(&rust_file, &mut violations)
            .await
            .expect("Should validate");

        let bandaid_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::UnderscoreBandaid)
            .collect();

        assert_eq!(bandaid_violations.len(), 2); // One for param, one for let
        assert!(bandaid_violations
            .iter()
            .any(|v| v.message.contains("parameter")));
        assert!(bandaid_violations
            .iter()
            .any(|v| v.message.contains("assignment")));
    }

    #[tokio::test]
    async fn test_validate_rust_file_unwrap_in_production() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let rust_file = temp_dir.path().join("test.rs");

        let content = r#"
fn production_code() {
    let value = some_result.unwrap();
    let other = another_result.expect("message");
}

#[test]
fn test_code() {
    let value = some_result.unwrap(); // This should be allowed
}
"#;
        fs::write(&rust_file, content)
            .await
            .expect("Failed to write Rust file");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_rust_file(&rust_file, &mut violations)
            .await
            .expect("Should validate");

        let unwrap_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::UnwrapInProduction)
            .collect();

        // Should find 2 violations in production code, but none in test code
        assert_eq!(unwrap_violations.len(), 2);
        assert!(unwrap_violations
            .iter()
            .any(|v| v.message.contains("unwrap")));
        assert!(unwrap_violations
            .iter()
            .any(|v| v.message.contains("expect")));
    }

    #[tokio::test]
    async fn test_find_rust_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create some Rust files
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)
            .await
            .expect("Failed to create src dir");

        fs::write(src_dir.join("main.rs"), "fn main() {}")
            .await
            .expect("Failed to write main.rs");
        fs::write(src_dir.join("lib.rs"), "// lib")
            .await
            .expect("Failed to write lib.rs");
        fs::write(temp_dir.path().join("build.rs"), "// build")
            .await
            .expect("Failed to write build.rs");

        // Create non-Rust file
        fs::write(temp_dir.path().join("README.md"), " Test")
            .await
            .expect("Failed to write README");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let rust_files = validator
            .find_rust_files()
            .await
            .expect("Should find files");

        assert_eq!(rust_files.len(), 3);
        assert!(rust_files
            .iter()
            .any(|f| f.file_name().expect("file name") == "main.rs"));
        assert!(rust_files
            .iter()
            .any(|f| f.file_name().expect("file name") == "lib.rs"));
        assert!(rust_files
            .iter()
            .any(|f| f.file_name().expect("file name") == "build.rs"));
    }

    #[tokio::test]
    async fn test_find_cargo_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create Cargo.toml files
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]")
            .await
            .expect("Failed to write Cargo.toml");

        let sub_dir = temp_dir.path().join("sub_project");
        fs::create_dir(&sub_dir)
            .await
            .expect("Failed to create sub dir");
        fs::write(sub_dir.join("Cargo.toml"), "[package]")
            .await
            .expect("Failed to write sub Cargo.toml");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let cargo_files = validator
            .find_cargo_files()
            .await
            .expect("Should find files");

        assert_eq!(cargo_files.len(), 2);
        assert!(cargo_files
            .iter()
            .all(|f| f.file_name().expect("file name") == "Cargo.toml"));
    }

    #[tokio::test]
    async fn test_validate_project_integration() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create a basic Rust project structure
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)
            .await
            .expect("Failed to create src dir");

        // Cargo.toml with correct edition
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#,
        )
        .await
        .expect("Failed to write Cargo.toml");

        // Good Rust file
        fs::write(
            src_dir.join("lib.rs"),
            r"
//! Test library

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
",
        )
        .await
        .expect("Failed to write lib.rs");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let violations = validator.validate_project().await.expect("Should validate");

        // Should pass validation (only rust version check might fail depending on system)
        let non_rust_version_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.violation_type != ViolationType::OldRustVersion)
            .collect();

        assert!(non_rust_version_violations.is_empty());
    }

    #[test]
    fn test_serialization() {
        let violation = Violation {
            violation_type: ViolationType::UnderscoreBandaid,
            file: PathBuf::from("test.rs"),
            line: 10,
            message: "Test violation".to_string(),
            severity: Severity::Error,
        };

        let serialized = serde_json::to_string(&violation).expect("Should serialize");
        let deserialized: Violation =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(violation.violation_type, deserialized.violation_type);
        assert_eq!(violation.file, deserialized.file);
        assert_eq!(violation.line, deserialized.line);
        assert_eq!(violation.message, deserialized.message);
    }

    #[tokio::test]
    async fn test_clippy_run() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create a minimal Cargo.toml
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#,
        )
        .await
        .expect("Failed to write Cargo.toml");

        // Create src directory with basic lib.rs
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)
            .await
            .expect("Failed to create src dir");
        fs::write(src_dir.join("lib.rs"), "// Empty lib")
            .await
            .expect("Failed to write lib.rs");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let result = validator.run_clippy().await;

        // The result might succeed or fail depending on the environment
        // but we should get a ClippyResult
        match result {
            Ok(clippy_result) => {
                assert!(!clippy_result.output.is_empty());
            }
            Err(_) => {
                // This is acceptable as clippy might not be available in test environment
            }
        }
    }

    // Edge case tests
    #[tokio::test]
    async fn test_empty_rust_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let rust_file = temp_dir.path().join("empty.rs");

        fs::write(&rust_file, "")
            .await
            .expect("Failed to write empty file");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_rust_file(&rust_file, &mut violations)
            .await
            .expect("Should validate");

        // Empty file should not generate violations
        assert!(violations.is_empty());
    }

    #[tokio::test]
    async fn test_function_size_limit() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let rust_file = temp_dir.path().join("test.rs");

        // Create a function with more than 50 lines
        let mut content = String::from("fn large_function() {\n");
        for i in 0..60 {
            content.push_str(&format!("    let x{} = {};\n", i, i));
        }
        content.push_str("}\n\nfn small_function() {\n    let x = 1;\n}\n");

        fs::write(&rust_file, content)
            .await
            .expect("Failed to write Rust file");

        let validator =
            RustValidator::new(temp_dir.path().to_path_buf()).expect("Should create validator");

        let mut violations = Vec::new();
        validator
            .validate_rust_file(&rust_file, &mut violations)
            .await
            .expect("Should validate");

        let function_size_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::FunctionTooLarge)
            .collect();

        assert_eq!(function_size_violations.len(), 1);
        assert!(function_size_violations[0].message.contains("63 lines"));
    }
}

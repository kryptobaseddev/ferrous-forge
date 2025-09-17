//! Rust code validation engine
//!
//! This module contains the core validation logic ported from the original Python implementation.
//! It enforces Ferrous Forge standards including:
//! - Zero underscore bandaid coding
//! - Edition 2024 enforcement
//! - File and function size limits
//! - Documentation requirements
//! - Security best practices

use crate::{Result, Error};
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
    required_crates: Vec<String>,
}

/// Compiled regex patterns for validation
struct ValidationPatterns {
    underscore_param: Regex,
    underscore_let: Regex,
    unwrap_call: Regex,
    expect_call: Regex,
    function_def: Regex,
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
            required_crates,
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

        tracing::info!("Found {} Rust files and {} Cargo.toml files", 
            rust_files.len(), cargo_files.len());

        // Validate Cargo.toml files
        for cargo_file in cargo_files {
            self.validate_cargo_toml(&cargo_file, &mut violations).await?;
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
            return "✅ All Rust validation checks passed! Code meets Ferrous Forge standards.".to_string();
        }

        let mut report = format!("❌ Found {} violations of Ferrous Forge standards:\n\n", violations.len());

        // Group by violation type
        let mut by_type = std::collections::HashMap::new();
        for violation in violations {
            by_type.entry(&violation.violation_type)
                .or_insert_with(Vec::new)
                .push(violation);
        }

        for (violation_type, violations) in by_type {
            let type_name = format!("{:?}", violation_type)
                .to_uppercase()
                .replace('_', " ");
                
            report.push_str(&format!("🚨 {} ({} violations):\n", type_name, violations.len()));
            
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
                "-D", "warnings",
                "-D", "clippy::unwrap_used",
                "-D", "clippy::expect_used",
                "-D", "clippy::panic",
                "-D", "clippy::unimplemented",
                "-D", "clippy::todo",
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
        if let Some(captures) = Regex::new(r"rustc (\d+)\.(\d+)\.(\d+)")
            .unwrap()
            .captures(&version_line)
        {
            let major: u32 = captures[1].parse().unwrap_or(0);
            let minor: u32 = captures[2].parse().unwrap_or(0);
            
            if major < 1 || (major == 1 && minor < 85) {
                violations.push(Violation {
                    violation_type: ViolationType::OldRustVersion,
                    file: PathBuf::from("<system>"),
                    line: 0,
                    message: format!("Rust version {}.{} is too old. Minimum required: 1.85.0", major, minor),
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

    fn collect_rust_files_recursive(&self, path: &Path, rust_files: &mut Vec<PathBuf>) -> Result<()> {
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

    fn collect_cargo_files_recursive(&self, path: &Path, cargo_files: &mut Vec<PathBuf>) -> Result<()> {
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

    async fn validate_cargo_toml(&self, cargo_file: &Path, violations: &mut Vec<Violation>) -> Result<()> {
        let content = fs::read_to_string(cargo_file).await?;
        let lines: Vec<&str> = content.lines().collect();

        // Check for Edition 2024
        let mut edition_found = false;
        for (i, line) in lines.iter().enumerate() {
            if line.contains("edition") {
                if !line.contains("2024") {
                    violations.push(Violation {
                        violation_type: ViolationType::WrongEdition,
                        file: cargo_file.to_path_buf(),
                        line: i,
                        message: "Must use Edition 2024, not 2021 or older".to_string(),
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
                message: "Missing edition specification - must be '2024'".to_string(),
                severity: Severity::Error,
            });
        }

        Ok(())
    }

    async fn validate_rust_file(&self, rust_file: &Path, violations: &mut Vec<Violation>) -> Result<()> {
        let content = fs::read_to_string(rust_file).await?;
        let lines: Vec<&str> = content.lines().collect();

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
                    line: i,
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
            if line_stripped.contains("#[test]") || line_stripped.contains("#[cfg(test)]") {
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
                            line: start,
                            message: format!("Function has {} lines, maximum allowed is 50", func_lines),
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
                    line: i,
                    message: "BANNED: Underscore parameter (_param) - fix the design instead of hiding warnings".to_string(),
                    severity: Severity::Error,
                });
            }

            if self.patterns.underscore_let.is_match(line) {
                violations.push(Violation {
                    violation_type: ViolationType::UnderscoreBandaid,
                    file: rust_file.to_path_buf(),
                    line: i,
                    message: "BANNED: Underscore assignment (let _ =) - handle errors properly".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for .unwrap() in production code (not in tests)
            if !in_test_block && self.patterns.unwrap_call.is_match(line) {
                violations.push(Violation {
                    violation_type: ViolationType::UnwrapInProduction,
                    file: rust_file.to_path_buf(),
                    line: i,
                    message: "BANNED: .unwrap() in production code - use proper error handling with ?".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for .expect() in production code
            if !in_test_block && self.patterns.expect_call.is_match(line) {
                violations.push(Violation {
                    violation_type: ViolationType::UnwrapInProduction,
                    file: rust_file.to_path_buf(),
                    line: i,
                    message: "BANNED: .expect() in production code - use proper error handling with ?".to_string(),
                    severity: Severity::Error,
                });
            }

            // Reset test block tracking
            if line_stripped.starts_with('}') && in_test_block {
                in_test_block = false;
            }
        }

        Ok(())
    }
}
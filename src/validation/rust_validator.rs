//! Core Rust validation - modularized structure

pub mod file_checks;
pub mod patterns;

use crate::validation::{Severity, Violation, ViolationType};
use crate::{Error, Result};
use file_checks::{validate_cargo_toml, validate_rust_file};
use patterns::ValidationPatterns;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result from running clippy
#[derive(Debug, Clone)]
pub struct ClippyResult {
    /// Whether clippy ran successfully
    pub success: bool,
    /// Output from clippy command
    pub output: String,
}

/// Core Rust validator
pub struct RustValidator {
    /// Root directory of the project to validate
    project_root: PathBuf,
    /// Compiled regex patterns for validation
    patterns: ValidationPatterns,
}

impl RustValidator {
    /// Create a new validator for the given project
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let patterns = ValidationPatterns::new()?;
        Ok(Self {
            project_root,
            patterns,
        })
    }

    /// Get reference to validation patterns
    pub fn patterns(&self) -> &ValidationPatterns {
        &self.patterns
    }

    /// Validate all Rust code in the project
    pub async fn validate_project(&self) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Check Rust version
        self.check_rust_version(&mut violations).await?;

        // Find and validate all Cargo.toml files
        let cargo_files = self.find_cargo_files().await?;
        for cargo_file in cargo_files {
            validate_cargo_toml(&cargo_file, &mut violations).await?;
        }

        // Find and validate all Rust source files
        let rust_files = self.find_rust_files().await?;
        for rust_file in rust_files {
            validate_rust_file(&rust_file, &mut violations, &self.patterns).await?;
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
        // Skip any path containing target directory
        if path.to_string_lossy().contains("target/") {
            return Ok(());
        }

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
                let entry_path = entry.path();
                // Skip target directory entries
                if entry_path.file_name() == Some(std::ffi::OsStr::new("target")) {
                    continue;
                }
                self.collect_rust_files_recursive(&entry_path, rust_files)?;
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
        // Skip any path containing target directory
        if path.to_string_lossy().contains("target/") {
            return Ok(());
        }

        if path.is_file() {
            if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
                cargo_files.push(path.to_path_buf());
            }
        } else if path.is_dir() {
            let entries = std::fs::read_dir(path)?;
            for entry in entries {
                let entry = entry?;
                let entry_path = entry.path();
                // Skip target directory entries
                if entry_path.file_name() == Some(std::ffi::OsStr::new("target")) {
                    continue;
                }
                self.collect_cargo_files_recursive(&entry_path, cargo_files)?;
            }
        }

        Ok(())
    }
}

// Re-export for backwards compatibility
pub use patterns::is_in_string_literal;

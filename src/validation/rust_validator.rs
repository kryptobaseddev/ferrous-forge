//! Core Rust validation - modularized structure

/// Individual file validation checks for Rust source and Cargo.toml.
pub mod file_checks;
/// Regex-based validation patterns for code style enforcement.
pub mod patterns;

use crate::config::Config;
use crate::validation::{Severity, Violation, ViolationType};
use crate::{Error, Result};
use file_checks::{validate_cargo_toml_full, validate_rust_file};
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
    /// Active configuration (drives limits and locked settings)
    config: Config,
}

impl RustValidator {
    /// Create a new validator with default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the validation regex patterns fail to compile.
    pub fn new(project_root: PathBuf) -> Result<Self> {
        Self::with_config(project_root, Config::default())
    }

    /// Create a new validator with explicit configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the validation regex patterns fail to compile.
    pub fn with_config(project_root: PathBuf, config: Config) -> Result<Self> {
        let patterns = ValidationPatterns::new()?;
        Ok(Self {
            project_root,
            patterns,
            config,
        })
    }

    /// Get reference to validation patterns
    pub fn patterns(&self) -> &ValidationPatterns {
        &self.patterns
    }

    /// Validate all Rust code in the project
    ///
    /// # Errors
    ///
    /// Returns an error if files cannot be read, the Rust compiler version
    /// cannot be determined, or `Cargo.toml` files cannot be parsed.
    pub async fn validate_project(&self) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Check installed Rust version against config minimum
        self.check_rust_version(&mut violations).await?;

        // Find and validate all Cargo.toml files
        let cargo_files = self.find_cargo_files().await?;
        for cargo_file in cargo_files {
            validate_cargo_toml_full(
                &cargo_file,
                &mut violations,
                &self.config.required_edition,
                &self.config.required_rust_version,
            )
            .await?;
        }

        // Find and validate all Rust source files
        let rust_files = self.find_rust_files().await?;
        for rust_file in rust_files {
            validate_rust_file(
                &rust_file,
                &mut violations,
                &self.patterns,
                self.config.max_file_lines,
                self.config.max_function_lines,
            )
            .await?;
        }

        // Check version consistency (SSoT)
        if self
            .config
            .validation
            .check_version_consistency
            .unwrap_or(true)
        {
            let version_validator = crate::validation::VersionConsistencyValidator::new(
                self.project_root.clone(),
                self.config.clone(),
            )?;
            let version_result = version_validator.validate().await?;
            violations.extend(version_result.violations);
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

        let grouped_violations = self.group_violations_by_type(violations);
        self.add_violation_sections(&mut report, grouped_violations);

        report
    }

    /// Group violations by their type
    fn group_violations_by_type<'a>(
        &self,
        violations: &'a [Violation],
    ) -> std::collections::HashMap<&'a ViolationType, Vec<&'a Violation>> {
        let mut by_type = std::collections::HashMap::new();
        for violation in violations {
            by_type
                .entry(&violation.violation_type)
                .or_insert_with(Vec::new)
                .push(violation);
        }
        by_type
    }

    /// Add violation sections to the report
    fn add_violation_sections(
        &self,
        report: &mut String,
        grouped_violations: std::collections::HashMap<&ViolationType, Vec<&Violation>>,
    ) {
        for (violation_type, violations) in grouped_violations {
            let type_name = format!("{:?}", violation_type)
                .to_uppercase()
                .replace('_', " ");

            report.push_str(&format!(
                "🚨 {} ({} violations):\n",
                type_name,
                violations.len()
            ));

            self.add_violation_details(report, &violations);
            report.push('\n');
        }
    }

    /// Add individual violation details to the report
    fn add_violation_details(&self, report: &mut String, violations: &[&Violation]) {
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
    }

    /// Run clippy with strict configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the clippy command fails to execute.
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

        let version_regex = Regex::new(r"rustc (\d+)\.(\d+)\.(\d+)")
            .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?;

        if let Some(captures) = version_regex.captures(&version_line) {
            let major: u32 = captures[1].parse().unwrap_or(0);
            let minor: u32 = captures[2].parse().unwrap_or(0);

            // Check against the configured minimum (parse required_rust_version)
            let min_minor = self.parse_required_minor();

            if major < 1 || (major == 1 && minor < min_minor) {
                violations.push(Violation {
                    violation_type: ViolationType::OldRustVersion,
                    file: PathBuf::from("<system>"),
                    line: 0,
                    message: format!(
                        "Rust version {}.{} is too old. Minimum required: {}",
                        major, minor, self.config.required_rust_version
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

    /// Parse the minor version number from the `required_rust_version` string
    fn parse_required_minor(&self) -> u32 {
        let parts: Vec<&str> = self.config.required_rust_version.split('.').collect();
        if parts.len() >= 2 {
            parts[1].parse().unwrap_or(82)
        } else {
            82 // fallback to 1.82
        }
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
        if path.to_string_lossy().contains("target/") {
            return Ok(());
        }

        if path.is_file() {
            if let Some(ext) = path.extension()
                && ext == "rs"
            {
                rust_files.push(path.to_path_buf());
            }
        } else if path.is_dir() {
            let entries = std::fs::read_dir(path)?;
            for entry in entries {
                let entry = entry?;
                let entry_path = entry.path();
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

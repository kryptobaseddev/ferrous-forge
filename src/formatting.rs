//! Code formatting and auto-correction module
#![allow(clippy::too_many_lines)]
//!
//! This module provides integration with rustfmt for code formatting
//! validation and automatic correction.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Formatting check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResult {
    /// Whether the code is properly formatted
    pub formatted: bool,
    /// Files that need formatting
    pub unformatted_files: Vec<String>,
    /// Suggested changes
    pub suggestions: Vec<FormatSuggestion>,
}

/// A formatting suggestion for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatSuggestion {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Description of the formatting issue
    pub description: String,
}

impl FormatResult {
    /// Generate a human-readable report
    pub fn report(&self) -> String {
        let mut report = String::new();

        if self.formatted {
            report.push_str("✅ Code formatting check passed - All files properly formatted!\n");
        } else {
            report.push_str(&format!(
                "⚠️ Code formatting issues found in {} files\n\n",
                self.unformatted_files.len()
            ));

            report.push_str("Files needing formatting:\n");
            for file in &self.unformatted_files {
                report.push_str(&format!("  • {}\n", file));
            }

            if !self.suggestions.is_empty() {
                report.push_str("\nFormatting suggestions:\n");
                for suggestion in &self.suggestions.iter().take(10).collect::<Vec<_>>() {
                    report.push_str(&format!(
                        "  {}:{} - {}\n",
                        suggestion.file, suggestion.line, suggestion.description
                    ));
                }

                if self.suggestions.len() > 10 {
                    report.push_str(&format!(
                        "  ... and {} more suggestions\n",
                        self.suggestions.len() - 10
                    ));
                }
            }

            report.push_str(
                "\n💡 Run 'ferrous-forge fix --format' to automatically fix these issues\n",
            );
        }

        report
    }
}

/// Check code formatting
pub async fn check_formatting(project_path: &Path) -> Result<FormatResult> {
    // Ensure rustfmt is installed
    ensure_rustfmt_installed().await?;

    // Run cargo fmt with check mode
    let output = Command::new("cargo")
        .args(&["fmt", "--", "--check", "--verbose"])
        .current_dir(project_path)
        .output()
        .map_err(|e| Error::process(format!("Failed to run cargo fmt: {}", e)))?;

    // Parse the output
    parse_format_output(&output.stdout, &output.stderr, output.status.success())
}

/// Auto-format code
pub async fn auto_format(project_path: &Path) -> Result<()> {
    // Ensure rustfmt is installed
    ensure_rustfmt_installed().await?;

    println!("🔧 Auto-formatting code...");

    // Run cargo fmt
    let output = Command::new("cargo")
        .arg("fmt")
        .current_dir(project_path)
        .output()
        .map_err(|e| Error::process(format!("Failed to run cargo fmt: {}", e)))?;

    if output.status.success() {
        println!("✨ Code formatted successfully!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::process(format!("Formatting failed: {}", stderr)))
    }
}

/// Check formatting for a specific file
pub async fn check_file_formatting(file_path: &Path) -> Result<bool> {
    // Ensure rustfmt is installed
    ensure_rustfmt_installed().await?;

    // Run rustfmt with check mode on single file
    let output = Command::new("rustfmt")
        .args(&[
            "--check",
            file_path
                .to_str()
                .ok_or_else(|| Error::process("Invalid file path"))?,
        ])
        .output()
        .map_err(|e| Error::process(format!("Failed to run rustfmt: {}", e)))?;

    Ok(output.status.success())
}

/// Format a specific file
pub async fn format_file(file_path: &Path) -> Result<()> {
    // Ensure rustfmt is installed
    ensure_rustfmt_installed().await?;

    // Run rustfmt on single file
    let output = Command::new("rustfmt")
        .arg(
            file_path
                .to_str()
                .ok_or_else(|| Error::process("Invalid file path"))?,
        )
        .output()
        .map_err(|e| Error::process(format!("Failed to run rustfmt: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::process(format!(
            "Failed to format {}: {}",
            file_path.display(),
            stderr
        )))
    }
}

/// Get formatting diff without applying changes
pub async fn get_format_diff(project_path: &Path) -> Result<String> {
    // Ensure rustfmt is installed
    ensure_rustfmt_installed().await?;

    // Run cargo fmt with diff output
    let output = Command::new("cargo")
        .args(&["fmt", "--", "--check", "--emit=stdout"])
        .current_dir(project_path)
        .output()
        .map_err(|e| Error::process(format!("Failed to run cargo fmt: {}", e)))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Ensure rustfmt is installed
async fn ensure_rustfmt_installed() -> Result<()> {
    let check = Command::new("rustfmt").arg("--version").output();

    if check
        .as_ref()
        .map_or(true, |output| !output.status.success())
    {
        println!("📦 Installing rustfmt...");

        let install = Command::new("rustup")
            .args(&["component", "add", "rustfmt"])
            .output()
            .map_err(|e| Error::process(format!("Failed to install rustfmt: {}", e)))?;

        if !install.status.success() {
            return Err(Error::process("Failed to install rustfmt"));
        }

        println!("✅ rustfmt installed successfully");
    }

    Ok(())
}

/// Parse formatting check output
fn parse_format_output(stdout: &[u8], stderr: &[u8], success: bool) -> Result<FormatResult> {
    if success {
        return Ok(FormatResult {
            formatted: true,
            unformatted_files: vec![],
            suggestions: vec![],
        });
    }

    let stderr_str = String::from_utf8_lossy(stderr);
    let stdout_str = String::from_utf8_lossy(stdout);

    let unformatted_files = parse_unformatted_files(&stderr_str);
    let suggestions = parse_formatting_suggestions(&stdout_str);

    Ok(FormatResult {
        formatted: false,
        unformatted_files,
        suggestions,
    })
}

/// Parse unformatted files from stderr
fn parse_unformatted_files(stderr_str: &str) -> Vec<String> {
    let mut unformatted_files = Vec::new();

    for line in stderr_str.lines() {
        if line.starts_with("Diff in") {
            if let Some(file) = line.strip_prefix("Diff in ") {
                let file = file.trim_end_matches(" at line 1:");
                let file = file.trim_end_matches(':');
                unformatted_files.push(file.to_string());
            }
        }
    }

    unformatted_files
}

/// Parse formatting suggestions from stdout
fn parse_formatting_suggestions(stdout_str: &str) -> Vec<FormatSuggestion> {
    let mut suggestions = Vec::new();

    for line in stdout_str.lines() {
        if line.starts_with("warning:") || line.contains("formatting") {
            if let Some(suggestion) = extract_format_suggestion(line) {
                suggestions.push(suggestion);
            }
        }
    }

    suggestions
}

/// Extract formatting suggestion from a line
fn extract_format_suggestion(line: &str) -> Option<FormatSuggestion> {
    let pos = line.find(".rs:")?;
    let start = line.rfind('/').unwrap_or(0);
    let file = &line[start..pos + 3];

    let line_num = if let Some(num_start) = line[pos + 3..].find(':') {
        line[pos + 4..pos + 3 + num_start].parse().unwrap_or(0)
    } else {
        0
    };

    Some(FormatSuggestion {
        file: file.to_string(),
        line: line_num,
        description: "Formatting required".to_string(),
    })
}

/// Apply formatting configuration
pub async fn apply_rustfmt_config(project_path: &Path) -> Result<()> {
    let rustfmt_toml = project_path.join("rustfmt.toml");

    if !rustfmt_toml.exists() {
        // Create default rustfmt.toml
        let config = r#"# Ferrous Forge rustfmt configuration
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Auto"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
format_strings = false
format_macro_matchers = false
format_macro_bodies = true
empty_item_single_line = true
struct_lit_single_line = true
fn_single_line = false
where_single_line = false
imports_indent = "Block"
imports_layout = "Mixed"
merge_derives = true
group_imports = "StdExternalCrate"
reorder_impl_items = false
spaces_around_ranges = false
trailing_semicolon = true
trailing_comma = "Vertical"
match_block_trailing_comma = false
blank_lines_upper_bound = 1
blank_lines_lower_bound = 0
"#;

        tokio::fs::write(&rustfmt_toml, config)
            .await
            .map_err(|e| Error::process(format!("Failed to create rustfmt.toml: {}", e)))?;

        println!("✅ Created rustfmt.toml with Ferrous Forge standards");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_result_formatted() {
        let result = FormatResult {
            formatted: true,
            unformatted_files: vec![],
            suggestions: vec![],
        };

        assert!(result.formatted);
        assert!(result.unformatted_files.is_empty());
        assert!(result.suggestions.is_empty());
    }

    #[test]
    fn test_format_result_unformatted() {
        let result = FormatResult {
            formatted: false,
            unformatted_files: vec!["src/main.rs".to_string()],
            suggestions: vec![FormatSuggestion {
                file: "src/main.rs".to_string(),
                line: 10,
                description: "Formatting required".to_string(),
            }],
        };

        assert!(!result.formatted);
        assert_eq!(result.unformatted_files.len(), 1);
        assert_eq!(result.suggestions.len(), 1);
    }
}

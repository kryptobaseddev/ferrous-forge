//! Documentation coverage checking module
//!
//! This module provides functionality to check documentation coverage
//! for Rust projects, ensuring all public APIs are properly documented.

use crate::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Documentation coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocCoverage {
    /// Total number of documentable items
    pub total_items: usize,
    /// Number of documented items
    pub documented_items: usize,
    /// Coverage percentage
    pub coverage_percent: f32,
    /// List of items missing documentation
    pub missing: Vec<String>,
}

impl DocCoverage {
    /// Check if coverage meets minimum threshold
    pub fn meets_threshold(&self, min_coverage: f32) -> bool {
        self.coverage_percent >= min_coverage
    }

    /// Generate a human-readable report
    pub fn report(&self) -> String {
        let mut report = String::new();

        if self.coverage_percent >= 100.0 {
            report.push_str("✅ Documentation coverage: 100% - All items documented!\n");
        } else if self.coverage_percent >= 80.0 {
            report.push_str(&format!(
                "📝 Documentation coverage: {:.1}% - Good coverage\n",
                self.coverage_percent
            ));
        } else {
            report.push_str(&format!(
                "⚠️ Documentation coverage: {:.1}% - Needs improvement\n",
                self.coverage_percent
            ));
        }

        if !self.missing.is_empty() {
            report.push_str("\nMissing documentation for:\n");
            for (i, item) in self.missing.iter().take(10).enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, item));
            }
            if self.missing.len() > 10 {
                report.push_str(&format!(
                    "  ... and {} more items\n",
                    self.missing.len() - 10
                ));
            }
        }

        report
    }
}

/// Check documentation coverage for a Rust project
pub async fn check_documentation_coverage(project_path: &Path) -> Result<DocCoverage> {
    let output = run_cargo_doc(project_path)?;
    let missing = find_missing_docs(&output)?;
    let (total, documented) = count_documentation_items(project_path).await?;

    let coverage_percent = calculate_coverage_percent(documented, total);

    Ok(DocCoverage {
        total_items: total,
        documented_items: documented,
        coverage_percent,
        missing,
    })
}

/// Run cargo doc and get output
fn run_cargo_doc(project_path: &Path) -> Result<std::process::Output> {
    Command::new("cargo")
        .args(&[
            "doc",
            "--no-deps",
            "--document-private-items",
            "--message-format=json",
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| Error::process(format!("Failed to run cargo doc: {}", e)))
}

/// Find missing documentation items from cargo doc output
fn find_missing_docs(output: &std::process::Output) -> Result<Vec<String>> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut missing = Vec::new();

    // Parse JSON messages for missing docs warnings
    for line in stdout.lines() {
        if line.contains("missing_docs") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(message) = json["message"]["rendered"].as_str() {
                    if let Some(item_match) = extract_item_name(message) {
                        missing.push(item_match);
                    }
                }
            }
        }
    }

    // Also check stderr for traditional warnings
    let warning_re = Regex::new(r"warning: missing documentation for (.+)")
        .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?;

    for cap in warning_re.captures_iter(&stderr) {
        missing.push(cap[1].to_string());
    }

    Ok(missing)
}

/// Calculate coverage percentage
fn calculate_coverage_percent(documented: usize, total: usize) -> f32 {
    if total > 0 {
        (documented as f32 / total as f32) * 100.0
    } else {
        100.0
    }
}

/// Count documentation items in the project
async fn count_documentation_items(project_path: &Path) -> Result<(usize, usize)> {
    let mut total = 0;
    let mut documented = 0;

    // Walk through all Rust files
    let walker = walkdir::WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("target")
        });

    for entry in walker {
        let content = tokio::fs::read_to_string(entry.path()).await?;
        let (file_total, file_documented) = count_items_in_file(&content)?;
        total += file_total;
        documented += file_documented;
    }

    Ok((total, documented))
}

/// Count documentation items in a single file
fn count_items_in_file(content: &str) -> Result<(usize, usize)> {
    let mut total = 0;
    let mut documented = 0;
    let lines: Vec<&str> = content.lines().collect();

    let pub_item_re = Regex::new(r"^\s*pub\s+(fn|struct|enum|trait|type|const|static|mod)\s+")
        .map_err(|e| Error::validation(format!("Failed to compile regex: {}", e)))?;

    for (i, line) in lines.iter().enumerate() {
        if pub_item_re.is_match(line) {
            total += 1;

            // Check if there's documentation above this line
            if i > 0
                && (lines[i - 1].trim().starts_with("///")
                    || lines[i - 1].trim().starts_with("//!"))
            {
                documented += 1;
            }
        }
    }

    Ok((total, documented))
}

/// Extract item name from error message
fn extract_item_name(message: &str) -> Option<String> {
    // Try to extract the item name from messages like:
    // "missing documentation for a struct"
    // "missing documentation for function `foo`"
    if let Some(start) = message.find('`') {
        if let Some(end) = message[start + 1..].find('`') {
            return Some(message[start + 1..start + 1 + end].to_string());
        }
    }

    // Fallback: extract type after "for"
    if let Some(pos) = message.find("for ") {
        Some(message[pos + 4..].trim().to_string())
    } else {
        None
    }
}

/// Suggest documentation for missing items
pub fn suggest_documentation(item_type: &str, item_name: &str) -> String {
    match item_type {
        "fn" | "function" => format!(
            "/// TODO: Document function `{}`.\n\
             ///\n\
             /// # Arguments\n\
             ///\n\
             /// # Returns\n\
             ///\n\
             /// # Examples\n\
             /// ```\n\
             /// // Example usage\n\
             /// ```",
            item_name
        ),
        "struct" => format!(
            "/// TODO: Document struct `{}`.\n\
             ///\n\
             /// # Fields\n\
             ///\n\
             /// # Examples\n\
             /// ```\n\
             /// // Example usage\n\
             /// ```",
            item_name
        ),
        "enum" => format!(
            "/// TODO: Document enum `{}`.\n\
             ///\n\
             /// # Variants\n\
             ///\n\
             /// # Examples\n\
             /// ```\n\
             /// // Example usage\n\
             /// ```",
            item_name
        ),
        _ => format!("/// TODO: Document `{}`.", item_name),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn test_count_items_in_file() {
        let content = r"
/// Documented function
pub fn documented() {}

pub fn undocumented() {}

/// Documented struct
pub struct DocStruct {}

pub struct UndocStruct {}
";
        let (total, documented) = count_items_in_file(content).unwrap();
        assert_eq!(total, 4);
        assert_eq!(documented, 2);
    }

    #[test]
    fn test_coverage_calculation() {
        let coverage = DocCoverage {
            total_items: 10,
            documented_items: 8,
            coverage_percent: 80.0,
            missing: vec![],
        };

        assert!(coverage.meets_threshold(75.0));
        assert!(!coverage.meets_threshold(85.0));
    }
}

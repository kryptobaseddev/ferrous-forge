//! Utility functions for AI report generation

use crate::{Result, validation::Violation};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Count violations by type
pub fn count_violations_by_type(violations: &[Violation]) -> HashMap<String, usize> {
    let mut violation_counts = HashMap::new();
    for violation in violations {
        *violation_counts
            .entry(format!("{:?}", violation.violation_type))
            .or_insert(0) += 1;
    }
    violation_counts
}

/// Get code snippet from a file at specific line
pub async fn get_code_snippet(file_path: &PathBuf, line: usize) -> Result<String> {
    if !file_path.exists() {
        return Ok("File not found".to_string());
    }

    let contents = fs::read_to_string(file_path).await?;
    let lines: Vec<&str> = contents.lines().collect();

    if line > 0 && line <= lines.len() {
        Ok(lines[line - 1].to_string())
    } else {
        Ok("Line not found".to_string())
    }
}

/// Count Rust files in a directory
pub async fn count_rust_files(project_path: &PathBuf) -> Result<usize> {
    let mut count = 0;
    let mut entries = fs::read_dir(project_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            count += 1;
        }
    }
    Ok(count)
}

/// Calculate compliance percentage
pub async fn calculate_compliance(project_path: &PathBuf, violations: &[Violation]) -> Result<f64> {
    let total_files = count_rust_files(project_path).await?;
    let files_with_violations = violations
        .iter()
        .map(|v| &v.file)
        .collect::<std::collections::HashSet<_>>()
        .len();

    Ok(if total_files > 0 && files_with_violations <= total_files {
        ((total_files - files_with_violations) as f64 / total_files as f64) * 100.0
    } else {
        0.0
    })
}

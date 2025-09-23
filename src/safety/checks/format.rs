//! Format checking with cargo fmt

use crate::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// Format check implementation
pub struct FormatCheck;

impl SafetyCheck for FormatCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "format"
    }

    fn description() -> &'static str {
        "Validates code formatting with rustfmt"
    }
}

/// Run cargo fmt --check
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Format);

    // Check if rustfmt is available
    if which::which("cargo").is_err() {
        result.add_error("cargo not found in PATH");
        result.add_suggestion("Install Rust and cargo from https://rustup.rs");
        result.set_duration(start.elapsed());
        return Ok(result);
    }

    // Run cargo fmt --check
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["fmt", "--check"])
        .output()?;

    result.set_duration(start.elapsed());

    if !output.status.success() {
        result.add_error("Code formatting violations found");
        result.add_suggestion("Run 'cargo fmt' to fix formatting automatically");

        // Parse formatting violations from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Look for diff output
        for line in stdout.lines().chain(stderr.lines()) {
            if line.starts_with("Diff in") {
                result.add_error(format!("Formatting issue: {}", line));
            } else if line.contains("rustfmt") && line.contains("failed") {
                result.add_error(line.to_string());
            }
        }

        // If no specific errors found, add general message
        if result.errors.len() == 1 {
            result.add_context("Run 'cargo fmt' to see detailed formatting issues");
        }
    } else {
        result.add_context("All code is properly formatted");
    }

    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_format_check_on_empty_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create a basic Cargo.toml
        let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)
            .await
            .unwrap();

        // Create src directory
        fs::create_dir_all(temp_dir.path().join("src"))
            .await
            .unwrap();

        // Create a properly formatted main.rs
        let main_rs = r#"fn main() {
    println!("Hello, world!");
}
"#;
        fs::write(temp_dir.path().join("src/main.rs"), main_rs)
            .await
            .unwrap();

        let result = run(temp_dir.path()).await.unwrap();

        // Should pass for properly formatted code
        assert!(result.passed);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_format_check_struct() {
        assert_eq!(FormatCheck::name(), "format");
        assert!(!FormatCheck::description().is_empty());
    }
}

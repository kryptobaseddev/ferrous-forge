//! Clippy checking with strict warnings

use crate::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// Clippy check implementation
pub struct ClippyCheck;

impl SafetyCheck for ClippyCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "clippy"
    }

    fn description() -> &'static str {
        "Runs clippy lints with strict warnings"
    }
}

/// Run cargo clippy with strict warnings
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Clippy);

    if !is_clippy_available() {
        result.add_error("clippy not available");
        result.add_suggestion("Install clippy with: rustup component add clippy");
        result.set_duration(start.elapsed());
        return Ok(result);
    }

    let output = run_clippy_command(project_path)?;
    result.set_duration(start.elapsed());

    if !output.status.success() {
        handle_clippy_errors(&output, &mut result);
    } else {
        result.add_context("All clippy lints passed");
    }

    Ok(result)
}

/// Check if clippy is available
fn is_clippy_available() -> bool {
    Command::new("cargo")
        .args(&["clippy", "--version"])
        .output()
        .map_or(false, |output| output.status.success())
}

/// Run clippy command with strict settings
fn run_clippy_command(project_path: &Path) -> Result<std::process::Output> {
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&[
            "clippy",
            "--all-targets", 
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .output()?;
    
    Ok(output)
}

/// Handle clippy errors and parse output
fn handle_clippy_errors(output: &std::process::Output, result: &mut CheckResult) {
    result.add_error("Clippy lints found");
    result.add_suggestion("Fix clippy warnings before proceeding");

    parse_clippy_output(&String::from_utf8_lossy(&output.stderr), result);
    
    // Add general suggestions
    result.add_suggestion("Run 'cargo clippy --fix' to auto-fix some issues");
    result.add_suggestion("Check https://rust-lang.github.io/rust-clippy/ for lint explanations");
}

/// Parse clippy output for specific issues
fn parse_clippy_output(stderr: &str, result: &mut CheckResult) {
    let mut error_count = 0;
    let mut in_error = false;

    for line in stderr.lines() {
        if (line.starts_with("error:") || line.starts_with("warning:")) && error_count < 5 {
            result.add_error(format!("Clippy: {}", line.trim()));
            error_count += 1;
            in_error = true;
        } else if in_error && line.trim().starts_with("-->") {
            // Add file location context
            result.add_context(format!("Location: {}", line.trim()));
            in_error = false;
        } else if line.contains("help:") && !line.trim().is_empty() {
            result.add_suggestion(
                line.trim()
                    .strip_prefix("help: ")
                    .unwrap_or(line.trim())
                    .to_string(),
            );
        }
    }

    if error_count >= 5 {
        result.add_error("... and more clippy issues (showing first 5)");
        result.add_suggestion("Fix the issues above first, then run again");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_clippy_check_on_clean_project() {
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

        // Create a clean main.rs
        let main_rs = r#"fn main() {
    println!("Hello, world!");
}
"#;
        fs::write(temp_dir.path().join("src/main.rs"), main_rs)
            .await
            .unwrap();

        let _result = run(temp_dir.path()).await.unwrap();

        // Should pass for clean code (assuming clippy is available)
        // Note: This might fail in CI if clippy isn't available
    }

    #[test]
    fn test_clippy_check_struct() {
        assert_eq!(ClippyCheck::name(), "clippy");
        assert!(!ClippyCheck::description().is_empty());
    }
}

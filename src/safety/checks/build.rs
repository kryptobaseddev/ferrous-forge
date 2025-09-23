//! Build checking with cargo build

use crate::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// Build check implementation
pub struct BuildCheck;

impl SafetyCheck for BuildCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "build"
    }

    fn description() -> &'static str {
        "Ensures project builds successfully in release mode"
    }
}

/// Run cargo build --release
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Build);

    // Run cargo build --release
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["build", "--release"])
        .output()?;

    result.set_duration(start.elapsed());

    if !output.status.success() {
        handle_build_failure(&mut result, &output);
    } else {
        handle_build_success(&mut result, &output);
    }

    Ok(result)
}

/// Handle build failure output
fn handle_build_failure(result: &mut CheckResult, output: &std::process::Output) {
    result.add_error("Build failed");
    result.add_suggestion("Fix compilation errors before proceeding");

    let error_count = parse_build_errors(result, &output.stderr);

    if error_count >= 3 {
        result.add_error("... and more build errors (showing first 3)");
    }

    result.add_suggestion("Run 'cargo build' to see detailed error messages");
    result.add_suggestion("Check for missing dependencies or syntax errors");
}

/// Parse build errors from stderr
fn parse_build_errors(result: &mut CheckResult, stderr: &[u8]) -> usize {
    let stderr = String::from_utf8_lossy(stderr);
    let mut error_count = 0;

    for line in stderr.lines() {
        if line.starts_with("error") && error_count < 3 {
            result.add_error(format!("Build: {}", line.trim()));
            error_count += 1;
        } else if line.trim().starts_with("-->") && error_count <= 3 {
            result.add_context(format!("Location: {}", line.trim()));
        }
    }

    error_count
}

/// Handle successful build output
fn handle_build_success(result: &mut CheckResult, output: &std::process::Output) {
    result.add_context("Project builds successfully in release mode");

    // Check for warnings
    let stderr = String::from_utf8_lossy(&output.stderr);
    let warning_count = stderr
        .lines()
        .filter(|line| line.starts_with("warning:"))
        .count();

    if warning_count > 0 {
        result.add_context(format!("Build completed with {} warnings", warning_count));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_build_check_on_valid_project() {
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

        // Create a valid main.rs
        let main_rs = r#"fn main() {
    println!("Hello, world!");
}
"#;
        fs::write(temp_dir.path().join("src/main.rs"), main_rs)
            .await
            .unwrap();

        let result = run(temp_dir.path()).await.unwrap();

        // Should pass for valid code
        assert!(result.passed);
    }

    #[test]
    fn test_build_check_struct() {
        assert_eq!(BuildCheck::name(), "build");
        assert!(!BuildCheck::description().is_empty());
    }
}

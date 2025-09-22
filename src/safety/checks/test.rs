//! Test execution checking

use crate::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// Test check implementation
pub struct TestCheck;

impl SafetyCheck for TestCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "test"
    }

    fn description() -> &'static str {
        "Runs the complete test suite"
    }
}

/// Run cargo test --all-targets --all-features
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Test);

    // Run cargo test with comprehensive flags
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["test", "--all-targets", "--all-features"])
        .output()?;

    result.set_duration(start.elapsed());

    if !output.status.success() {
        result.add_error("Tests failed");
        result.add_suggestion("Fix failing tests before proceeding");

        // Parse test failures
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut failure_count = 0;
        let mut in_failure = false;

        for line in stdout.lines().chain(stderr.lines()) {
            if line.starts_with("test ") && line.contains("FAILED") && failure_count < 5 {
                result.add_error(format!("Test failure: {}", line.trim()));
                failure_count += 1;
            } else if line.starts_with("---- ") && line.contains("stdout ----") {
                in_failure = true;
            } else if in_failure && !line.trim().is_empty() && failure_count <= 5 {
                result.add_context(format!("Test output: {}", line.trim()));
                in_failure = false;
            } else if line.contains("test result:") && line.contains("FAILED") {
                result.add_error(line.trim().to_string());
            }
        }

        if failure_count >= 5 {
            result.add_error("... and more test failures (showing first 5)");
        }

        result.add_suggestion("Run 'cargo test' to see detailed test output");
        result.add_suggestion("Check test logic and fix failing assertions");
    } else {
        // Parse successful test output
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("test result: ok.") {
                result.add_context(format!("Tests: {}", line.trim()));
                break;
            }
        }

        if result.context.is_empty() {
            result.add_context("All tests passed");
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_test_check_on_project_with_tests() {
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

        // Create a lib.rs with tests
        let lib_rs = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#;
        fs::write(temp_dir.path().join("src/lib.rs"), lib_rs)
            .await
            .unwrap();

        let result = run(temp_dir.path()).await.unwrap();

        // Should pass for working tests
        assert!(result.passed);
    }

    #[test]
    fn test_test_check_struct() {
        assert_eq!(TestCheck::name(), "test");
        assert!(!TestCheck::description().is_empty());
    }
}

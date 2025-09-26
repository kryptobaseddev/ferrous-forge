//! Testing operations for edition migration

use super::EditionMigrator;
use super::types::{MigrationResult, MigrationStep, TestResults};
use crate::Result;
use std::process::Command;

impl EditionMigrator {
    /// Run tests after migration
    pub(super) async fn run_tests(&self, result: &mut MigrationResult) -> Result<()> {
        let output = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--nocapture")
            .current_dir(&self.project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse test output
        let passed = stdout.matches("test result: ok").count();
        let failed = stdout.matches("FAILED").count();
        let ignored = stdout.matches("ignored").count();
        let test_results = TestResults {
            total: passed + failed + ignored,
            passed,
            failed,
            ignored,
            filtered_out: 0,
        };

        result.test_results = Some(test_results);

        if output.status.success() {
            result.steps_performed.push(MigrationStep {
                name: "Run tests".to_string(),
                description: "Executed test suite".to_string(),
                success: true,
                message: Some("All tests passed".to_string()),
            });
        } else {
            result.steps_performed.push(MigrationStep {
                name: "Run tests".to_string(),
                description: "Executed test suite".to_string(),
                success: false,
                message: Some(format!("Tests failed: {}", stderr)),
            });
        }

        Ok(())
    }

    /// Commit migration changes
    pub(super) async fn commit_changes(&self, result: &mut MigrationResult) -> Result<()> {
        let output = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&self.project_path)
            .output()?;

        if !output.status.success() {
            result.steps_performed.push(MigrationStep {
                name: "Commit changes".to_string(),
                description: "Git add failed".to_string(),
                success: false,
                message: Some("Failed to stage changes".to_string()),
            });
            return Ok(());
        }

        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("feat: migrate to new Rust edition")
            .current_dir(&self.project_path)
            .output()?;

        if output.status.success() {
            result.steps_performed.push(MigrationStep {
                name: "Commit changes".to_string(),
                description: "Committed migration changes".to_string(),
                success: true,
                message: Some("Changes committed successfully".to_string()),
            });
        } else {
            result.steps_performed.push(MigrationStep {
                name: "Commit changes".to_string(),
                description: "Git commit failed".to_string(),
                success: false,
                message: Some("Failed to commit changes".to_string()),
            });
        }

        Ok(())
    }
}

//! Safety pipeline CLI commands

pub mod hooks;
pub mod status;

use crate::Result;
use crate::safety::{PipelineStage, SafetyPipeline};
use console::style;
use std::path::Path;

/// Handle safety install command
///
/// # Errors
///
/// Returns an error if the project path is not a git repository, the hooks
/// directory cannot be created, or any hook file fails to write.
pub async fn handle_install(force: bool, project_path: &Path, install_cargo: bool) -> Result<()> {
    hooks::display_install_header();
    let hooks_dir = hooks::validate_git_repo_and_create_hooks_dir(project_path)?;

    hooks::install_pre_commit_hook(&hooks_dir, force)?;
    hooks::install_pre_push_hook(&hooks_dir, force)?;

    if install_cargo {
        hooks::install_cargo_interception(force)?;
    }

    hooks::display_install_success(install_cargo);

    Ok(())
}

/// Handle safety check command
///
/// # Errors
///
/// Returns an error if the stage string cannot be parsed, the safety pipeline
/// fails to initialize, or the checks fail to run.
pub async fn handle_check(stage_str: &str, project_path: &Path, verbose: bool) -> Result<()> {
    let stage = stage_str.parse::<PipelineStage>()?;

    println!("🧪 Testing Safety Pipeline - {}", stage.display_name());
    println!("{}", "=".repeat(50));

    let pipeline = SafetyPipeline::new(project_path).await?;
    let report = pipeline.run_checks(stage).await?;

    if verbose {
        report.print_detailed();
    } else {
        report.print_summary();
    }

    if !report.passed {
        println!(
            "\n{}",
            style("⚠️  This operation would be BLOCKED by the safety pipeline")
                .red()
                .bold()
        );
        println!("Fix the issues above before committing/pushing/publishing");
    } else {
        println!(
            "\n{}",
            style("✅ This operation would be ALLOWED by the safety pipeline")
                .green()
                .bold()
        );
    }

    Ok(())
}

/// Handle safety status command
///
/// # Errors
///
/// Returns an error if the safety configuration cannot be loaded.
pub async fn handle_status() -> Result<()> {
    status::display_status_header();
    status::display_safety_configuration().await;
    status::display_git_hooks_status();
    Ok(())
}

/// Test individual safety checks
///
/// # Errors
///
/// Returns an error if any safety check fails to execute.
pub async fn test_individual_checks(project_path: &Path) -> Result<()> {
    crate::safety::checks::test_runner::test_safety_checks(project_path).await
}

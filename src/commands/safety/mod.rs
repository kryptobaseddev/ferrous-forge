//! Safety pipeline CLI commands
//!
//! @task T017
//! @epic T014

/// Bypass command implementation for emergency safety check bypassing.
pub mod bypass_cmd;
/// Safety configuration command implementation.
pub mod config_cmd;
/// Git hook installation and management for safety checks.
pub mod hooks;
/// Safety report command implementation.
pub mod report_cmd;
/// Safety stats command implementation.
pub mod stats_cmd;
/// Safety status display command implementation.
pub mod status;

use crate::Result;
use crate::commands::SafetyBypassStage;
use crate::safety::{PipelineStage, SafetyPipeline, bypass::BypassManager, config::BypassConfig};
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

/// Handle check-bypass command (used by git hooks)
///
/// Returns silently if no bypass is active, or prints "active" and exits 0 if bypass exists.
///
/// # Errors
///
/// Returns an error if the bypass check fails to execute.
pub async fn handle_check_bypass(stage: SafetyBypassStage) -> Result<()> {
    let pipeline_stage = stage.to_pipeline_stage();

    // Load bypass configuration
    let config = BypassConfig::load_or_default().await?;

    if !config.enabled {
        // Bypass system is disabled, so no bypass possible
        std::process::exit(1);
    }

    let manager = BypassManager::new(&config)?;

    match manager.check_active_bypass(pipeline_stage).await? {
        Some(_) => {
            // Bypass is active
            println!("active");
            Ok(())
        }
        None => {
            // No active bypass
            std::process::exit(1);
        }
    }
}

/// Handle uninstall command
///
/// # Errors
///
/// Returns an error if hooks cannot be removed.
pub async fn handle_uninstall(project_path: &Path, confirm: bool) -> Result<()> {
    if !confirm {
        println!("⚠️  This will remove all Ferrous Forge safety hooks.");
        println!("   Run with --confirm to proceed.");
        return Ok(());
    }

    hooks::uninstall_hooks(project_path)
}

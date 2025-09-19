//! Safety pipeline CLI commands

use crate::safety::{PipelineStage, SafetyPipeline};
use crate::Result;
use console::style;
use std::path::Path;

/// Handle safety check command
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
pub async fn handle_status() -> Result<()> {
    println!("🛡️  Ferrous Forge Safety Pipeline Status");
    println!("{}", "=".repeat(40));

    // Check if safety is configured
    match crate::safety::SafetyConfig::load().await {
        Ok(config) => {
            println!("✅ Safety configuration: Found");
            println!("   Enabled: {}", if config.enabled { "Yes" } else { "No" });
            println!(
                "   Strict mode: {}",
                if config.strict_mode { "Yes" } else { "No" }
            );

            println!("\n📋 Stage Configuration:");
            println!(
                "   Pre-commit: {} (timeout: {}s)",
                if config.pre_commit.enabled {
                    "✅ Enabled"
                } else {
                    "❌ Disabled"
                },
                config.pre_commit.timeout_seconds
            );
            println!(
                "   Pre-push: {} (timeout: {}s)",
                if config.pre_push.enabled {
                    "✅ Enabled"
                } else {
                    "❌ Disabled"
                },
                config.pre_push.timeout_seconds
            );
            println!(
                "   Publish: {} (timeout: {}s)",
                if config.publish.enabled {
                    "✅ Enabled"
                } else {
                    "❌ Disabled"
                },
                config.publish.timeout_seconds
            );

            println!("\n🚫 Bypass System:");
            println!(
                "   Enabled: {}",
                if config.bypass.enabled { "Yes" } else { "No" }
            );
            if config.bypass.enabled {
                println!("   Max per day: {}", config.bypass.max_bypasses_per_day);
                println!("   Requires reason: {}", config.bypass.require_reason);
            }
        }
        Err(_) => {
            println!("❌ Safety configuration: Not found");
            println!("   Run 'ferrous-forge safety install' to set up safety pipeline");
        }
    }

    // Check if git hooks are installed
    let git_dir = Path::new(".git");
    if git_dir.exists() {
        println!("\n🪝 Git Hooks:");

        let pre_commit_hook = git_dir.join("hooks/pre-commit");
        let pre_push_hook = git_dir.join("hooks/pre-push");

        println!(
            "   Pre-commit: {}",
            if pre_commit_hook.exists() {
                "✅ Installed"
            } else {
                "❌ Not installed"
            }
        );
        println!(
            "   Pre-push: {}",
            if pre_push_hook.exists() {
                "✅ Installed"
            } else {
                "❌ Not installed"
            }
        );
    } else {
        println!("\n🪝 Git Hooks: Not a git repository");
    }

    Ok(())
}

/// Test individual safety checks
pub async fn test_individual_checks(project_path: &Path) -> Result<()> {
    crate::safety::checks::test_runner::test_safety_checks(project_path).await
}

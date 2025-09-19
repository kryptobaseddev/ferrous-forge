//! Safety pipeline CLI commands

use crate::safety::{PipelineStage, SafetyPipeline};
use crate::Result;
use console::style;
use std::path::Path;
use std::fs;

/// Handle safety install command
pub async fn handle_install(force: bool, project_path: &Path) -> Result<()> {
    println!("🔧 Installing Ferrous Forge Safety Pipeline Git Hooks");
    println!("{}", "=".repeat(50));

    // Check if it's a git repository
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        return Err(crate::error::Error::Config(
            "Not a git repository. Run 'git init' first.".to_string(),
        ));
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    // Pre-commit hook
    let pre_commit_path = hooks_dir.join("pre-commit");
    let pre_commit_exists = pre_commit_path.exists();
    
    if pre_commit_exists && !force {
        println!("⚠️  Pre-commit hook already exists. Use --force to overwrite.");
    } else {
        let pre_commit_content = r#"#!/bin/bash
# Ferrous Forge Safety Pipeline - Pre-Commit Hook

echo "🦀 Running Ferrous Forge safety checks..."
ferrous-forge safety check --stage pre-commit

if [ $? -ne 0 ]; then
    echo "❌ Safety checks failed. Please fix the issues before committing."
    exit 1
fi

echo "✅ All safety checks passed!"
exit 0
"#;
        fs::write(&pre_commit_path, pre_commit_content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&pre_commit_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&pre_commit_path, perms)?;
        }
        println!("✅ Installed pre-commit hook");
    }

    // Pre-push hook
    let pre_push_path = hooks_dir.join("pre-push");
    let pre_push_exists = pre_push_path.exists();
    
    if pre_push_exists && !force {
        println!("⚠️  Pre-push hook already exists. Use --force to overwrite.");
    } else {
        let pre_push_content = r#"#!/bin/bash
# Ferrous Forge Safety Pipeline - Pre-Push Hook

echo "🦀 Running Ferrous Forge safety checks..."
ferrous-forge safety check --stage pre-push

if [ $? -ne 0 ]; then
    echo "❌ Safety checks failed. Please fix the issues before pushing."
    exit 1
fi

echo "✅ All safety checks passed!"
exit 0
"#;
        fs::write(&pre_push_path, pre_push_content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&pre_push_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&pre_push_path, perms)?;
        }
        println!("✅ Installed pre-push hook");
    }

    println!("\n🎉 Safety pipeline git hooks installed successfully!");
    println!("\n📝 Next steps:");
    println!("   1. The pre-commit hook will run before each commit");
    println!("   2. The pre-push hook will run before each push");
    println!("   3. To bypass temporarily: git commit --no-verify");
    println!("   4. To uninstall: Remove .git/hooks/pre-commit and pre-push");

    Ok(())
}

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

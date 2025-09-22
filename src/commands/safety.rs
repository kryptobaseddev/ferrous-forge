//! Safety pipeline CLI commands

use crate::safety::{PipelineStage, SafetyPipeline};
use crate::Result;
use console::style;
use std::fs;
use std::path::Path;

/// Handle safety install command
pub async fn handle_install(force: bool, project_path: &Path) -> Result<()> {
    display_install_header();
    let hooks_dir = validate_git_repo_and_create_hooks_dir(project_path)?;
    
    install_pre_commit_hook(&hooks_dir, force)?;
    install_pre_push_hook(&hooks_dir, force)?;
    display_install_success();

    Ok(())
}

/// Display installation header
fn display_install_header() {
    println!("🔧 Installing Ferrous Forge Safety Pipeline Git Hooks");
    println!("{}", "=".repeat(50));
}

/// Validate git repository and create hooks directory
fn validate_git_repo_and_create_hooks_dir(project_path: &Path) -> Result<std::path::PathBuf> {
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        return Err(crate::error::Error::Config(
            "Not a git repository. Run 'git init' first.".to_string(),
        ));
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;
    Ok(hooks_dir)
}

/// Install pre-commit hook
fn install_pre_commit_hook(hooks_dir: &Path, force: bool) -> Result<()> {
    let pre_commit_path = hooks_dir.join("pre-commit");
    
    if pre_commit_path.exists() && !force {
        println!("⚠️  Pre-commit hook already exists. Use --force to overwrite.");
        return Ok(());
    }

    let content = get_pre_commit_hook_content();
    install_hook(&pre_commit_path, &content)?;
    println!("✅ Installed pre-commit hook");
    Ok(())
}

/// Install pre-push hook
fn install_pre_push_hook(hooks_dir: &Path, force: bool) -> Result<()> {
    let pre_push_path = hooks_dir.join("pre-push");
    
    if pre_push_path.exists() && !force {
        println!("⚠️  Pre-push hook already exists. Use --force to overwrite.");
        return Ok(());
    }

    let content = get_pre_push_hook_content();
    install_hook(&pre_push_path, &content)?;
    println!("✅ Installed pre-push hook");
    Ok(())
}

/// Get pre-commit hook content
fn get_pre_commit_hook_content() -> &'static str {
    r#"#!/bin/bash
# Ferrous Forge Safety Pipeline - Pre-Commit Hook

echo "🦀 Running Ferrous Forge safety checks..."
ferrous-forge safety check --stage pre-commit

if [ $? -ne 0 ]; then
    echo "❌ Safety checks failed. Please fix the issues before committing."
    exit 1
fi

echo "✅ All safety checks passed!"
exit 0
"#
}

/// Get pre-push hook content
fn get_pre_push_hook_content() -> &'static str {
    r#"#!/bin/bash
# Ferrous Forge Safety Pipeline - Pre-Push Hook

echo "🦀 Running Ferrous Forge safety checks..."
ferrous-forge safety check --stage pre-push

if [ $? -ne 0 ]; then
    echo "❌ Safety checks failed. Please fix the issues before pushing."
    exit 1
fi

echo "✅ All safety checks passed!"
exit 0
"#
}

/// Install a hook file with proper permissions
fn install_hook(hook_path: &Path, content: &str) -> Result<()> {
    fs::write(hook_path, content)?;
    set_executable_permissions(hook_path)?;
    Ok(())
}

/// Set executable permissions on Unix systems
fn set_executable_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Display installation success message
fn display_install_success() {
    println!("\n🎉 Safety pipeline git hooks installed successfully!");
    println!("\n📝 Next steps:");
    println!("   1. The pre-commit hook will run before each commit");
    println!("   2. The pre-push hook will run before each push");
    println!("   3. To bypass temporarily: git commit --no-verify");
    println!("   4. To uninstall: Remove .git/hooks/pre-commit and pre-push");
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
    display_status_header();
    display_safety_configuration().await;
    display_git_hooks_status();
    Ok(())
}

/// Display status header
fn display_status_header() {
    println!("🛡️  Ferrous Forge Safety Pipeline Status");
    println!("{}", "=".repeat(40));
}

/// Display safety configuration status
async fn display_safety_configuration() {
    match crate::safety::SafetyConfig::load().await {
        Ok(config) => {
            display_safety_config_found(&config);
        }
        Err(_) => {
            display_safety_config_not_found();
        }
    }
}

/// Display safety configuration when found
fn display_safety_config_found(config: &crate::safety::SafetyConfig) {
    println!("✅ Safety configuration: Found");
    println!("   Enabled: {}", if config.enabled { "Yes" } else { "No" });
    println!(
        "   Strict mode: {}",
        if config.strict_mode { "Yes" } else { "No" }
    );

    display_stage_configuration(config);
    display_bypass_configuration(config);
}

/// Display stage configuration details
fn display_stage_configuration(config: &crate::safety::SafetyConfig) {
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
}

/// Display bypass system configuration
fn display_bypass_configuration(config: &crate::safety::SafetyConfig) {
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

/// Display message when safety configuration is not found
fn display_safety_config_not_found() {
    println!("❌ Safety configuration: Not found");
    println!("   Run 'ferrous-forge safety install' to set up safety pipeline");
}

/// Display git hooks installation status
fn display_git_hooks_status() {
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
}

/// Test individual safety checks
pub async fn test_individual_checks(project_path: &Path) -> Result<()> {
    crate::safety::checks::test_runner::test_safety_checks(project_path).await
}

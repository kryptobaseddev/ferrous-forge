//! Safety status display functions
//!
//! @task T017
//! @epic T014

use std::path::Path;

/// Display status header
pub fn display_status_header() {
    println!("🛡️  Ferrous Forge Safety Pipeline Status");
    println!("{}", "=".repeat(40));
}

/// Display safety configuration status
pub async fn display_safety_configuration() {
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
    println!(
        "   Blocking: {}",
        if config.strict_mode {
            "🛡️  MANDATORY (blocks on failure)"
        } else {
            "⚠️  Warning only"
        }
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
        println!("\n   To bypass: ferrous-forge safety bypass --stage=STAGE --reason=\"...\"");
    } else {
        println!("\n   ⚠️  Bypass is disabled - no way to skip safety checks!");
    }
}

/// Display message when safety configuration is not found
fn display_safety_config_not_found() {
    println!("❌ Safety configuration: Not found");
    println!("   Run 'ferrous-forge safety install' to set up safety pipeline");
}

/// Display git hooks installation status
pub fn display_git_hooks_status() {
    let git_dir = Path::new(".git");
    if git_dir.exists() {
        println!("\n🪝 Git Hooks:");

        let hooks_dir = git_dir.join("hooks");
        let pre_commit_hook = hooks_dir.join("pre-commit");
        let pre_push_hook = hooks_dir.join("pre-push");

        let pre_commit_status = if pre_commit_hook.exists() {
            if let Ok(content) = std::fs::read_to_string(&pre_commit_hook) {
                if content.contains("🛡️  FERROUS FORGE BLOCKED") {
                    "✅ Installed (blocking)"
                } else if content.contains("Ferrous Forge") {
                    "⚠️  Installed (non-blocking)"
                } else {
                    "⚠️  Installed (third-party)"
                }
            } else {
                "⚠️  Installed (cannot read)"
            }
        } else {
            "❌ Not installed"
        };

        let pre_push_status = if pre_push_hook.exists() {
            if let Ok(content) = std::fs::read_to_string(&pre_push_hook) {
                if content.contains("🛡️  FERROUS FORGE BLOCKED") {
                    "✅ Installed (blocking)"
                } else if content.contains("Ferrous Forge") {
                    "⚠️  Installed (non-blocking)"
                } else {
                    "⚠️  Installed (third-party)"
                }
            } else {
                "⚠️  Installed (cannot read)"
            }
        } else {
            "❌ Not installed"
        };

        println!("   Pre-commit: {}", pre_commit_status);
        println!("   Pre-push: {}", pre_push_status);

        // Show summary
        if pre_commit_status.contains("blocking") && pre_push_status.contains("blocking") {
            println!("\n🛡️  Mandatory blocking is ACTIVE");
            println!("   Commits and pushes will be BLOCKED if checks fail");
        } else if pre_commit_hook.exists() || pre_push_hook.exists() {
            println!("\n⚠️  Partial installation detected");
            println!("   Run: ferrous-forge safety install --force");
        } else {
            println!("\n⚠️  Safety hooks are NOT installed");
            println!("   Run: ferrous-forge safety install");
        }
    } else {
        println!("\n🪝 Git Hooks: Not a git repository");
    }
}

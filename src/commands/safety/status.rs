//! Safety status display functions

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
pub fn display_git_hooks_status() {
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

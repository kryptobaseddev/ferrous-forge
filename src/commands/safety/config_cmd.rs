//! Safety configuration command implementation
//!
//! Provides commands to view and modify safety pipeline configuration.
//!
//! @task T019
//! @epic T014

use crate::{
    Result,
    safety::config::{SafetyConfig, StageConfig},
};
use console::style;

/// Handle config show command
///
/// Displays the current safety configuration in a user-friendly format.
///
/// # Errors
///
/// Returns an error if the safety configuration cannot be loaded.
pub async fn handle_config_show() -> Result<()> {
    let config = SafetyConfig::load_or_default().await?;

    println!("{}", style("🛡️  Safety Pipeline Configuration").bold());
    println!("{}", "=".repeat(50));
    println!();

    // Global settings
    println!("{}", style("Global Settings:").cyan().bold());
    println!("  Enabled:          {}", format_bool(config.enabled));
    println!("  Strict Mode:      {}", format_bool(config.strict_mode));
    println!("  Show Progress:    {}", format_bool(config.show_progress));
    println!(
        "  Parallel Checks:  {}",
        format_bool(config.parallel_checks)
    );
    println!();

    // Stage configurations
    println!("{}", style("Stage Configurations:").cyan().bold());
    print_stage_config("Pre-Commit", &config.pre_commit);
    print_stage_config("Pre-Push", &config.pre_push);
    print_stage_config("Publish", &config.publish);

    // Bypass configuration
    println!("{}", style("Bypass Settings:").cyan().bold());
    println!(
        "  Enabled:              {}",
        format_bool(config.bypass.enabled)
    );
    println!(
        "  Require Reason:       {}",
        format_bool(config.bypass.require_reason)
    );
    println!(
        "  Require Confirmation: {}",
        format_bool(config.bypass.require_confirmation)
    );
    println!(
        "  Log Bypasses:         {}",
        format_bool(config.bypass.log_bypasses)
    );
    println!(
        "  Max Bypasses/Day:     {}",
        style(config.bypass.max_bypasses_per_day).yellow()
    );
    println!();

    // Configuration file location
    let config_path = SafetyConfig::config_file_path()?;
    println!(
        "{} {}",
        style("Config File:").dim(),
        style(config_path.display()).dim()
    );

    Ok(())
}

/// Handle config set command
///
/// Sets a safety configuration value.
///
/// # Errors
///
/// Returns an error if the key is unknown, the value cannot be parsed,
/// or the configuration cannot be saved.
pub async fn handle_config_set(key: String, value: String) -> Result<()> {
    let mut config = SafetyConfig::load_or_default().await?;

    println!(
        "{} Setting {} = {}",
        style("⚙️").cyan(),
        style(&key).yellow(),
        style(&value).green()
    );

    // Validate and set the value
    config.set(&key, &value)?;

    // Save the configuration
    config.save().await?;

    println!("{} Configuration updated successfully", style("✅").green());

    // Show the new value
    if let Some(display_value) = config.get(&key) {
        println!("   New value: {}", style(display_value).green());
    }

    Ok(())
}

/// Handle config get command
///
/// Gets a specific configuration value.
///
/// # Errors
///
/// Returns an error if the configuration cannot be loaded.
pub async fn handle_config_get(key: String) -> Result<()> {
    let config = SafetyConfig::load_or_default().await?;

    match config.get(&key) {
        Some(value) => {
            println!(
                "{} {} = {}",
                style("🔧").cyan(),
                style(&key).yellow(),
                style(&value).green()
            );
        }
        None => {
            println!(
                "{} Unknown configuration key: {}",
                style("❌").red(),
                style(&key).yellow()
            );
            println!();
            println!("Available keys:");
            print_available_keys();
        }
    }

    Ok(())
}

/// Print stage configuration
fn print_stage_config(name: &str, config: &StageConfig) {
    let status = if config.enabled {
        style("enabled").green()
    } else {
        style("disabled").red()
    };

    println!("  {}:", style(name).yellow());
    println!("    Status:   {}", status);
    println!(
        "    Timeout:  {} seconds",
        style(config.timeout_seconds).yellow()
    );
    println!(
        "    Checks:   {}",
        config
            .checks
            .iter()
            .map(|c| c.display_name())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
}

/// Format boolean for display
fn format_bool(value: bool) -> console::StyledObject<&'static str> {
    if value {
        style("Yes").green()
    } else {
        style("No").red()
    }
}

/// Print available configuration keys
fn print_available_keys() {
    let keys = [
        "enabled",
        "strict_mode",
        "show_progress",
        "parallel_checks",
        "pre_commit.enabled",
        "pre_commit.timeout_seconds",
        "pre_push.enabled",
        "pre_push.timeout_seconds",
        "publish.enabled",
        "publish.timeout_seconds",
        "bypass.enabled",
    ];

    for key in &keys {
        println!("  • {}", style(key).cyan());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bool() {
        let enabled = format_bool(true);
        assert_eq!(enabled.to_string(), "Yes");

        let disabled = format_bool(false);
        assert_eq!(disabled.to_string(), "No");
    }
}

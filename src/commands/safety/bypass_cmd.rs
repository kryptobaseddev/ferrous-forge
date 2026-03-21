//! Safety bypass command implementation
//!
//! @task T016
//! @epic T014

use crate::{
    Result,
    commands::SafetyBypassStage,
    safety::{PipelineStage, bypass::BypassManager, config::BypassConfig},
};
use console::style;

/// Handle the bypass command
///
/// Creates an emergency bypass for safety checks with audit logging.
///
/// # Errors
///
/// Returns an error if the bypass system is disabled, the reason is empty,
/// the daily bypass limit is reached, or the bypass cannot be saved.
pub async fn handle_bypass(
    stage: SafetyBypassStage,
    reason: String,
    duration: u64,
    user: Option<String>,
) -> Result<()> {
    let pipeline_stage = stage.to_pipeline_stage();
    let user = user.unwrap_or_else(whoami::username);

    print_bypass_header(&pipeline_stage, &reason, &user, duration);

    // Load bypass configuration
    let config = load_bypass_config().await?;

    if !config.enabled {
        println!("{}", style("❌ Bypass system is disabled").red().bold());
        return Err(crate::Error::safety(
            "Bypass system is disabled. Contact your administrator.",
        ));
    }

    // Create bypass manager and bypass
    let manager = BypassManager::new(&config)?;
    let bypass = manager
        .create_bypass(pipeline_stage, reason.clone(), user.clone(), duration)
        .await?;

    // Print success message
    print_bypass_success(&bypass, duration);

    // Show audit warning
    println!(
        "\n{}",
        style("⚠️  This bypass has been audit logged").yellow()
    );
    println!("   All team members will be notified of this bypass.");

    Ok(())
}

/// Handle the audit log command
///
/// Displays the bypass audit log entries.
///
/// # Errors
///
/// Returns an error if the audit log cannot be read.
pub async fn handle_audit(limit: usize) -> Result<()> {
    println!("{}", style("📋 Safety Bypass Audit Log").bold());
    println!("{}", "=".repeat(60));

    let config = load_bypass_config().await?;
    let manager = BypassManager::new(&config)?;
    let entries = manager.get_audit_log(limit).await?;

    if entries.is_empty() {
        println!("\n{}", style("No bypass entries found").dim());
        return Ok(());
    }

    println!("\nShowing last {} entries:\n", entries.len());

    for (i, entry) in entries.iter().enumerate() {
        let status = if entry.successful {
            style("✓ SUCCESS").green()
        } else {
            style("✗ FAILED").red()
        };

        println!("{}. {}", i + 1, status);
        println!("   Stage:    {}", style(entry.stage.display_name()).cyan());
        println!("   User:     {}", style(&entry.user).yellow());
        println!(
            "   Time:     {}",
            style(entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC")).dim()
        );
        println!("   Reason:   {}", entry.reason);
        println!();
    }

    println!("{}", style("End of audit log").dim());

    Ok(())
}

/// Load bypass configuration
async fn load_bypass_config() -> Result<BypassConfig> {
    // Try to load from safety config file, use defaults if not found
    let safety_config = crate::safety::config::SafetyConfig::load_or_default().await?;
    Ok(safety_config.bypass)
}

/// Print bypass creation header
fn print_bypass_header(stage: &PipelineStage, reason: &str, user: &str, duration: u64) {
    println!();
    println!("{}", style("🚨 EMERGENCY BYPASS REQUEST").red().bold());
    println!("{}", "=".repeat(60));
    println!();
    println!("   Stage:    {}", style(stage.display_name()).cyan());
    println!("   User:     {}", style(user).yellow());
    println!("   Duration: {} hours", style(duration).yellow());
    println!("   Reason:   {}", style(reason).yellow());
    println!();
    println!(
        "{}",
        style("⚠️  WARNING: This will skip all safety checks!")
            .red()
            .bold()
    );
    println!();
}

/// Print bypass success message
fn print_bypass_success(bypass: &crate::safety::bypass::ActiveBypass, duration: u64) {
    println!();
    println!("{}", style("✅ Bypass created successfully").green().bold());
    println!();
    println!(
        "   Bypass ID:   {}",
        style(format!("{:?}", bypass.created_at.timestamp())).dim()
    );
    println!(
        "   Expires:     {}",
        style(bypass.expires_at.format("%Y-%m-%d %H:%M:%S UTC")).yellow()
    );
    println!();
    println!("{}", style("Next steps:").bold());
    println!(
        "   1. The safety checks for {} are now bypassed",
        style(bypass.stage.display_name()).cyan()
    );
    println!(
        "   2. You can now run: {}",
        style(get_command_for_stage(bypass.stage)).green()
    );
    println!(
        "   3. Bypass will expire in {} hours",
        style(duration).yellow()
    );
}

/// Get the command to run for a given stage
fn get_command_for_stage(stage: PipelineStage) -> &'static str {
    match stage {
        PipelineStage::PreCommit => "git commit",
        PipelineStage::PrePush => "git push",
        PipelineStage::Publish => "cargo publish",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_bypass_stage_conversion() {
        assert_eq!(
            SafetyBypassStage::PreCommit.to_pipeline_stage(),
            PipelineStage::PreCommit
        );
        assert_eq!(
            SafetyBypassStage::PrePush.to_pipeline_stage(),
            PipelineStage::PrePush
        );
        assert_eq!(
            SafetyBypassStage::Publish.to_pipeline_stage(),
            PipelineStage::Publish
        );
    }

    #[test]
    fn test_get_command_for_stage() {
        assert_eq!(
            get_command_for_stage(PipelineStage::PreCommit),
            "git commit"
        );
        assert_eq!(get_command_for_stage(PipelineStage::PrePush), "git push");
        assert_eq!(
            get_command_for_stage(PipelineStage::Publish),
            "cargo publish"
        );
    }
}

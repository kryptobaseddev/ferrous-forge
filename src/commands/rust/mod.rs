//! Rust version management commands
//!
//! Provides commands for checking, updating, and managing Rust toolchains
//! via rustup integration. Includes version enforcement based on locked config.
//!
//! @task T020
//! @epic T014

/// Terminal output formatting for version information.
pub mod display;
/// Helper utilities for Rust version commands.
pub mod utils;

use crate::rust_version::{
    VersionManager,
    rustup::{RustupManager, ToolchainChannel},
};
use crate::{Error, Result};
use console::style;
use display::{
    display_recommendation, display_recommendation_details, display_recommendation_header,
    display_releases_list, display_version_status,
};
use utils::{create_spinner, fetch_latest_version};

/// Handle rust version check command
///
/// Checks current Rust version and optionally enforces locked requirements.
///
/// # Errors
///
/// Returns an error if the version manager fails to initialize, the current
/// version cannot be determined, or version requirements are not met (when enforce is true).
pub async fn handle_check(verbose: bool, enforce: bool) -> Result<()> {
    let spinner = create_spinner("Checking Rust version...");
    let manager = VersionManager::new()?;
    let rustup = RustupManager::new();

    let current = manager.check_current().await?;
    spinner.set_message("Fetching latest release information...");

    let latest = match fetch_latest_version(&manager, &spinner, &current).await {
        Some(release) => release,
        None => return Ok(()), // Error already handled in fetch_latest_version.clone()
    };

    // Check locked version requirements if enforce is enabled
    let version_check = if enforce || rustup.is_available() {
        spinner.set_message("Checking version requirements...");
        Some(rustup.check_version_requirements().await?)
    } else {
        None
    };

    spinner.finish_and_clear();

    display_version_status(&current, &latest, verbose);

    // Display version requirement status
    if let Some(check) = version_check {
        println!("{}", check.status_message());

        if enforce && !check.meets_requirements {
            println!();
            println!(
                "{}",
                style("🚫 Version enforcement enabled. Operations blocked.")
                    .red()
                    .bold()
            );
            println!(
                "{}",
                style("   Update Rust or adjust locked version requirements.").dim()
            );
            return Err(Error::validation(format!(
                "Current Rust version {} does not meet locked requirements ({})",
                check.current,
                check.requirements.description()
            )));
        }
    }

    let recommendation = manager.get_recommendation().await?;
    display_recommendation(&recommendation);

    Ok(())
}

/// Handle rust recommendation command
///
/// # Errors
///
/// Returns an error if the version manager fails to initialize or the
/// recommendation cannot be fetched.
pub async fn handle_recommend(stable_only: bool) -> Result<()> {
    let manager = VersionManager::new()?;
    let current = manager.check_current().await?;

    display_recommendation_header(&current);

    let recommendation = manager.get_recommendation().await?;
    display_recommendation_details(&recommendation, stable_only);

    Ok(())
}

/// Handle rust list command
///
/// Lists either recent releases or installed toolchains.
///
/// # Errors
///
/// Returns an error if the version manager fails to initialize or the
/// release list cannot be fetched, or if listing toolchains fails.
pub async fn handle_list(count: usize, toolchains: bool) -> Result<()> {
    if toolchains {
        handle_list_toolchains().await
    } else {
        handle_list_releases(count).await
    }
}

/// List recent Rust releases from GitHub
async fn handle_list_releases(count: usize) -> Result<()> {
    let manager = VersionManager::new()?;
    let releases = manager.get_recent_releases(count).await?;

    display_releases_list(&releases);

    Ok(())
}

/// List installed toolchains
async fn handle_list_toolchains() -> Result<()> {
    let rustup = RustupManager::new();

    if !rustup.is_available() {
        println!("{}", style("⚠️  rustup not found").yellow());
        println!("   Install rustup from https://rustup.rs to manage toolchains");
        return Ok(());
    }

    let toolchains = rustup.list_toolchains()?;

    println!();
    println!("{}", style("🔧 Installed Toolchains").bold().cyan());
    println!();

    if toolchains.is_empty() {
        println!("   No toolchains installed.");
        println!("   Install one with: ferrous-forge rust install-toolchain <channel>");
    } else {
        for toolchain in toolchains {
            let name = toolchain.channel.to_string();
            if toolchain.is_default {
                println!(
                    "   {} {}",
                    style("●").green().bold(),
                    style(&name).green().bold()
                );
                println!("     {} {}", style("└─").dim(), style("(default)").dim());
            } else {
                println!("   {} {}", style("○").dim(), name);
            }
        }
    }

    // Show active toolchain info
    match rustup.show_active_toolchain() {
        Ok(active) => {
            println!();
            println!("{} {}", style("Active:").bold(), active);
        }
        Err(_) => {
            // Already displayed error
        }
    }

    println!();
    Ok(())
}

/// Handle rust update command
///
/// Updates all installed toolchains via rustup.
///
/// # Errors
///
/// Returns an error if rustup is not available or the update fails.
pub async fn handle_update(dry_run: bool, yes: bool, self_update: bool) -> Result<()> {
    let rustup = RustupManager::new();

    if !rustup.is_available() {
        return Err(Error::rust_not_found(
            "rustup not found. Please install rustup from https://rustup.rs",
        ));
    }

    // Check version requirements first
    let version_check = rustup.check_version_requirements().await?;

    println!();
    println!("{}", style("🔄 Rust Update").bold().cyan());
    println!();

    // Show current version status
    let current = rustup.get_current_version().await?;
    println!("📦 Current version: {}", style(&current.version).green());

    if dry_run {
        println!();
        println!(
            "{}",
            style("🔍 Dry run mode - no changes will be made").yellow()
        );
        println!();
        println!("Would update the following toolchains:");

        let toolchains = rustup.list_toolchains()?;
        for toolchain in toolchains {
            println!("   • {}", toolchain.channel);
        }

        if self_update {
            println!("   • rustup (self-update)");
        }

        println!();
        return Ok(());
    }

    // Get confirmation unless --yes flag is used
    if !yes {
        println!();
        println!("This will update all installed toolchains.");
        if self_update {
            println!("Rustup itself will also be updated.");
        }
        print!("{} ", style("Continue? [y/N]:").bold());

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    println!();
    let spinner = create_spinner("Updating toolchains...");

    let result = rustup.update_toolchains().await?;

    spinner.finish_and_clear();

    if result.success {
        println!(
            "{}",
            style("✅ Update completed successfully!").green().bold()
        );

        if !result.updated.is_empty() {
            println!();
            println!("Updated:");
            for item in &result.updated {
                println!("   • {}", item);
            }
        }

        // Also do self-update if requested
        if self_update {
            let spinner = create_spinner("Updating rustup...");

            match rustup.self_update().await {
                Ok(()) => {
                    spinner.finish_and_clear();
                    println!("{}", style("✅ rustup self-update completed!").green());
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    eprintln!(
                        "{}",
                        style(format!("⚠️  rustup self-update failed: {}", e)).yellow()
                    );
                }
            }
        }

        // Show new version.clone()
        let new_version = rustup.get_current_version().await?;
        if new_version.version != current.version {
            println!();
            println!(
                "📦 New version: {}",
                style(&new_version.version).green().bold()
            );
        }

        // Check if new version still meets requirements
        if !version_check.meets_requirements {
            let new_check = rustup.check_version_requirements().await?;
            if new_check.meets_requirements {
                println!(
                    "{}",
                    style("✅ Version now meets locked requirements").green()
                );
            }
        }
    } else {
        println!("{}", style("❌ Update failed").red());
    }

    println!();
    Ok(())
}

/// Handle install-toolchain command
///
/// Installs a specific toolchain via rustup.
///
/// # Errors
///
/// Returns an error if rustup is not available or installation fails.
pub async fn handle_install_toolchain(channel: String, set_default: bool) -> Result<()> {
    let rustup = RustupManager::new();

    if !rustup.is_available() {
        return Err(Error::rust_not_found(
            "rustup not found. Please install rustup from https://rustup.rs",
        ));
    }

    let channel_parsed = ToolchainChannel::parse(&channel);

    println!();
    println!("{}", style("🔧 Install Toolchain").bold().cyan());
    println!();
    println!("Channel: {}", style(&channel).blue().bold());

    let spinner = create_spinner(format!("Installing {} toolchain...", channel).as_str());

    rustup.install_toolchain(&channel_parsed).await?;

    spinner.finish_and_clear();

    println!(
        "{}",
        style(format!("✅ Successfully installed {} toolchain", channel))
            .green()
            .bold()
    );

    // Set as default if requested
    if set_default {
        let spinner = create_spinner("Setting as default...");
        rustup.switch_toolchain(&channel_parsed).await?;
        spinner.finish_and_clear();
        println!(
            "{}",
            style(format!("✅ {} is now the default toolchain", channel)).green()
        );
    }

    println!();
    Ok(())
}

/// Handle uninstall-toolchain command
///
/// Uninstalls a specific toolchain via rustup.
///
/// # Errors
///
/// Returns an error if rustup is not available or uninstallation fails.
pub async fn handle_uninstall_toolchain(channel: String) -> Result<()> {
    let rustup = RustupManager::new();

    if !rustup.is_available() {
        return Err(Error::rust_not_found(
            "rustup not found. Please install rustup from https://rustup.rs",
        ));
    }

    let channel_parsed = ToolchainChannel::parse(&channel);

    println!();
    println!("{}", style("🗑️  Uninstall Toolchain").bold().cyan());
    println!();
    println!("Channel: {}", style(&channel).yellow().bold());

    // Get confirmation
    print!("{}", style("Are you sure? [y/N]: ").bold());

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cancelled.");
        return Ok(());
    }

    let spinner = create_spinner(format!("Uninstalling {} toolchain...", channel).as_str());

    rustup.uninstall_toolchain(&channel_parsed).await?;

    spinner.finish_and_clear();

    println!(
        "{}",
        style(format!("✅ Successfully uninstalled {} toolchain", channel))
            .green()
            .bold()
    );
    println!();

    Ok(())
}

/// Handle switch command
///
/// Switches to a different toolchain (sets as default).
///
/// # Errors
///
/// Returns an error if rustup is not available or switching fails.
pub async fn handle_switch(channel: String) -> Result<()> {
    let rustup = RustupManager::new();

    if !rustup.is_available() {
        return Err(Error::rust_not_found(
            "rustup not found. Please install rustup from https://rustup.rs",
        ));
    }

    let channel_parsed = ToolchainChannel::parse(&channel);

    println!();
    println!("{}", style("🔄 Switch Toolchain").bold().cyan());
    println!();
    println!("Switching to: {}", style(&channel).blue().bold());

    let spinner = create_spinner(format!("Activating {} toolchain...", channel).as_str());

    rustup.switch_toolchain(&channel_parsed).await?;

    spinner.finish_and_clear();

    println!(
        "{}",
        style(format!("✅ Successfully switched to {} toolchain", channel))
            .green()
            .bold()
    );

    // Show new active toolchain
    if let Ok(active) = rustup.show_active_toolchain() {
        println!("{} {}", style("Active:").bold(), active);
    }

    println!();
    Ok(())
}

/// Check version requirements without full version check
///
/// This is a convenience function for enforcing version requirements
/// before operations.
///
/// # Errors
///
/// Returns an error if the current version doesn't meet locked requirements.
pub async fn enforce_version_requirements() -> Result<()> {
    let rustup = RustupManager::new();

    if !rustup.is_available() {
        // If rustup isn't available, we can't enforce requirements
        return Ok(());
    }

    let check = rustup.check_version_requirements().await?;

    if !check.meets_requirements {
        println!();
        println!("{}", style("🚫 Version Requirements Not Met").red().bold());
        println!();
        println!("{}", check.status_message());
        println!();
        println!(
            "{}",
            style("This operation is blocked by locked configuration.").red()
        );
        println!("{}", style("Options:").bold());
        println!("   1. Update Rust: ferrous-forge rust update");
        println!("   2. Adjust locked requirements: ferrous-forge config unlock rust-version");
        println!();

        return Err(Error::validation(format!(
            "Current Rust version {} does not meet locked requirements ({})",
            check.current,
            check.requirements.description()
        )));
    }

    Ok(())
}
/// Handle releases command (alias for list releases)
///
/// Lists recent Rust releases from GitHub.
///
/// # Errors
///
/// Returns an error if the version manager fails to initialize or the
/// release list cannot be fetched.
pub async fn handle_releases(count: usize) -> Result<()> {
    handle_list_releases(count).await
}

/// Handle check-updates command
///
/// Checks if Rust updates are available and shows concise output.
///
/// # Errors
///
/// Returns an error if the version manager fails to initialize or the
/// check fails.
///
/// # Panics
///
/// Panics if `updates_available` is true but `latest_version` is None.
/// This should never happen if the version manager is working correctly.
pub async fn handle_check_updates(verbose: bool) -> Result<()> {
    let spinner = create_spinner("Checking for updates...");
    let manager = VersionManager::new()?;

    let (updates_available, latest_version) = match manager.check_updates().await {
        Ok(result) => result,
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("{}", style("Failed to check for updates").red());
            eprintln!("Error: {}", e);

            if manager.is_offline_mode() {
                println!();
                println!(
                    "{}",
                    style("Operating in offline mode (using cached data)").dim()
                );
            }

            return Err(e);
        }
    };

    let current = manager.check_current().await?;
    spinner.finish_and_clear();

    println!();
    if updates_available {
        let latest = latest_version.ok_or_else(|| {
            Error::validation("Expected latest_version when updates_available is true")
        })?;
        println!("{}", style("Update Available!").green().bold());
        println!(
            "   Current: {} -> Latest: {}",
            style(&current.version).dim(),
            style(&latest).green().bold()
        );

        if verbose {
            let recommendation = manager.get_recommendation().await?;
            display_recommendation(&recommendation);
        } else {
            println!();
            println!("Update command: {}", style("rustup update").cyan());
        }
    } else {
        println!("{}", style("Rust is up to date!").green().bold());
        println!("   Current version: {}", style(&current.version).green());
    }

    if manager.is_offline_mode() {
        println!();
        println!(
            "{}",
            style("Operating in offline mode (using cached data)").dim()
        );
    }

    println!();
    Ok(())
}

/// Handle release-notes command
///
/// Shows release notes for a specific Rust version.
///
/// # Errors
///
/// Returns an error if the version manager fails to initialize or the
/// release notes cannot be fetched.
pub async fn handle_release_notes(version: String, detailed: bool) -> Result<()> {
    let manager = VersionManager::new()?;

    let version_normalized = if version.starts_with('v') {
        version.clone()
    } else {
        format!("v{}", version)
    };

    let spinner = create_spinner(&format!(
        "Fetching release notes for {}...",
        version_normalized
    ));

    let notes = match manager.get_release_notes(&version_normalized).await {
        Ok(n) => n,
        Err(e) => {
            spinner.finish_and_clear();

            if e.to_string().contains("not found") {
                eprintln!(
                    "{}",
                    style(format!("Release '{}' not found", version)).red()
                );
                println!();
                println!("{}", style("Possible causes:").bold());
                println!("   - The version hasn't been released yet");
                println!("   - The version number is incorrect");
                println!();
                println!("{}", style("Try:").bold());
                println!("   ferrous-forge rust releases  -- list recent releases");
            } else if manager.is_offline_mode() {
                eprintln!("{}", style("Release not found in offline mode").red());
                println!();
                println!(
                    "{}",
                    style("This release is not available in the local cache.").dim()
                );
            } else {
                return Err(e);
            }

            return Ok(());
        }
    };

    spinner.finish_and_clear();

    println!();
    println!(
        "{}",
        style(format!("Release Notes: {}", notes.version))
            .bold()
            .cyan()
    );
    println!();

    if !notes.parsed.security_advisories.is_empty() {
        println!("{}", style("Security Advisories:").red().bold());
        for advisory in &notes.parsed.security_advisories {
            let severity_icon = match advisory.severity {
                crate::rust_version::parser::Severity::Critical => "🔴",
                crate::rust_version::parser::Severity::High => "🟠",
                crate::rust_version::parser::Severity::Medium => "🟡",
                crate::rust_version::parser::Severity::Low => "🔵",
                crate::rust_version::parser::Severity::Unknown => "⚪",
            };

            println!(
                "   {} [{}] {}",
                severity_icon,
                style(&advisory.severity).bold(),
                advisory.description
            );

            if let Some(ref id) = advisory.id {
                println!("      ID: {}", style(id).cyan());
            }
        }
        println!();
    }

    if !notes.parsed.breaking_changes.is_empty() {
        println!("{}", style("Breaking Changes:").yellow().bold());
        for change in &notes.parsed.breaking_changes {
            println!("   • {}", change.description);
            if let Some(ref migration) = change.migration {
                println!("     Migration: {}", style(migration).dim());
            }
        }
        println!();
    }

    println!("{}", style("Full Release Notes:").bold());
    println!();

    for line in notes.full_notes.lines() {
        if line.trim().is_empty() {
            println!();
        } else {
            println!("{}", line);
        }
    }

    if detailed {
        println!();
        println!("{}", style("Detailed Analysis:").bold().cyan());
        println!();

        println!(
            "   Security advisories: {}",
            notes.parsed.security_advisories.len()
        );
        println!(
            "   Breaking changes: {}",
            notes.parsed.breaking_changes.len()
        );
        println!("   New features: {}", notes.parsed.new_features.len());
        println!(
            "   Performance improvements: {}",
            notes.parsed.performance_improvements.len()
        );
        println!("   Bug fixes: {}", notes.parsed.bug_fixes.len());
    }

    if manager.is_offline_mode() {
        println!();
        println!(
            "{}",
            style("Showing cached release notes (offline mode)").dim()
        );
    }

    println!();
    Ok(())
}

/// Handle security command
///
/// Checks for security advisories affecting the current Rust version.
///
/// # Errors
///
/// Returns an error if the security check fails or if `fail_on_issues` is true
/// and issues are found.
pub async fn handle_security(fail_on_issues: bool) -> Result<()> {
    use crate::rust_version::SecurityChecker;

    let spinner = create_spinner("Checking for security advisories...");
    let checker = SecurityChecker::new()?;

    let result = match checker.check_current_version().await {
        Ok(r) => r,
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("{}", style("Security check failed").red());
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    spinner.finish_and_clear();

    SecurityChecker::display_results(&result);

    if fail_on_issues && !result.is_secure {
        return Err(Error::validation("Security issues found. Update required."));
    }

    Ok(())
}

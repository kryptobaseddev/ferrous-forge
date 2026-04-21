//! Update command implementation

use crate::{Error, Result};
use console::style;

/// Execute the update command
///
/// # Errors
///
/// Returns an error if the binary or rules update process fails.
pub async fn execute(channel: String, rules_only: bool, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "{}",
            style("🔍 Dry run mode - showing what would be updated")
                .bold()
                .yellow()
        );
    } else {
        println!("{}", style("🔄 Updating Ferrous Forge...").bold().cyan());
    }

    if rules_only {
        update_rules(&channel, dry_run).await?;
    } else {
        update_binary(&channel, dry_run).await?;
        update_rules(&channel, dry_run).await?;
    }

    if !dry_run {
        println!("{}", style("✅ Update complete!").bold().green());
    }

    Ok(())
}

/// Update the Ferrous Forge binary to the latest version from GitHub releases.
///
/// Uses the `self_update` crate to download the correct archive for the
/// current platform, extract it, and replace the running binary.
///
/// # Arguments
/// * `channel` - Update channel (stable, beta, nightly)
/// * `dry_run` - If true, only shows what would be updated without making changes
async fn update_binary(channel: &str, dry_run: bool) -> Result<()> {
    println!("📦 Checking for binary updates on {} channel...", channel);

    if dry_run {
        println!("  Would check GitHub releases for newer version");
        println!(
            "  Would download and install new binary for {}",
            get_target_name()?
        );
        return Ok(());
    }

    let status = self_update::backends::github::Update::configure()
        .repo_owner("kryptobaseddev")
        .repo_name("ferrous-forge")
        .bin_name("ferrous-forge")
        .target(&get_target_name()?)
        .bin_path_in_archive(&get_bin_path_in_archive())
        .bin_install_path(
            std::env::current_exe()
                .map_err(|e| Error::io(format!("Cannot locate current executable: {e}")))?,
        )
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_download_progress(true)
        .show_output(true)
        .build()
        .map_err(|e| Error::process(format!("Failed to configure updater: {e}")))?
        .update()
        .map_err(|e| Error::process(format!("Update failed: {e}")))?;

    match status {
        self_update::Status::Updated(v) => {
            println!(
                "{}",
                style(format!("🆕 Updated to version {v}!")).green().bold()
            );
        }
        self_update::Status::UpToDate(v) => {
            println!("{}", style(format!("✅ Already up to date (v{v})")).green());
        }
    }

    Ok(())
}

/// Determine the asset target name for the current platform.
///
/// GitHub release assets are named like `ferrous-forge-{target}.{ext}`.
/// This function returns the `{target}` portion.
fn get_target_name() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let suffix = match (os, arch) {
        ("linux", "x86_64") => {
            if cfg!(target_env = "musl") {
                "linux-x86_64-musl.tar.gz"
            } else {
                "linux-x86_64.tar.gz"
            }
        }
        ("macos", "x86_64") => "macos-x86_64.tar.gz",
        ("macos", "aarch64") => "macos-aarch64.tar.gz",
        ("windows", "x86_64") => "windows-x86_64.zip",
        _ => {
            return Err(Error::config(format!(
                "Self-update is not supported on {os}/{arch}"
            )));
        }
    };

    Ok(suffix.to_string())
}

/// Return the binary path inside the release archive.
fn get_bin_path_in_archive() -> String {
    if cfg!(windows) {
        "ferrous-forge.exe".to_string()
    } else {
        "ferrous-forge".to_string()
    }
}

/// Update validation rules from the remote repository
///
/// Fetches the latest clippy rules and other validation configurations
/// from the Ferrous Forge repository for the specified channel.
///
/// # Arguments
/// * `channel` - Update channel for rules (stable, beta, nightly)
/// * `dry_run` - If true, only shows what would be updated without making changes
async fn update_rules(channel: &str, dry_run: bool) -> Result<()> {
    println!("📋 Checking for rules updates on {} channel...", channel);

    if dry_run {
        println!("  Would fetch latest clippy rules from repository");
        println!("  Would update ~/.clippy.toml with new rules");
        return Ok(());
    }

    // Rules are embedded in the binary; updating the binary also updates rules.
    println!("  ℹ️  Rules are bundled with the binary and updated automatically.");

    Ok(())
}

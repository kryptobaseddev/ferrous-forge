//! Uninstall command implementation

use crate::{Error, Result};
use console::style;
use std::io::{self, Write};

/// Execute the uninstall command
///
/// # Errors
///
/// Returns an error if user confirmation fails to read, or if any
/// uninstall step (removing files, shell integration) fails.
pub async fn execute(confirm: bool) -> Result<()> {
    if !confirm && !get_user_confirmation()? {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    println!(
        "{}",
        style("🗑️  Uninstalling Ferrous Forge...").bold().red()
    );

    perform_uninstall().await?;
    print_uninstall_complete();

    Ok(())
}

/// Get user confirmation for uninstall
fn get_user_confirmation() -> Result<bool> {
    print!(
        "Are you sure you want to uninstall Ferrous Forge? \
        This will remove all system integration. [y/N]: "
    );
    io::stdout()
        .flush()
        .map_err(|e| Error::process(format!("Failed to flush stdout: {}", e)))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| Error::process(format!("Failed to read input: {}", e)))?;

    Ok(input.trim().to_lowercase().starts_with('y'))
}

/// Perform the actual uninstall steps
async fn perform_uninstall() -> Result<()> {
    remove_cargo_hijacking().await?;
    remove_clippy_config().await?;
    remove_shell_integration().await?;
    remove_configuration().await?;
    Ok(())
}

/// Print uninstall completion message
fn print_uninstall_complete() {
    println!(
        "{}",
        style("✅ Ferrous Forge has been uninstalled")
            .bold()
            .green()
    );
    println!();
    println!("Note: You may need to restart your shell or run:");
    println!("  source ~/.bashrc");
    println!();
    println!("To completely remove the binary, run:");
    println!("  cargo uninstall ferrous-forge");
}

/// Remove the cargo wrapper script from ~/.local/bin
///
/// Deletes the cargo hijacking wrapper that intercepts cargo commands.
async fn remove_cargo_hijacking() -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| Error::config("Could not find home directory"))?;

    let cargo_wrapper = home_dir.join(".local").join("bin").join("cargo");
    if cargo_wrapper.exists() {
        tokio::fs::remove_file(&cargo_wrapper).await?;
        println!("  ✅ Removed cargo hijacking");
    }

    Ok(())
}

/// Remove the global clippy configuration file
///
/// Deletes ~/.clippy.toml that contains Ferrous Forge's clippy rules.
async fn remove_clippy_config() -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| Error::config("Could not find home directory"))?;

    let clippy_config = home_dir.join(".clippy.toml");
    if clippy_config.exists() {
        tokio::fs::remove_file(&clippy_config).await?;
        println!("  ✅ Removed clippy configuration");
    }

    Ok(())
}

/// Remove shell integration from configuration files
///
/// Removes PATH modifications and Ferrous Forge sections from
/// .bashrc, .zshrc, and .profile files.
async fn remove_shell_integration() -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| Error::config("Could not find home directory"))?;

    for shell_file in &[".bashrc", ".zshrc", ".profile"] {
        let shell_path = home_dir.join(shell_file);
        if shell_path.exists() {
            if let Ok(contents) = tokio::fs::read_to_string(&shell_path).await {
                if contents.contains("Ferrous Forge") {
                    // Remove Ferrous Forge section
                    let new_contents = remove_ferrous_forge_section(&contents);
                    tokio::fs::write(&shell_path, new_contents).await?;
                    println!("  ✅ Removed shell integration from {}", shell_file);
                }
            }
        }
    }

    Ok(())
}

/// Remove Ferrous Forge configuration directory
///
/// Deletes the entire configuration directory containing
/// Ferrous Forge settings and cached data.
async fn remove_configuration() -> Result<()> {
    let config_dir = crate::config::Config::config_dir_path()?;
    if config_dir.exists() {
        tokio::fs::remove_dir_all(&config_dir).await?;
        println!("  ✅ Removed configuration directory");
    }

    Ok(())
}

/// Remove Ferrous Forge sections from shell configuration content
///
/// Identifies and removes lines between "# Ferrous Forge" markers
/// and any related environment variable exports.
fn remove_ferrous_forge_section(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut skip = false;

    for line in lines {
        if line.contains("# Ferrous Forge") {
            skip = true;
            continue;
        }

        if skip
            && (line.trim().is_empty() || line.starts_with('#') || line.contains("FERROUS_FORGE"))
        {
            continue;
        }

        if skip
            && !line.trim().is_empty()
            && !line.starts_with('#')
            && !line.contains("FERROUS_FORGE")
        {
            skip = false;
        }

        if !skip {
            result.push(line);
        }
    }

    result.join("\n")
}

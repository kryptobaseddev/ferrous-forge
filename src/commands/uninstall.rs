//! Uninstall command implementation

use crate::{Error, Result};
use console::style;
use std::io::{self, Write};

/// Execute the uninstall command
pub async fn execute(confirm: bool) -> Result<()> {
    if !confirm {
        print!(
            "Are you sure you want to uninstall Ferrous Forge? This will remove all system integration. [y/N]: "
        );
        io::stdout()
            .flush()
            .map_err(|e| Error::process(format!("Failed to flush stdout: {}", e)))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| Error::process(format!("Failed to read input: {}", e)))?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Uninstall cancelled.");
            return Ok(());
        }
    }

    println!(
        "{}",
        style("🗑️  Uninstalling Ferrous Forge...").bold().red()
    );

    // Remove cargo wrapper
    remove_cargo_hijacking().await?;

    // Remove clippy config
    remove_clippy_config().await?;

    // Remove shell integration
    remove_shell_integration().await?;

    // Remove configuration
    remove_configuration().await?;

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

    Ok(())
}

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

async fn remove_configuration() -> Result<()> {
    let config_dir = crate::config::Config::config_dir_path()?;
    if config_dir.exists() {
        tokio::fs::remove_dir_all(&config_dir).await?;
        println!("  ✅ Removed configuration directory");
    }

    Ok(())
}

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

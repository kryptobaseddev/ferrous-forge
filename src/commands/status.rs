//! Status command implementation

use crate::{Result, config::Config};
use console::style;

/// Execute the status command
///
/// # Errors
///
/// Returns an error if the configuration cannot be loaded.
pub async fn execute() -> Result<()> {
    println!("{}", style("🔨 Ferrous Forge Status").bold().cyan());
    println!();

    // Load configuration
    let config = Config::load_or_default().await?;

    // Basic status
    if config.is_initialized() {
        println!("{}", style("✅ Status: Initialized").green());
    } else {
        println!("{}", style("❌ Status: Not initialized").red());
        println!("Run 'ferrous-forge init' to set up system-wide standards.");
        return Ok(());
    }

    // Version information
    println!("📦 Version: {}", crate::VERSION);
    println!("🦀 Min Rust Version: {}", crate::MIN_RUST_VERSION);
    println!("📐 Required Edition: {}", crate::REQUIRED_EDITION);
    println!();

    // Configuration
    println!("{}", style("⚙️  Configuration:").bold());
    println!("  Update Channel: {}", config.update_channel);
    println!("  Auto Update: {}", config.auto_update);
    println!("  Max File Lines: {}", config.max_file_lines);
    println!("  Max Function Lines: {}", config.max_function_lines);
    println!("  Required Edition: {}", config.required_edition);
    println!("  Required Rust Version: {}", config.required_rust_version);
    println!(
        "  Ban Underscore Bandaid: {}",
        config.ban_underscore_bandaid
    );
    println!("  Require Documentation: {}", config.require_documentation);
    println!();

    // Check system integration
    println!("{}", style("🔗 System Integration:").bold());
    check_cargo_hijacking().await;
    check_clippy_config().await;
    check_shell_integration().await;

    Ok(())
}

/// Check if cargo wrapper hijacking is installed
///
/// Verifies whether the Ferrous Forge cargo wrapper is present in ~/.local/bin/cargo
/// to intercept cargo commands and enforce standards.
async fn check_cargo_hijacking() {
    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => {
            println!("  ❌ Cargo hijacking: Cannot find home directory");
            return;
        }
    };

    let cargo_wrapper = home_dir.join(".local").join("bin").join("cargo");
    if cargo_wrapper.exists() {
        println!("  ✅ Cargo hijacking: Installed");
    } else {
        println!("  ❌ Cargo hijacking: Not installed");
    }
}

/// Check if global clippy configuration is installed
///
/// Verifies whether the Ferrous Forge clippy configuration file exists
/// at ~/.clippy.toml for enforcing strict linting rules.
async fn check_clippy_config() {
    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => {
            println!("  ❌ Clippy config: Cannot find home directory");
            return;
        }
    };

    let clippy_config = home_dir.join(".clippy.toml");
    if clippy_config.exists() {
        println!("  ✅ Clippy config: Installed");
    } else {
        println!("  ❌ Clippy config: Not installed");
    }
}

/// Check if shell integration is configured
///
/// Scans common shell configuration files (.bashrc, .zshrc, .profile)
/// to verify if Ferrous Forge PATH modifications are present.
async fn check_shell_integration() {
    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => {
            println!("  ❌ Shell integration: Cannot find home directory");
            return;
        }
    };

    let mut found = false;
    for shell_file in &[".bashrc", ".zshrc", ".profile"] {
        let shell_path = home_dir.join(shell_file);
        if shell_path.exists() {
            if let Ok(contents) = tokio::fs::read_to_string(&shell_path).await {
                if contents.contains("Ferrous Forge") {
                    found = true;
                    break;
                }
            }
        }
    }

    if found {
        println!("  ✅ Shell integration: Installed");
    } else {
        println!("  ❌ Shell integration: Not installed");
    }
}

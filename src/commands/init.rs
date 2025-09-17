//! Initialize command implementation

use crate::{Result, config::Config};
use console::style;

/// Execute the init command
pub async fn execute(force: bool) -> Result<()> {
    println!("{}", style("🔨 Initializing Ferrous Forge...").bold().cyan());

    // Check if already initialized
    let config = Config::load_or_default().await?;
    if config.is_initialized() && !force {
        println!("{}", style("✅ Ferrous Forge is already initialized!").green());
        println!("Use --force to reinitialize.");
        return Ok(());
    }

    // Create configuration directories
    println!("📁 Creating configuration directories...");
    config.ensure_directories().await?;

    // Install system-wide cargo hijacking
    println!("🔧 Setting up cargo command hijacking...");
    install_cargo_hijacking().await?;

    // Copy clippy configuration
    println!("📋 Installing clippy configuration...");
    install_clippy_config().await?;

    // Setup shell integration
    println!("🐚 Installing shell integration...");
    install_shell_integration().await?;

    // Mark as initialized
    let mut config = config;
    config.mark_initialized();
    config.save().await?;

    println!("{}", style("🎉 Ferrous Forge initialization complete!").bold().green());
    println!();
    println!("Next steps:");
    println!("• Restart your shell or run: source ~/.bashrc");
    println!("• Create a new project: cargo new my-project");
    println!("• All new projects will automatically use Edition 2024 + strict standards!");

    Ok(())
}

async fn install_cargo_hijacking() -> Result<()> {
    // This will create wrapper scripts that intercept cargo commands
    // and apply our standards before delegating to the real cargo
    
    let home_dir = dirs::home_dir()
        .ok_or_else(|| crate::Error::config("Could not find home directory"))?;
    
    let bin_dir = home_dir.join(".local").join("bin");
    tokio::fs::create_dir_all(&bin_dir).await?;

    // Create cargo wrapper script
    let cargo_wrapper = include_str!("../../templates/cargo-wrapper.sh");
    let cargo_path = bin_dir.join("cargo");
    tokio::fs::write(&cargo_path, cargo_wrapper).await?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&cargo_path).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&cargo_path, perms).await?;
    }

    Ok(())
}

async fn install_clippy_config() -> Result<()> {
    // Copy our strict clippy.toml to the home directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| crate::Error::config("Could not find home directory"))?;
    
    let clippy_config = include_str!("../../templates/clippy.toml");
    let clippy_path = home_dir.join(".clippy.toml");
    tokio::fs::write(&clippy_path, clippy_config).await?;

    Ok(())
}

async fn install_shell_integration() -> Result<()> {
    // Add Ferrous Forge to PATH and setup completion
    let home_dir = dirs::home_dir()
        .ok_or_else(|| crate::Error::config("Could not find home directory"))?;
    
    let shell_config = format!(
        r#"
# Ferrous Forge - Rust Development Standards Enforcer
export PATH="$HOME/.local/bin:$PATH"

# Enable Ferrous Forge for all Rust development
export FERROUS_FORGE_ENABLED=1
"#
    );

    // Add to common shell config files
    for shell_file in &[".bashrc", ".zshrc", ".profile"] {
        let shell_path = home_dir.join(shell_file);
        if shell_path.exists() {
            let mut contents = tokio::fs::read_to_string(&shell_path).await?;
            if !contents.contains("Ferrous Forge") {
                contents.push_str(&shell_config);
                tokio::fs::write(&shell_path, contents).await?;
            }
        }
    }

    Ok(())
}
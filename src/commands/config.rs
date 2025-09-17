//! Config command implementation

use crate::{Result, config::Config};
use console::style;

/// Execute the config command
pub async fn execute(
    set: Option<String>,
    get: Option<String>, 
    list: bool,
    reset: bool,
) -> Result<()> {
    let mut config = Config::load_or_default().await?;

    if reset {
        config.reset();
        config.save().await?;
        println!("{}", style("✅ Configuration reset to defaults").green());
        return Ok(());
    }

    if list {
        println!("{}", style("⚙️  Ferrous Forge Configuration:").bold().cyan());
        println!();
        
        for (key, value) in config.list() {
            println!("  {}: {}", style(&key).bold(), value);
        }
        return Ok(());
    }

    if let Some(get_key) = get {
        if let Some(value) = config.get(&get_key) {
            println!("{}", value);
        } else {
            println!("{}", style(&format!("Unknown configuration key: {}", get_key)).red());
            std::process::exit(1);
        }
        return Ok(());
    }

    if let Some(set_value) = set {
        if let Some((key, value)) = set_value.split_once('=') {
            match config.set(key, value) {
                Ok(()) => {
                    config.save().await?;
                    println!("{}", style(&format!("✅ Set {} = {}", key, value)).green());
                }
                Err(e) => {
                    println!("{}", style(&format!("❌ Error: {}", e)).red());
                    std::process::exit(1);
                }
            }
        } else {
            println!("{}", style("❌ Invalid format. Use: key=value").red());
            std::process::exit(1);
        }
        return Ok(());
    }

    // If no specific action, show help
    println!("{}", style("⚙️  Ferrous Forge Configuration").bold().cyan());
    println!();
    println!("Usage:");
    println!("  ferrous-forge config --list           # Show all settings");
    println!("  ferrous-forge config --get key        # Get a setting");
    println!("  ferrous-forge config --set key=value  # Set a setting");
    println!("  ferrous-forge config --reset          # Reset to defaults");
    println!();
    println!("Available settings:");
    println!("  update_channel (stable, beta, nightly)");
    println!("  auto_update (true, false)");
    println!("  max_file_lines (number)");
    println!("  max_function_lines (number)");
    println!("  enforce_edition_2024 (true, false)");
    println!("  ban_underscore_bandaid (true, false)");
    println!("  require_documentation (true, false)");

    Ok(())
}
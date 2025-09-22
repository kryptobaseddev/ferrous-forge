//! Config command implementation

use crate::{config::Config, Result};
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
        return handle_reset(&mut config).await;
    }

    if list {
        return handle_list(&config);
    }

    if let Some(get_key) = get {
        return handle_get(&config, &get_key);
    }

    if let Some(set_value) = set {
        return handle_set(&mut config, &set_value).await;
    }

    // If no specific action, show help
    show_help();
    Ok(())
}

/// Handle the reset command
async fn handle_reset(config: &mut Config) -> Result<()> {
    config.reset();
    config.save().await?;
    println!("{}", style("✅ Configuration reset to defaults").green());
    Ok(())
}

/// Handle the list command
fn handle_list(config: &Config) -> Result<()> {
    println!(
        "{}",
        style("⚙️  Ferrous Forge Configuration:").bold().cyan()
    );
    println!();

    for (key, value) in config.list() {
        println!("  {}: {}", style(&key).bold(), value);
    }
    Ok(())
}

/// Handle the get command
fn handle_get(config: &Config, get_key: &str) -> Result<()> {
    if let Some(value) = config.get(get_key) {
        println!("{}", value);
    } else {
        println!(
            "{}",
            style(&format!("Unknown configuration key: {}", get_key)).red()
        );
        std::process::exit(1);
    }
    Ok(())
}

/// Handle the set command
async fn handle_set(config: &mut Config, set_value: &str) -> Result<()> {
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
    Ok(())
}

/// Show configuration help
fn show_help() {
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
}

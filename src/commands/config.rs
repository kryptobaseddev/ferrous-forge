//! Config command implementation
//!
//! @task T015
//! @task T018
//! @epic T014

use crate::commands::ConfigCommand;
use crate::config::{
    Config, ConfigValidator, HierarchicalConfig, HierarchicalLockManager, ImportOptions,
    SharedConfig, audit_log, import_shared_config,
};
use crate::{Error, Result};
use console::style;

/// Execute the config command with subcommand
///
/// # Errors
///
/// Returns an error if any configuration operation fails.
pub async fn execute_with_subcommand(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Get { key } => handle_get(&key).await,
        ConfigCommand::Set { value } => handle_set(&value).await,
        ConfigCommand::List => handle_list().await,
        ConfigCommand::Reset => handle_reset().await,
        ConfigCommand::Sources => show_sources().await,
        ConfigCommand::Migrate => migrate_config().await,
        ConfigCommand::Lock {
            key,
            value,
            reason,
            level,
        } => handle_lock(&key, value, &reason, level.to_config_level()).await,
        ConfigCommand::Unlock { key, reason, level } => {
            handle_unlock(&key, &reason, level.to_config_level()).await
        }
        ConfigCommand::LockStatus => handle_lock_status().await,
        ConfigCommand::LockAudit { limit } => handle_lock_audit(limit).await,
        ConfigCommand::Export {
            level,
            output,
            description,
        } => handle_export(level.to_config_level(), output, description).await,
        ConfigCommand::Import {
            file,
            level,
            no_locks,
            force,
        } => handle_import(file, level.to_config_level(), no_locks, force).await,
    }
}

/// Handle the get command
async fn handle_get(key: &str) -> Result<()> {
    let config = Config::load_or_default().await?;

    if let Some(value) = config.get(key) {
        println!("{}", value);
    } else {
        println!(
            "{}",
            style(&format!("Unknown configuration key: {}", key)).red()
        );
        std::process::exit(1);
    }
    Ok(())
}

/// Handle the set command with lock validation
async fn handle_set(set_value: &str) -> Result<()> {
    let mut config = Config::load_or_default().await?;

    if let Some((key, value)) = set_value.split_once('=') {
        // Validate against locks before setting
        ConfigValidator::validate_change(key, value).await?;

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

/// Handle the list command
async fn handle_list() -> Result<()> {
    let config = Config::load_or_default().await?;
    let lock_manager = HierarchicalLockManager::load().await?;

    println!(
        "{}",
        style("⚙️  Ferrous Forge Configuration:").bold().cyan()
    );
    println!();

    for (key, value) in config.list() {
        if lock_manager.is_locked(&key).is_some() {
            println!(
                "  {} {}: {}",
                style(&key).bold(),
                style("🔒").yellow(),
                value
            );
        } else {
            println!("  {}: {}", style(&key).bold(), value);
        }
    }

    // Show locked keys separately if any
    let locks = lock_manager.get_effective_locks();
    if !locks.is_empty() {
        println!();
        println!(
            "{}",
            style("🔒 Locked Configuration Values:").bold().yellow()
        );
        println!();

        for (key, (level, entry)) in locks {
            println!(
                "  {} = {} (locked at {} level)",
                style(&key).bold(),
                entry.value,
                level.display_name()
            );
            println!("    Reason: {}", entry.reason);
            println!(
                "    Locked by {} at {}",
                entry.locked_by,
                entry.locked_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!();
        }
    }

    Ok(())
}

/// Handle the reset command
#[allow(clippy::collapsible_if)]
async fn handle_reset() -> Result<()> {
    let mut config = Config::load_or_default().await?;

    // Check for locked keys before reset
    let lock_manager = HierarchicalLockManager::load().await?;
    let locks = lock_manager.get_effective_locks();

    if !locks.is_empty() {
        println!(
            "{}",
            style("⚠️  Warning: Some configuration values are locked and will not be reset:")
                .yellow()
        );
        for key in locks.keys() {
            println!("  - {}", key);
        }
        println!();
    }

    // Reset only non-locked values
    let default = Config::default();
    for key in config.list().iter().map(|(k, _)| k) {
        if !locks.contains_key(key) {
            if let Some(default_value) = default.get(key) {
                let _ = config.set(key, &default_value);
            }
        }
    }

    config.save().await?;
    println!(
        "{}",
        style("✅ Configuration reset to defaults (locked values preserved)").green()
    );
    Ok(())
}

/// Handle the lock command
async fn handle_lock(
    key: &str,
    value: Option<String>,
    reason: &str,
    level: crate::config::ConfigLevel,
) -> Result<()> {
    // Get current value if not provided
    let value_to_lock = if let Some(v) = value {
        v
    } else {
        let config = Config::load_or_default().await?;
        config
            .get(key)
            .ok_or_else(|| Error::config(format!("Unknown configuration key: {}", key)))?
    };

    let mut lock_manager = HierarchicalLockManager::load().await?;

    match lock_manager
        .lock(key, value_to_lock.clone(), reason, level)
        .await
    {
        Ok(()) => {
            println!(
                "{}",
                style(format!(
                    "🔒 Locked {} = {} at {} level",
                    key,
                    value_to_lock,
                    level.display_name()
                ))
                .green()
                .bold()
            );
            println!("  Reason: {}", reason);
        }
        Err(e) => {
            println!("{}", style(format!("❌ Failed to lock: {}", e)).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle the unlock command
async fn handle_unlock(key: &str, reason: &str, level: crate::config::ConfigLevel) -> Result<()> {
    let mut lock_manager = HierarchicalLockManager::load().await?;

    match lock_manager.unlock(key, level, reason).await {
        Ok(entry) => {
            println!(
                "{}",
                style(format!(
                    "🔓 Unlocked {} at {} level",
                    key,
                    level.display_name()
                ))
                .green()
                .bold()
            );
            println!("  Previous value: {}", entry.value);
            println!("  Unlock reason: {}", reason);
            println!(
                "  Originally locked by {} at {}",
                entry.locked_by,
                entry.locked_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
        Err(e) => {
            println!("{}", style(format!("❌ Failed to unlock: {}", e)).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle the lock status command
async fn handle_lock_status() -> Result<()> {
    let lock_manager = HierarchicalLockManager::load().await?;
    println!("{}", lock_manager.status_report());
    Ok(())
}

/// Handle the lock audit command
async fn handle_lock_audit(limit: usize) -> Result<()> {
    let entries = audit_log::read_audit_log().await?;

    println!("{}", style("📋 Configuration Lock Audit Log").bold().cyan());
    println!();

    if entries.is_empty() {
        println!("No audit entries found.");
        return Ok(());
    }

    // Show most recent entries first
    for entry in entries.iter().rev().take(limit) {
        let status_icon = if entry.success {
            style("✅").green()
        } else {
            style("❌").red()
        };

        println!(
            "{} {} - {} at {} level",
            status_icon,
            entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            entry.operation.to_uppercase(),
            entry.level
        );
        println!("   Key: {}", entry.key);
        if let Some(ref value) = entry.value {
            println!("   Value: {}", value);
        }
        println!("   User: {}", entry.user);
        println!("   Reason: {}", entry.reason);
        if let Some(ref error) = entry.error {
            println!("   Error: {}", error);
        }
        println!();
    }

    if entries.len() > limit {
        println!(
            "... and {} more entries (use --limit to show more)",
            entries.len() - limit
        );
    }

    Ok(())
}

/// Show configuration help
pub fn show_help() {
    println!("{}", style("⚙️  Ferrous Forge Configuration").bold().cyan());
    println!();
    println!("Usage:");
    println!("  ferrous-forge config list                    # Show all settings");
    println!("  ferrous-forge config get <key>              # Get a setting");
    println!("  ferrous-forge config set <key=value>        # Set a setting");
    println!("  ferrous-forge config reset                   # Reset to defaults");
    println!("  ferrous-forge config sources                 # Show config hierarchy");
    println!("  ferrous-forge config migrate                 # Migrate old config");
    println!();
    println!("Locking Commands:");
    println!("  ferrous-forge config lock <key> --reason=\"...\" [--level=project]");
    println!("  ferrous-forge config unlock <key> --reason=\"...\" [--level=project]");
    println!("  ferrous-forge config lock-status             # Show all locked values");
    println!("  ferrous-forge config lock-audit              # View lock audit log");
    println!();
    println!("Sharing Commands:");
    println!("  ferrous-forge config export --level=user > team-config.toml");
    println!("  ferrous-forge config import team-config.toml [--level=project]");
    println!("  ferrous-forge config export --output=shared.toml --description=\"Team standards\"");
    println!();
    println!("Configuration Hierarchy:");
    println!("  1. System: /etc/ferrous-forge/config.toml");
    println!("  2. User: ~/.config/ferrous-forge/config.toml");
    println!("  3. Project: ./.ferrous-forge/config.toml");
    println!();
    println!("Lock Levels:");
    println!("  Project-level locks override User-level and System-level locks");
    println!("  User-level locks override System-level locks");
    println!();
    println!("Sharing:");
    println!("  Export creates a git-friendly TOML file with config and locks");
    println!("  Import merges safely, respecting existing locks at higher levels");
    println!();
    println!("Available settings:");
    println!("  update_channel (stable, beta, nightly)");
    println!("  auto_update (true, false)");
    println!("  max_file_lines (number)");
    println!("  max_function_lines (number)");
    println!("  required_edition (2015, 2018, 2021, 2024) 🔒 lockable");
    println!("  required_rust_version (version string) 🔒 lockable");
    println!("  ban_underscore_bandaid (true, false)");
    println!("  require_documentation (true, false)");
}

/// Show configuration sources
///
/// # Errors
///
/// Returns an error if the hierarchical configuration cannot be loaded.
pub async fn show_sources() -> Result<()> {
    println!("{}", style("🔍 Configuration Hierarchy").bold().cyan());
    println!();

    let hier = HierarchicalConfig::load().await?;
    println!("{}", hier.sources_report());

    // Also show lock sources
    let lock_manager = HierarchicalLockManager::load().await?;
    println!("{}", lock_manager.status_report());

    Ok(())
}

/// Migrate old configuration to hierarchical system
///
/// # Errors
///
/// Returns an error if the configuration migration fails.
pub async fn migrate_config() -> Result<()> {
    println!("{}", style("📦 Migrating configuration...").bold().cyan());

    crate::config::hierarchy::migrate_config().await?;

    println!(
        "{}",
        style("✅ Configuration migrated successfully").green()
    );
    Ok(())
}

/// Handle config export command
///
/// # Errors
///
/// Returns an error if the export fails.
async fn handle_export(
    level: crate::config::ConfigLevel,
    output: Option<std::path::PathBuf>,
    description: Option<String>,
) -> Result<()> {
    let mut shared = SharedConfig::from_level(level).await?;

    if let Some(desc) = description {
        shared = shared.with_description(desc);
    }

    // Print summary
    println!("{}", style("📤 Exporting Configuration").bold().cyan());
    println!();
    println!("{}", shared.summary());

    if let Some(path) = output {
        // Export to file
        shared.export_to_file(&path).await?;
        println!(
            "{}",
            style(format!("✅ Exported to {}", path.display())).green()
        );
    } else {
        // Export to stdout
        let toml = shared.to_toml()?;
        println!("{}", style("--- Configuration (TOML) ---").dim());
        println!("{}", toml);
    }

    Ok(())
}

/// Handle config import command
///
/// # Errors
///
/// Returns an error if the import fails.
async fn handle_import(
    file: std::path::PathBuf,
    level: crate::config::ConfigLevel,
    no_locks: bool,
    force: bool,
) -> Result<()> {
    println!("{}", style("📥 Importing Configuration").bold().cyan());
    println!();

    // Load the shared config
    let shared = SharedConfig::import_from_file(&file).await?;
    println!("{}", shared.summary());

    // Configure import options
    let options = ImportOptions {
        target_level: level,
        overwrite: force,
        import_locks: !no_locks,
        require_justification: true,
    };

    // Perform import
    match import_shared_config(&shared, options).await {
        Ok(report) => {
            if !report.conflicts.is_empty() {
                println!(
                    "{}",
                    style("⚠️ Import blocked by lock conflicts:")
                        .yellow()
                        .bold()
                );
                for conflict in &report.conflicts {
                    println!(
                        "  • {} is locked at {} level with value '{}'",
                        style(&conflict.key).bold(),
                        conflict.locked_at.display_name(),
                        conflict.current_value
                    );
                    println!("    Attempted value: '{}'", conflict.attempted_value);
                }
                println!();
                println!(
                    "{}",
                    style("Use --force to override (requires unlock first)").dim()
                );
                return Err(Error::config("Import blocked by lock conflicts"));
            }

            println!("{}", style("✅ Import successful!").green().bold());
            println!(
                "  • {} configuration keys updated",
                report.config_keys_updated
            );
            println!("  • {} locks imported", report.locks_imported.len());
            if !report.locks_skipped.is_empty() {
                println!(
                    "  • {} locks skipped (already exist)",
                    report.locks_skipped.len()
                );
            }
        }
        Err(e) => {
            println!("{}", style(format!("❌ Import failed: {}", e)).red());
            return Err(e);
        }
    }

    Ok(())
}

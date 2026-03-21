//! Hierarchical configuration system
//!
//! Implements a three-level configuration hierarchy:
//! 1. System: /etc/ferrous-forge/config.toml
//! 2. User: ~/.config/ferrous-forge/config.toml  
//! 3. Project: ./.ferrous-forge/config.toml

/// Configuration level definitions (system, user, project).
pub mod levels;
/// Partial configuration for merging across hierarchy levels.
pub mod partial;

pub use levels::ConfigLevel;
pub use partial::PartialConfig;

use crate::config::Config;
use crate::{Error, Result};
use tokio::fs;
use tracing::info;

/// Hierarchical configuration manager
pub struct HierarchicalConfig {
    /// System-level configuration
    pub system: Option<PartialConfig>,
    /// User-level configuration
    pub user: Option<PartialConfig>,
    /// Project-level configuration
    pub project: Option<PartialConfig>,
}

impl HierarchicalConfig {
    /// Load configuration from all levels
    ///
    /// # Errors
    ///
    /// Returns an error if reading or parsing any configuration level fails.
    pub async fn load() -> Result<Self> {
        let system = PartialConfig::load_from_level(ConfigLevel::System).await?;
        let user = PartialConfig::load_from_level(ConfigLevel::User).await?;
        let project = PartialConfig::load_from_level(ConfigLevel::Project).await?;

        Ok(Self {
            system,
            user,
            project,
        })
    }

    /// Get merged configuration with proper precedence
    pub fn merged(&self) -> Config {
        let mut merged = PartialConfig::default();

        // Apply in order of precedence (lowest to highest)
        if let Some(system) = &self.system {
            merged = merged.merge(system.clone());
        }
        if let Some(user) = &self.user {
            merged = merged.merge(user.clone());
        }
        if let Some(project) = &self.project {
            merged = merged.merge(project.clone());
        }

        merged.to_full_config()
    }

    /// Save configuration at a specific level
    ///
    /// # Errors
    ///
    /// Returns an error if serializing the config, creating directories,
    /// or writing the file fails.
    pub async fn save_at_level(config: &Config, level: ConfigLevel) -> Result<()> {
        let path = level.path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let contents = toml::to_string_pretty(config)
            .map_err(|e| Error::config(format!("Failed to serialize config: {}", e)))?;

        fs::write(&path, contents).await.map_err(|e| {
            Error::config(format!(
                "Failed to write {} config: {}",
                level.display_name(),
                e
            ))
        })?;

        info!(
            "Saved {} configuration to {}",
            level.display_name(),
            path.display()
        );
        Ok(())
    }

    /// Get configuration sources report
    pub fn sources_report(&self) -> String {
        let mut report = String::from("Configuration sources:\n");

        if self.system.is_some() {
            report.push_str(&format!("  ✓ System: /etc/ferrous-forge/config.toml\n"));
        } else {
            report.push_str("  ✗ System: Not found\n");
        }

        if self.user.is_some() {
            let path = ConfigLevel::User.path().unwrap_or_default();
            report.push_str(&format!("  ✓ User: {}\n", path.display()));
        } else {
            report.push_str("  ✗ User: Not found\n");
        }

        if self.project.is_some() {
            report.push_str("  ✓ Project: ./.ferrous-forge/config.toml\n");
        } else {
            report.push_str("  ✗ Project: Not found\n");
        }

        report
    }
}

/// Migrate from old single-file config to hierarchical system
///
/// # Errors
///
/// Returns an error if reading the old config, parsing it, saving the migrated
/// config, or renaming the backup file fails.
pub async fn migrate_config() -> Result<()> {
    let old_path = Config::config_file_path()?;

    if !old_path.exists() {
        info!("No old configuration to migrate");
        return Ok(());
    }

    info!("Migrating configuration from {}", old_path.display());

    // Load old config
    let old_contents = fs::read_to_string(&old_path)
        .await
        .map_err(|e| Error::config(format!("Failed to read old config: {}", e)))?;

    let config: Config = toml::from_str(&old_contents)
        .map_err(|e| Error::config(format!("Failed to parse old config: {}", e)))?;

    // Save to user level (most appropriate for migrated config)
    HierarchicalConfig::save_at_level(&config, ConfigLevel::User).await?;

    // Rename old config file to backup
    let backup_path = old_path.with_extension("toml.backup");
    fs::rename(&old_path, &backup_path)
        .await
        .map_err(|e| Error::config(format!("Failed to backup old config: {}", e)))?;

    info!(
        "Migration complete. Old config backed up to {}",
        backup_path.display()
    );
    Ok(())
}

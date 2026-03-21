//! Configuration sharing utilities for team-wide config export/import
//!
//! Provides functionality to export and import configuration at different levels,
//! enabling team-wide standard sharing across the organization.
//!
//! @task T018
//! @epic T014

use crate::config::hierarchy::{ConfigLevel, HierarchicalConfig, PartialConfig};
use crate::config::locking::HierarchicalLockManager;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

/// Shareable team configuration format
///
/// This structure represents a complete configuration snapshot that can be
/// exported from one level and imported to another, preserving both
/// configuration values and lock states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedConfig {
    /// Version of the shared config format
    pub version: String,
    /// Configuration level this was exported from
    pub exported_from: ConfigLevel,
    /// Username who exported the config
    pub exported_by: String,
    /// Timestamp of export (ISO 8601)
    pub exported_at: String,
    /// Description of this config (optional)
    pub description: Option<String>,
    /// The configuration values
    pub config: PartialConfig,
    /// Locked configuration keys
    pub locks: HashMap<String, LockExport>,
}

/// Lock entry for export (simplified from `LockEntry`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockExport {
    /// The locked value
    pub value: String,
    /// Reason for locking (for documentation)
    pub reason: String,
}

impl SharedConfig {
    /// Create a new shared config from a specific level
    ///
    /// # Errors
    ///
    /// Returns an error if the hierarchical configuration cannot be loaded.
    pub async fn from_level(level: ConfigLevel) -> Result<Self> {
        let hier_config = HierarchicalConfig::load().await?;
        let lock_manager = HierarchicalLockManager::load().await?;

        // Get the partial config for this level
        let partial = match level {
            ConfigLevel::System => hier_config.system.clone(),
            ConfigLevel::User => hier_config.user.clone(),
            ConfigLevel::Project => hier_config.project.clone(),
        };

        // If no config at this level, create empty partial
        let config = partial.unwrap_or_default();

        // Get locks at this level
        let locks = match level {
            ConfigLevel::System => lock_manager.get_locks_at_level(ConfigLevel::System).await?,
            ConfigLevel::User => lock_manager.get_locks_at_level(ConfigLevel::User).await?,
            ConfigLevel::Project => {
                lock_manager
                    .get_locks_at_level(ConfigLevel::Project)
                    .await?
            }
        };

        // Convert locks to export format
        let lock_exports: HashMap<String, LockExport> = locks
            .into_iter()
            .map(|(key, entry)| {
                (
                    key,
                    LockExport {
                        value: entry.value,
                        reason: entry.reason,
                    },
                )
            })
            .collect();

        Ok(Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            exported_from: level,
            exported_by: whoami::username(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            description: None,
            config,
            locks: lock_exports,
        })
    }

    /// Set a description for this shared config
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Export to TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize shared config: {}", e)))
    }

    /// Parse from TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str)
            .map_err(|e| Error::config(format!("Failed to parse shared config: {}", e)))
    }

    /// Export to a file
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails or the file cannot be written.
    pub async fn export_to_file(&self, path: &PathBuf) -> Result<()> {
        let contents = self.to_toml()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                Error::config(format!("Failed to create directory for export: {}", e))
            })?;
        }

        fs::write(path, contents)
            .await
            .map_err(|e| Error::config(format!("Failed to write shared config to file: {}", e)))?;

        info!("Exported configuration to {}", path.display());
        Ok(())
    }

    /// Import from a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn import_from_file(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(Error::config(format!(
                "Shared config file not found: {}",
                path.display()
            )));
        }

        let contents = fs::read_to_string(path)
            .await
            .map_err(|e| Error::config(format!("Failed to read shared config file: {}", e)))?;

        Self::from_toml(&contents)
    }

    /// Get a summary of this shared config for display
    pub fn summary(&self) -> String {
        let mut summary = format!("Shared Configuration\n");
        summary.push_str(&format!("  Version: {}\n", self.version));
        summary.push_str(&format!(
            "  Exported from: {} level\n",
            self.exported_from.display_name()
        ));
        summary.push_str(&format!("  Exported by: {}\n", self.exported_by));
        summary.push_str(&format!("  Exported at: {}\n", self.exported_at));
        if let Some(ref desc) = self.description {
            summary.push_str(&format!("  Description: {}\n", desc));
        }
        summary.push_str(&format!(
            "\n  Configuration entries: {}\n",
            self.count_config_entries()
        ));
        summary.push_str(&format!("  Locked keys: {}\n", self.locks.len()));
        summary
    }

    /// Count non-None configuration entries
    fn count_config_entries(&self) -> usize {
        let mut count = 0;
        if self.config.initialized.is_some() {
            count += 1;
        }
        if self.config.version.is_some() {
            count += 1;
        }
        if self.config.update_channel.is_some() {
            count += 1;
        }
        if self.config.auto_update.is_some() {
            count += 1;
        }
        if self.config.clippy_rules.is_some() {
            count += 1;
        }
        if self.config.max_file_lines.is_some() {
            count += 1;
        }
        if self.config.max_function_lines.is_some() {
            count += 1;
        }
        if self.config.required_edition.is_some() {
            count += 1;
        }
        if self.config.required_rust_version.is_some() {
            count += 1;
        }
        if self.config.ban_underscore_bandaid.is_some() {
            count += 1;
        }
        if self.config.require_documentation.is_some() {
            count += 1;
        }
        if self.config.custom_rules.is_some() {
            count += 1;
        }
        count
    }
}

/// Import options for merging shared configs
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Target level to import to
    pub target_level: ConfigLevel,
    /// Whether to overwrite existing values
    pub overwrite: bool,
    /// Whether to import locks as well
    pub import_locks: bool,
    /// Whether to require justification for overriding locks
    pub require_justification: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            target_level: ConfigLevel::Project,
            overwrite: true,
            import_locks: true,
            require_justification: true,
        }
    }
}

/// Import a shared configuration to a specific level
///
/// This merges the shared config into the existing configuration at the target level.
/// If locks are included, they will be applied at the target level as well.
///
/// # Errors
///
/// Returns an error if the import fails due to lock conflicts or file I/O errors.
pub async fn import_shared_config(
    shared: &SharedConfig,
    options: ImportOptions,
) -> Result<ImportReport> {
    let mut report = ImportReport::default();

    // Load existing config at target level
    let hier_config = HierarchicalConfig::load().await?;
    let mut lock_manager = HierarchicalLockManager::load().await?;

    // Get or create the partial config at target level
    let existing_partial = match options.target_level {
        ConfigLevel::System => hier_config.system.clone().unwrap_or_default(),
        ConfigLevel::User => hier_config.user.clone().unwrap_or_default(),
        ConfigLevel::Project => hier_config.project.clone().unwrap_or_default(),
    };

    // Merge the shared config into existing
    let merged_partial = if options.overwrite {
        existing_partial.merge(shared.config.clone())
    } else {
        // If not overwriting, shared config fills in gaps only
        shared.config.clone().merge(existing_partial)
    };

    // Check for lock conflicts before applying
    let mut conflicts = Vec::new();
    if options.require_justification {
        // Check if any values we're trying to set are locked at higher levels
        let locks = lock_manager.get_effective_locks();
        for key in shared.config.list_keys() {
            if let Some((level, entry)) = locks.get(&key) {
                // Check if this lock is at a higher level than our target
                if *level > options.target_level {
                    conflicts.push(LockConflict {
                        key: key.clone(),
                        locked_at: *level,
                        current_value: entry.value.clone(),
                        attempted_value: shared.config.get_value(&key).unwrap_or_default(),
                    });
                }
            }
        }
    }

    if !conflicts.is_empty() {
        report.conflicts = conflicts;
        return Ok(report);
    }

    // Save the merged config
    HierarchicalConfig::save_partial_at_level(&merged_partial, options.target_level).await?;
    report.config_imported = true;
    report.config_keys_updated = merged_partial.count_set_fields();

    // Import locks if requested
    if options.import_locks && !shared.locks.is_empty() {
        for (key, lock_export) in &shared.locks {
            // Check if already locked at this level
            if lock_manager
                .is_locked_at_level(key, options.target_level)
                .is_some()
            {
                report.locks_skipped.push(key.clone());
                continue;
            }

            // Create lock entry
            let entry = crate::config::locking::LockEntry::new(
                &lock_export.value,
                format!("Imported from shared config: {}", lock_export.reason),
                options.target_level,
            );

            lock_manager.lock_with_entry(key, entry).await?;
            report.locks_imported.push(key.clone());
        }
    }

    info!(
        "Imported {} config keys and {} locks to {} level",
        report.config_keys_updated,
        report.locks_imported.len(),
        options.target_level.display_name()
    );

    Ok(report)
}

/// Report of an import operation
#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    /// Whether the config was imported
    pub config_imported: bool,
    /// Number of configuration keys updated
    pub config_keys_updated: usize,
    /// Locks that were imported
    pub locks_imported: Vec<String>,
    /// Locks that were skipped (already existed)
    pub locks_skipped: Vec<String>,
    /// Lock conflicts that prevented import
    pub conflicts: Vec<LockConflict>,
}

/// A lock conflict during import
#[derive(Debug, Clone)]
pub struct LockConflict {
    /// The configuration key in conflict
    pub key: String,
    /// The level where the lock exists
    pub locked_at: ConfigLevel,
    /// The currently locked value
    pub current_value: String,
    /// The value we tried to set
    pub attempted_value: String,
}

/// Extension trait for `PartialConfig` to get list of keys
pub trait PartialConfigExt {
    /// List all keys that have values set
    fn list_keys(&self) -> Vec<String>;
    /// Get value for a key as string
    fn get_value(&self, key: &str) -> Option<String>;
    /// Count number of fields that are Some
    fn count_set_fields(&self) -> usize;
}

impl PartialConfigExt for PartialConfig {
    fn list_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        if self.initialized.is_some() {
            keys.push("initialized".to_string());
        }
        if self.version.is_some() {
            keys.push("version".to_string());
        }
        if self.update_channel.is_some() {
            keys.push("update_channel".to_string());
        }
        if self.auto_update.is_some() {
            keys.push("auto_update".to_string());
        }
        if self.clippy_rules.is_some() {
            keys.push("clippy_rules".to_string());
        }
        if self.max_file_lines.is_some() {
            keys.push("max_file_lines".to_string());
        }
        if self.max_function_lines.is_some() {
            keys.push("max_function_lines".to_string());
        }
        if self.required_edition.is_some() {
            keys.push("required_edition".to_string());
        }
        if self.required_rust_version.is_some() {
            keys.push("required_rust_version".to_string());
        }
        if self.ban_underscore_bandaid.is_some() {
            keys.push("ban_underscore_bandaid".to_string());
        }
        if self.require_documentation.is_some() {
            keys.push("require_documentation".to_string());
        }
        if self.custom_rules.is_some() {
            keys.push("custom_rules".to_string());
        }
        keys
    }

    fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "initialized" => self.initialized.map(|v| v.to_string()),
            "version" => self.version.clone(),
            "update_channel" => self.update_channel.clone(),
            "auto_update" => self.auto_update.map(|v| v.to_string()),
            "clippy_rules" => self.clippy_rules.as_ref().map(|v| format!("{:?}", v)),
            "max_file_lines" => self.max_file_lines.map(|v| v.to_string()),
            "max_function_lines" => self.max_function_lines.map(|v| v.to_string()),
            "required_edition" => self.required_edition.clone(),
            "required_rust_version" => self.required_rust_version.clone(),
            "ban_underscore_bandaid" => self.ban_underscore_bandaid.map(|v| v.to_string()),
            "require_documentation" => self.require_documentation.map(|v| v.to_string()),
            "custom_rules" => self.custom_rules.as_ref().map(|v| format!("{:?}", v)),
            _ => None,
        }
    }

    fn count_set_fields(&self) -> usize {
        self.list_keys().len()
    }
}

/// Extension trait for `HierarchicalLockManager` to get locks at specific level
pub trait HierarchicalLockManagerExt {
    /// Get all locks at a specific level
    #[allow(async_fn_in_trait)]
    async fn get_locks_at_level(
        &self,
        level: ConfigLevel,
    ) -> Result<HashMap<String, crate::config::locking::LockEntry>>;
    /// Lock with a pre-built entry
    #[allow(async_fn_in_trait)]
    async fn lock_with_entry(
        &mut self,
        key: &str,
        entry: crate::config::locking::LockEntry,
    ) -> Result<()>;
}

impl HierarchicalLockManagerExt for HierarchicalLockManager {
    async fn get_locks_at_level(
        &self,
        level: ConfigLevel,
    ) -> Result<HashMap<String, crate::config::locking::LockEntry>> {
        use crate::config::locking::LockedConfig;

        if let Some(locked) = LockedConfig::load_from_level(level).await? {
            Ok(locked.locks)
        } else {
            Ok(HashMap::new())
        }
    }

    async fn lock_with_entry(
        &mut self,
        key: &str,
        entry: crate::config::locking::LockEntry,
    ) -> Result<()> {
        use crate::config::locking::LockedConfig;

        let level = entry.level;

        // Load or create locks at this level
        let mut locks = LockedConfig::load_from_level(level)
            .await?
            .unwrap_or_default();
        locks.lock(key.to_string(), entry);
        locks.save_to_level(level).await?;

        Ok(())
    }
}

/// Extension trait for `HierarchicalConfig` to save partial config
pub trait HierarchicalConfigExt {
    /// Save a partial config at a specific level
    #[allow(async_fn_in_trait)]
    async fn save_partial_at_level(partial: &PartialConfig, level: ConfigLevel) -> Result<()>;
}

impl HierarchicalConfigExt for HierarchicalConfig {
    async fn save_partial_at_level(partial: &PartialConfig, level: ConfigLevel) -> Result<()> {
        use tokio::fs;

        let path = level.path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Convert partial to full config for serialization
        let full_config = partial.clone().to_full_config();
        let contents = toml::to_string_pretty(&full_config)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_config_serialization() {
        let config = PartialConfig {
            required_edition: Some("2024".to_string()),
            max_file_lines: Some(300),
            ..Default::default()
        };

        let shared = SharedConfig {
            version: "1.0.0".to_string(),
            exported_from: ConfigLevel::Project,
            exported_by: "test_user".to_string(),
            exported_at: "2024-01-01T00:00:00Z".to_string(),
            description: Some("Test config".to_string()),
            config,
            locks: HashMap::new(),
        };

        let toml = shared.to_toml().unwrap();
        assert!(toml.contains("version"));
        assert!(toml.contains("2024"));
    }

    #[test]
    fn test_partial_config_ext() {
        let partial = PartialConfig {
            required_edition: Some("2024".to_string()),
            max_file_lines: Some(300),
            ..Default::default()
        };

        let keys = partial.list_keys();
        assert!(keys.contains(&"required_edition".to_string()));
        assert!(keys.contains(&"max_file_lines".to_string()));
        assert_eq!(partial.count_set_fields(), 2);
    }
}

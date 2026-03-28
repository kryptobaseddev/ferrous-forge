//! Configuration locking system
//!
//! Provides mandatory locking mechanism for critical configuration values.
//! Once locked, values cannot be changed without explicit unlock with justification.
//!
//! @task T015
//! @epic T014

use crate::config::hierarchy::ConfigLevel;
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};

/// Audit logging for lock/unlock operations
pub mod audit_log;
/// Configuration change validator
pub mod validator;

pub use validator::ConfigValidator;

/// Configuration lock entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    /// The locked value
    pub value: String,
    /// When the lock was created
    pub locked_at: DateTime<Utc>,
    /// Who/what created the lock (user, system, etc.)
    pub locked_by: String,
    /// Reason for locking
    pub reason: String,
    /// Configuration level where lock is set
    pub level: ConfigLevel,
}

impl LockEntry {
    /// Create a new lock entry
    pub fn new(value: impl Into<String>, reason: impl Into<String>, level: ConfigLevel) -> Self {
        Self {
            value: value.into(),
            locked_at: Utc::now(),
            locked_by: whoami::username(),
            reason: reason.into(),
            level,
        }
    }
}

/// Locked configuration storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LockedConfig {
    /// Map of configuration keys to their lock entries
    pub locks: HashMap<String, LockEntry>,
    /// Version of the lock file format
    pub version: String,
}

impl LockedConfig {
    /// Create a new empty locked config
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
            version: "1.0.0".to_string(),
        }
    }

    /// Load locked configuration from a specific level
    ///
    /// # Errors
    ///
    /// Returns an error if reading or parsing the lock file fails.
    pub async fn load_from_level(level: ConfigLevel) -> Result<Option<Self>> {
        let path = Self::lock_file_path_for_level(level)?;

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path).await.map_err(|e| {
            Error::config(format!(
                "Failed to read {} lock file: {}",
                level.display_name(),
                e
            ))
        })?;

        let locked: LockedConfig = toml::from_str(&contents).map_err(|e| {
            Error::config(format!(
                "Failed to parse {} lock file: {}",
                level.display_name(),
                e
            ))
        })?;

        info!(
            "Loaded {} locks from {} level",
            locked.locks.len(),
            level.display_name()
        );
        Ok(Some(locked))
    }

    /// Save locked configuration to a specific level
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails or the file cannot be written.
    pub async fn save_to_level(&self, level: ConfigLevel) -> Result<()> {
        let path = Self::lock_file_path_for_level(level)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                Error::config(format!(
                    "Failed to create directory for {} lock file: {}",
                    level.display_name(),
                    e
                ))
            })?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize lock file: {}", e)))?;

        fs::write(&path, contents).await.map_err(|e| {
            Error::config(format!(
                "Failed to write {} lock file: {}",
                level.display_name(),
                e
            ))
        })?;

        info!(
            "Saved {} lock file to {}",
            level.display_name(),
            path.display()
        );
        Ok(())
    }

    /// Get the lock file path for a specific level
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be determined.
    pub fn lock_file_path_for_level(level: ConfigLevel) -> Result<PathBuf> {
        match level {
            ConfigLevel::System => Ok(PathBuf::from("/etc/ferrous-forge/locked.toml")),
            ConfigLevel::User => {
                let config_dir = dirs::config_dir()
                    .ok_or_else(|| Error::config("Could not find config directory"))?;
                Ok(config_dir.join("ferrous-forge").join("locked.toml"))
            }
            ConfigLevel::Project => Ok(PathBuf::from(".forge/locked.toml")),
        }
    }

    /// Check if a key is locked
    pub fn is_locked(&self, key: &str) -> bool {
        self.locks.contains_key(key)
    }

    /// Get lock entry for a key
    pub fn get_lock(&self, key: &str) -> Option<&LockEntry> {
        self.locks.get(key)
    }

    /// Lock a configuration key
    pub fn lock(&mut self, key: impl Into<String>, entry: LockEntry) {
        let key = key.into();
        self.locks.insert(key, entry);
    }

    /// Unlock a configuration key
    pub fn unlock(&mut self, key: &str) -> Option<LockEntry> {
        self.locks.remove(key)
    }

    /// List all locked keys
    pub fn list_locks(&self) -> Vec<(&String, &LockEntry)> {
        self.locks.iter().collect()
    }
}

/// Hierarchical lock manager that respects precedence
pub struct HierarchicalLockManager {
    /// System-level locks (lowest priority)
    system: Option<LockedConfig>,
    /// User-level locks
    user: Option<LockedConfig>,
    /// Project-level locks (highest priority)
    project: Option<LockedConfig>,
}

impl HierarchicalLockManager {
    /// Load locks from all levels
    ///
    /// # Errors
    ///
    /// Returns an error if reading or parsing any lock file fails.
    pub async fn load() -> Result<Self> {
        let system = LockedConfig::load_from_level(ConfigLevel::System).await?;
        let user = LockedConfig::load_from_level(ConfigLevel::User).await?;
        let project = LockedConfig::load_from_level(ConfigLevel::Project).await?;

        Ok(Self {
            system,
            user,
            project,
        })
    }

    /// Check if a key is locked at any level
    ///
    /// Returns the most specific lock (project > user > system)
    #[allow(clippy::collapsible_if)]
    pub fn is_locked(&self, key: &str) -> Option<(ConfigLevel, &LockEntry)> {
        // Check in order of precedence (highest first)
        if let Some(project) = &self.project {
            if let Some(entry) = project.get_lock(key) {
                return Some((ConfigLevel::Project, entry));
            }
        }

        if let Some(user) = &self.user {
            if let Some(entry) = user.get_lock(key) {
                return Some((ConfigLevel::User, entry));
            }
        }

        if let Some(system) = &self.system {
            if let Some(entry) = system.get_lock(key) {
                return Some((ConfigLevel::System, entry));
            }
        }

        None
    }

    /// Check if a specific level has a lock
    pub fn is_locked_at_level(&self, key: &str, level: ConfigLevel) -> Option<&LockEntry> {
        let locks = match level {
            ConfigLevel::System => self.system.as_ref(),
            ConfigLevel::User => self.user.as_ref(),
            ConfigLevel::Project => self.project.as_ref(),
        };

        locks.and_then(|l| l.get_lock(key))
    }

    /// Get all locks merged with proper precedence
    pub fn get_effective_locks(&self) -> HashMap<String, (ConfigLevel, LockEntry)> {
        let mut effective = HashMap::new();

        // Apply in order of precedence (lowest to highest)
        if let Some(system) = &self.system {
            for (key, entry) in &system.locks {
                effective.insert(key.clone(), (ConfigLevel::System, entry.clone()));
            }
        }

        if let Some(user) = &self.user {
            for (key, entry) in &user.locks {
                effective.insert(key.clone(), (ConfigLevel::User, entry.clone()));
            }
        }

        if let Some(project) = &self.project {
            for (key, entry) in &project.locks {
                effective.insert(key.clone(), (ConfigLevel::Project, entry.clone()));
            }
        }

        effective
    }

    /// Lock a key at a specific level
    ///
    /// # Errors
    ///
    /// Returns an error if saving the lock file fails.
    #[allow(clippy::collapsible_if)]
    pub async fn lock(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
        level: ConfigLevel,
    ) -> Result<()> {
        let key = key.into();
        let entry = LockEntry::new(value, reason, level);

        // Check if already locked at same or higher level
        if let Some((existing_level, _)) = self.is_locked(&key) {
            if existing_level >= level {
                warn!(
                    "Key '{}' is already locked at {} level",
                    key,
                    existing_level.display_name()
                );
                return Err(Error::config(format!(
                    "Key '{}' is already locked at {} level",
                    key,
                    existing_level.display_name()
                )));
            }
        }

        // Get or create the config for this level
        let locks = match level {
            ConfigLevel::System => &mut self.system,
            ConfigLevel::User => &mut self.user,
            ConfigLevel::Project => &mut self.project,
        };

        if locks.is_none() {
            *locks = Some(LockedConfig::new());
        }

        if let Some(config) = locks {
            config.lock(key.clone(), entry);
            config.save_to_level(level).await?;

            info!("Locked key '{}' at {} level", key, level.display_name());
        }

        Ok(())
    }

    /// Unlock a key at a specific level
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not locked at this level or if saving fails.
    pub async fn unlock(
        &mut self,
        key: &str,
        level: ConfigLevel,
        reason: &str,
    ) -> Result<LockEntry> {
        // Verify the lock exists at this exact level
        let locks = match level {
            ConfigLevel::System => &mut self.system,
            ConfigLevel::User => &mut self.user,
            ConfigLevel::Project => &mut self.project,
        };

        let config = locks.as_mut().ok_or_else(|| {
            Error::config(format!(
                "No locks defined at {} level",
                level.display_name()
            ))
        })?;

        let entry = config.unlock(key).ok_or_else(|| {
            Error::config(format!(
                "Key '{}' is not locked at {} level",
                key,
                level.display_name()
            ))
        })?;

        // Save the updated locks
        config.save_to_level(level).await?;

        info!(
            "Unlocked key '{}' at {} level. Reason: {}",
            key,
            level.display_name(),
            reason
        );

        // Audit log the unlock operation
        audit_log::log_unlock(key, &entry, level, reason).await?;

        Ok(entry)
    }

    /// Get lock status report
    pub fn status_report(&self) -> String {
        let mut report = String::from("Configuration Lock Status:\n\n");

        let effective = self.get_effective_locks();

        if effective.is_empty() {
            report.push_str("No configuration values are currently locked.\n");
            return report;
        }

        report.push_str(&format!("Total locked keys: {}\n\n", effective.len()));

        for (key, (level, entry)) in effective {
            report.push_str(&format!(
                "  {}: {} (locked at {} level)\n",
                key,
                entry.value,
                level.display_name()
            ));
            report.push_str(&format!(
                "    Locked by: {} at {}\n",
                entry.locked_by, entry.locked_at
            ));
            report.push_str(&format!("    Reason: {}\n\n", entry.reason));
        }

        report
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_entry_creation() {
        let entry = LockEntry::new("2024", "Required for project", ConfigLevel::Project);
        assert_eq!(entry.value, "2024");
        assert_eq!(entry.reason, "Required for project");
        assert_eq!(entry.level, ConfigLevel::Project);
    }

    #[test]
    fn test_locked_config_lock_unlock() {
        let mut config = LockedConfig::new();
        assert!(!config.is_locked("edition"));

        let entry = LockEntry::new("2024", "Required", ConfigLevel::User);
        config.lock("edition", entry.clone());
        assert!(config.is_locked("edition"));
        assert_eq!(config.get_lock("edition").unwrap().value, "2024");

        let removed = config.unlock("edition");
        assert!(removed.is_some());
        assert!(!config.is_locked("edition"));
    }

    #[test]
    fn test_locked_config_list_locks() {
        let mut config = LockedConfig::new();
        let entry1 = LockEntry::new("2024", "Required", ConfigLevel::User);
        let entry2 = LockEntry::new("1.85", "Required", ConfigLevel::User);

        config.lock("edition", entry1);
        config.lock("rust-version", entry2);

        let locks = config.list_locks();
        assert_eq!(locks.len(), 2);
    }

    #[test]
    fn test_hierarchical_lock_precedence() {
        let mut system = LockedConfig::new();
        let mut user = LockedConfig::new();
        let mut project = LockedConfig::new();

        system.lock(
            "edition",
            LockEntry::new("2021", "System default", ConfigLevel::System),
        );
        user.lock(
            "edition",
            LockEntry::new("2024", "User preference", ConfigLevel::User),
        );
        project.lock(
            "rust-version",
            LockEntry::new("1.88", "Project requirement", ConfigLevel::Project),
        );

        let manager = HierarchicalLockManager {
            system: Some(system),
            user: Some(user),
            project: Some(project),
        };

        // Project-level should override system/user for edition
        let result = manager.is_locked("edition");
        assert!(result.is_some());
        let (level, entry) = result.unwrap();
        assert_eq!(level, ConfigLevel::User);
        assert_eq!(entry.value, "2024");

        // Project should take precedence for rust-version
        let result = manager.is_locked("rust-version");
        assert!(result.is_some());
        let (level, entry) = result.unwrap();
        assert_eq!(level, ConfigLevel::Project);
        assert_eq!(entry.value, "1.88");
    }
}

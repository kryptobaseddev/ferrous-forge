//! Configuration management for Ferrous Forge

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Ferrous Forge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Whether Ferrous Forge has been initialized
    pub initialized: bool,
    /// Version of the configuration format
    pub version: String,
    /// Update channel (stable, beta, nightly)
    pub update_channel: String,
    /// Whether to automatically check for updates
    pub auto_update: bool,
    /// Custom clippy rules
    pub clippy_rules: Vec<String>,
    /// File size limit in lines
    pub max_file_lines: usize,
    /// Function size limit in lines
    pub max_function_lines: usize,
    /// Whether to enforce Edition 2024
    pub enforce_edition_2024: bool,
    /// Whether to ban underscore bandaid patterns
    pub ban_underscore_bandaid: bool,
    /// Whether to require documentation
    pub require_documentation: bool,
    /// Custom validation rules
    pub custom_rules: Vec<CustomRule>,
}

/// Custom validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Name of the rule
    pub name: String,
    /// Pattern to match (regex)
    pub pattern: String,
    /// Error message to display
    pub message: String,
    /// Whether this rule is enabled
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initialized: false,
            version: "0.1.0".to_string(),
            update_channel: "stable".to_string(),
            auto_update: true,
            clippy_rules: vec![
                "-D warnings".to_string(),
                "-D clippy::unwrap_used".to_string(),
                "-D clippy::expect_used".to_string(),
                "-D clippy::panic".to_string(),
                "-D clippy::unimplemented".to_string(),
                "-D clippy::todo".to_string(),
            ],
            max_file_lines: 300,
            max_function_lines: 50,
            enforce_edition_2024: true,
            ban_underscore_bandaid: true,
            require_documentation: true,
            custom_rules: vec![],
        }
    }
}

impl Config {
    /// Load configuration from file, or return default if not found
    pub async fn load_or_default() -> Result<Self> {
        match Self::load().await {
            Ok(config) => Ok(config),
            Err(_) => Ok(Self::default()),
        }
    }

    /// Load configuration from file
    pub async fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        let contents = fs::read_to_string(&config_path)
            .await
            .map_err(|e| Error::config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| Error::config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, contents)
            .await
            .map_err(|e| Error::config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Get the path to the configuration file
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| Error::config("Could not find config directory"))?;

        Ok(config_dir.join("ferrous-forge").join("config.toml"))
    }

    /// Get the path to the configuration directory
    pub fn config_dir_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| Error::config("Could not find config directory"))?;

        Ok(config_dir.join("ferrous-forge"))
    }

    /// Ensure configuration directories exist
    pub async fn ensure_directories(&self) -> Result<()> {
        let config_dir = Self::config_dir_path()?;
        fs::create_dir_all(&config_dir).await?;

        // Create subdirectories
        fs::create_dir_all(config_dir.join("templates")).await?;
        fs::create_dir_all(config_dir.join("rules")).await?;
        fs::create_dir_all(config_dir.join("backups")).await?;

        Ok(())
    }

    /// Check if Ferrous Forge is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Mark Ferrous Forge as initialized
    pub fn mark_initialized(&mut self) {
        self.initialized = true;
    }

    /// Get a configuration value by key
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "update_channel" => Some(self.update_channel.clone()),
            "auto_update" => Some(self.auto_update.to_string()),
            "max_file_lines" => Some(self.max_file_lines.to_string()),
            "max_function_lines" => Some(self.max_function_lines.to_string()),
            "enforce_edition_2024" => Some(self.enforce_edition_2024.to_string()),
            "ban_underscore_bandaid" => Some(self.ban_underscore_bandaid.to_string()),
            "require_documentation" => Some(self.require_documentation.to_string()),
            _ => None,
        }
    }

    /// Set a configuration value by key
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "update_channel" => {
                if !["stable", "beta", "nightly"].contains(&value) {
                    return Err(Error::config(
                        "Invalid update channel. Must be: stable, beta, or nightly",
                    ));
                }
                self.update_channel = value.to_string();
            }
            "auto_update" => {
                self.auto_update = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for auto_update"))?;
            }
            "max_file_lines" => {
                self.max_file_lines = value
                    .parse()
                    .map_err(|_| Error::config("Invalid number for max_file_lines"))?;
            }
            "max_function_lines" => {
                self.max_function_lines = value
                    .parse()
                    .map_err(|_| Error::config("Invalid number for max_function_lines"))?;
            }
            "enforce_edition_2024" => {
                self.enforce_edition_2024 = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for enforce_edition_2024"))?;
            }
            "ban_underscore_bandaid" => {
                self.ban_underscore_bandaid = value.parse().map_err(|_| {
                    Error::config("Invalid boolean value for ban_underscore_bandaid")
                })?;
            }
            "require_documentation" => {
                self.require_documentation = value.parse().map_err(|_| {
                    Error::config("Invalid boolean value for require_documentation")
                })?;
            }
            _ => return Err(Error::config(format!("Unknown configuration key: {}", key))),
        }

        Ok(())
    }

    /// List all configuration keys and values
    pub fn list(&self) -> Vec<(String, String)> {
        vec![
            ("update_channel".to_string(), self.update_channel.clone()),
            ("auto_update".to_string(), self.auto_update.to_string()),
            (
                "max_file_lines".to_string(),
                self.max_file_lines.to_string(),
            ),
            (
                "max_function_lines".to_string(),
                self.max_function_lines.to_string(),
            ),
            (
                "enforce_edition_2024".to_string(),
                self.enforce_edition_2024.to_string(),
            ),
            (
                "ban_underscore_bandaid".to_string(),
                self.ban_underscore_bandaid.to_string(),
            ),
            (
                "require_documentation".to_string(),
                self.require_documentation.to_string(),
            ),
        ]
    }

    /// Reset configuration to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
        self.initialized = true; // Keep initialized state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert!(!config.initialized);
        assert_eq!(config.version, "0.1.0");
        assert_eq!(config.update_channel, "stable");
        assert!(config.auto_update);
        assert_eq!(config.max_file_lines, 300);
        assert_eq!(config.max_function_lines, 50);
        assert!(config.enforce_edition_2024);
        assert!(config.ban_underscore_bandaid);
        assert!(config.require_documentation);
        assert!(!config.clippy_rules.is_empty());
        assert!(config.custom_rules.is_empty());
    }

    #[test]
    fn test_config_initialization() {
        let mut config = Config::default();

        assert!(!config.is_initialized());
        config.mark_initialized();
        assert!(config.is_initialized());
    }

    #[test]
    fn test_config_get() {
        let config = Config::default();

        assert_eq!(config.get("update_channel"), Some("stable".to_string()));
        assert_eq!(config.get("auto_update"), Some("true".to_string()));
        assert_eq!(config.get("max_file_lines"), Some("300".to_string()));
        assert_eq!(config.get("max_function_lines"), Some("50".to_string()));
        assert_eq!(config.get("enforce_edition_2024"), Some("true".to_string()));
        assert_eq!(
            config.get("ban_underscore_bandaid"),
            Some("true".to_string())
        );
        assert_eq!(
            config.get("require_documentation"),
            Some("true".to_string())
        );
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_config_set_update_channel() {
        let mut config = Config::default();

        // Valid channels
        assert!(config.set("update_channel", "stable").is_ok());
        assert_eq!(config.update_channel, "stable");

        assert!(config.set("update_channel", "beta").is_ok());
        assert_eq!(config.update_channel, "beta");

        assert!(config.set("update_channel", "nightly").is_ok());
        assert_eq!(config.update_channel, "nightly");

        // Invalid channel
        assert!(config.set("update_channel", "invalid").is_err());
    }

    #[test]
    fn test_config_set_boolean_values() {
        let mut config = Config::default();

        // Test auto_update
        assert!(config.set("auto_update", "false").is_ok());
        assert!(!config.auto_update);
        assert!(config.set("auto_update", "true").is_ok());
        assert!(config.auto_update);
        assert!(config.set("auto_update", "invalid").is_err());

        // Test enforce_edition_2024
        assert!(config.set("enforce_edition_2024", "false").is_ok());
        assert!(!config.enforce_edition_2024);

        // Test ban_underscore_bandaid
        assert!(config.set("ban_underscore_bandaid", "false").is_ok());
        assert!(!config.ban_underscore_bandaid);

        // Test require_documentation
        assert!(config.set("require_documentation", "false").is_ok());
        assert!(!config.require_documentation);
    }

    #[test]
    fn test_config_set_numeric_values() {
        let mut config = Config::default();

        // Test max_file_lines
        assert!(config.set("max_file_lines", "500").is_ok());
        assert_eq!(config.max_file_lines, 500);
        assert!(config.set("max_file_lines", "invalid").is_err());

        // Test max_function_lines
        assert!(config.set("max_function_lines", "100").is_ok());
        assert_eq!(config.max_function_lines, 100);
        assert!(config.set("max_function_lines", "invalid").is_err());
    }

    #[test]
    fn test_config_set_unknown_key() {
        let mut config = Config::default();
        assert!(config.set("unknown_key", "value").is_err());
    }

    #[test]
    fn test_config_list() {
        let config = Config::default();
        let list = config.list();

        assert_eq!(list.len(), 7);
        assert!(list
            .iter()
            .any(|(k, v)| k == "update_channel" && v == "stable"));
        assert!(list.iter().any(|(k, v)| k == "auto_update" && v == "true"));
        assert!(list
            .iter()
            .any(|(k, v)| k == "max_file_lines" && v == "300"));
        assert!(list
            .iter()
            .any(|(k, v)| k == "max_function_lines" && v == "50"));
    }

    #[test]
    fn test_config_reset() {
        let mut config = Config::default();
        config.mark_initialized();
        config.update_channel = "beta".to_string();
        config.auto_update = false;

        config.reset();

        assert!(config.is_initialized()); // Should keep initialized state
        assert_eq!(config.update_channel, "stable"); // Should reset to default
        assert!(config.auto_update); // Should reset to default
    }

    #[test]
    fn test_custom_rule() {
        let rule = CustomRule {
            name: "test_rule".to_string(),
            pattern: r"test_.*".to_string(),
            message: "Test message".to_string(),
            enabled: true,
        };

        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.pattern, r"test_.*");
        assert_eq!(rule.message, "Test message");
        assert!(rule.enabled);
    }

    // Note: Tests that would require environment variable manipulation are excluded
    // because this crate forbids unsafe code. In practice, the save/load functionality
    // would be tested in integration tests where unsafe code restrictions don't apply.

    #[test]
    fn test_config_file_path() {
        let result = Config::config_file_path();
        assert!(result.is_ok());
        let path = result.expect("Should get config file path");
        assert!(path.to_string_lossy().contains("ferrous-forge"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_config_dir_path() {
        let result = Config::config_dir_path();
        assert!(result.is_ok());
        let path = result.expect("Should get config dir path");
        assert!(path.to_string_lossy().contains("ferrous-forge"));
    }

    // Property-based tests using proptest
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_config_get_set_roundtrip(
                channel in prop::sample::select(vec!["stable", "beta", "nightly"]),
                auto_update in any::<bool>(),
                max_file_lines in 1usize..10000,
                max_function_lines in 1usize..1000,
            ) {
                let mut config = Config::default();

                prop_assert!(config.set("update_channel", &channel).is_ok());
                prop_assert_eq!(config.get("update_channel"), Some(channel.to_string()));

                prop_assert!(config.set("auto_update", &auto_update.to_string()).is_ok());
                prop_assert_eq!(config.get("auto_update"), Some(auto_update.to_string()));

                prop_assert!(config.set("max_file_lines", &max_file_lines.to_string()).is_ok());
                prop_assert_eq!(config.get("max_file_lines"), Some(max_file_lines.to_string()));

                prop_assert!(config.set("max_function_lines", &max_function_lines.to_string()).is_ok());
                prop_assert_eq!(config.get("max_function_lines"), Some(max_function_lines.to_string()));
            }
        }
    }
}

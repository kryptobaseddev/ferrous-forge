//! Partial configuration for hierarchical merging

use super::ConfigLevel;
use crate::config::{Config, CustomRule};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::debug;

/// Partial configuration that allows optional fields for merging
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialConfig {
    /// Whether Ferrous Forge has been initialized
    pub initialized: Option<bool>,
    /// Version of the configuration format
    pub version: Option<String>,
    /// Update channel (stable, beta, nightly)
    pub update_channel: Option<String>,
    /// Whether to automatically check for updates
    pub auto_update: Option<bool>,
    /// Custom clippy rules
    pub clippy_rules: Option<Vec<String>>,
    /// File size limit in lines
    pub max_file_lines: Option<usize>,
    /// Function size limit in lines
    pub max_function_lines: Option<usize>,
    /// Whether to enforce Edition 2024
    pub enforce_edition_2024: Option<bool>,
    /// Whether to ban underscore bandaid patterns
    pub ban_underscore_bandaid: Option<bool>,
    /// Whether to require documentation
    pub require_documentation: Option<bool>,
    /// Custom validation rules
    pub custom_rules: Option<Vec<CustomRule>>,
}

impl PartialConfig {
    /// Load partial config from a specific level
    pub async fn load_from_level(level: ConfigLevel) -> Result<Option<Self>> {
        let path = level.path()?;

        if !path.exists() {
            debug!(
                "No config found at {} level: {}",
                level.display_name(),
                path.display()
            );
            return Ok(None);
        }

        let contents = fs::read_to_string(&path).await.map_err(|e| {
            Error::config(format!(
                "Failed to read {} config: {}",
                level.display_name(),
                e
            ))
        })?;

        let partial: PartialConfig = toml::from_str(&contents).map_err(|e| {
            Error::config(format!(
                "Failed to parse {} config: {}",
                level.display_name(),
                e
            ))
        })?;

        tracing::info!(
            "Loaded {} configuration from {}",
            level.display_name(),
            path.display()
        );
        Ok(Some(partial))
    }

    /// Merge another partial config into this one (other takes precedence)
    pub fn merge(mut self, other: PartialConfig) -> Self {
        if other.initialized.is_some() {
            self.initialized = other.initialized;
        }
        if other.version.is_some() {
            self.version = other.version;
        }
        if other.update_channel.is_some() {
            self.update_channel = other.update_channel;
        }
        if other.auto_update.is_some() {
            self.auto_update = other.auto_update;
        }
        if other.clippy_rules.is_some() {
            self.clippy_rules = other.clippy_rules;
        }
        if other.max_file_lines.is_some() {
            self.max_file_lines = other.max_file_lines;
        }
        if other.max_function_lines.is_some() {
            self.max_function_lines = other.max_function_lines;
        }
        if other.enforce_edition_2024.is_some() {
            self.enforce_edition_2024 = other.enforce_edition_2024;
        }
        if other.ban_underscore_bandaid.is_some() {
            self.ban_underscore_bandaid = other.ban_underscore_bandaid;
        }
        if other.require_documentation.is_some() {
            self.require_documentation = other.require_documentation;
        }
        if other.custom_rules.is_some() {
            self.custom_rules = other.custom_rules;
        }
        self
    }

    /// Convert to full config, using defaults for missing values
    pub fn to_full_config(self) -> Config {
        let default = Config::default();
        Config {
            initialized: self.initialized.unwrap_or(default.initialized),
            version: self.version.unwrap_or(default.version),
            update_channel: self.update_channel.unwrap_or(default.update_channel),
            auto_update: self.auto_update.unwrap_or(default.auto_update),
            clippy_rules: self.clippy_rules.unwrap_or(default.clippy_rules),
            max_file_lines: self.max_file_lines.unwrap_or(default.max_file_lines),
            max_function_lines: self
                .max_function_lines
                .unwrap_or(default.max_function_lines),
            enforce_edition_2024: self
                .enforce_edition_2024
                .unwrap_or(default.enforce_edition_2024),
            ban_underscore_bandaid: self
                .ban_underscore_bandaid
                .unwrap_or(default.ban_underscore_bandaid),
            require_documentation: self
                .require_documentation
                .unwrap_or(default.require_documentation),
            custom_rules: self.custom_rules.unwrap_or(default.custom_rules),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_config_merge() {
        let base = PartialConfig {
            max_file_lines: Some(300),
            max_function_lines: Some(50),
            ..Default::default()
        };

        let override_config = PartialConfig {
            max_file_lines: Some(400),
            enforce_edition_2024: Some(false),
            ..Default::default()
        };

        let merged = base.merge(override_config);
        assert_eq!(merged.max_file_lines, Some(400));
        assert_eq!(merged.max_function_lines, Some(50));
        assert_eq!(merged.enforce_edition_2024, Some(false));
    }

    #[test]
    fn test_partial_to_full_config() {
        let partial = PartialConfig {
            max_file_lines: Some(500),
            ..Default::default()
        };

        let full = partial.to_full_config();
        assert_eq!(full.max_file_lines, 500);
        assert_eq!(full.max_function_lines, 50); // Default value
    }
}

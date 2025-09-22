//! Configuration operations (get, set, list, etc.)

use super::types::Config;
use crate::{Error, Result};

impl Config {
    /// Check if configuration has been initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Mark configuration as initialized
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
            "update_channel" => self.set_update_channel(value)?,
            "auto_update" => self.set_auto_update(value)?,
            "max_file_lines" => self.set_max_file_lines(value)?,
            "max_function_lines" => self.set_max_function_lines(value)?,
            "enforce_edition_2024" => self.set_enforce_edition_2024(value)?,
            "ban_underscore_bandaid" => self.set_ban_underscore_bandaid(value)?,
            "require_documentation" => self.set_require_documentation(value)?,
            _ => return Err(Error::config(format!("Unknown configuration key: {}", key))),
        }
        Ok(())
    }

    /// List all configuration keys and values
    pub fn list(&self) -> Vec<(String, String)> {
        vec![
            ("update_channel".to_string(), self.update_channel.clone()),
            ("auto_update".to_string(), self.auto_update.to_string()),
            ("max_file_lines".to_string(), self.max_file_lines.to_string()),
            ("max_function_lines".to_string(), self.max_function_lines.to_string()),
            ("enforce_edition_2024".to_string(), self.enforce_edition_2024.to_string()),
            ("ban_underscore_bandaid".to_string(), self.ban_underscore_bandaid.to_string()),
            ("require_documentation".to_string(), self.require_documentation.to_string()),
            ("clippy_rules".to_string(), format!("{:?}", self.clippy_rules)),
            ("custom_rules_count".to_string(), self.custom_rules.len().to_string()),
        ]
    }

    /// Reset configuration to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    // Helper methods for setting individual values
    fn set_update_channel(&mut self, value: &str) -> Result<()> {
        if !["stable", "beta", "nightly"].contains(&value) {
            return Err(Error::config(
                "Invalid update channel. Must be: stable, beta, or nightly",
            ));
        }
        self.update_channel = value.to_string();
        Ok(())
    }

    fn set_auto_update(&mut self, value: &str) -> Result<()> {
        self.auto_update = value
            .parse()
            .map_err(|_| Error::config("Invalid boolean value for auto_update"))?;
        Ok(())
    }

    fn set_max_file_lines(&mut self, value: &str) -> Result<()> {
        self.max_file_lines = value
            .parse()
            .map_err(|_| Error::config("Invalid number for max_file_lines"))?;
        Ok(())
    }

    fn set_max_function_lines(&mut self, value: &str) -> Result<()> {
        self.max_function_lines = value
            .parse()
            .map_err(|_| Error::config("Invalid number for max_function_lines"))?;
        Ok(())
    }

    fn set_enforce_edition_2024(&mut self, value: &str) -> Result<()> {
        self.enforce_edition_2024 = value
            .parse()
            .map_err(|_| Error::config("Invalid boolean value for enforce_edition_2024"))?;
        Ok(())
    }

    fn set_ban_underscore_bandaid(&mut self, value: &str) -> Result<()> {
        self.ban_underscore_bandaid = value.parse().map_err(|_| {
            Error::config("Invalid boolean value for ban_underscore_bandaid")
        })?;
        Ok(())
    }

    fn set_require_documentation(&mut self, value: &str) -> Result<()> {
        self.require_documentation = value.parse().map_err(|_| {
            Error::config("Invalid boolean value for require_documentation")
        })?;
        Ok(())
    }
}
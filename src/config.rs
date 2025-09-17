//! Configuration management for Ferrous Forge

use crate::{Result, Error};
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
        let contents = fs::read_to_string(&config_path).await
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
        
        fs::write(&config_path, contents).await
            .map_err(|e| Error::config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }

    /// Get the path to the configuration file
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| Error::config("Could not find config directory"))?;
        
        Ok(config_dir.join("ferrous-forge").join("config.toml"))
    }

    /// Get the path to the configuration directory
    pub fn config_dir_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| Error::config("Could not find config directory"))?;
        
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
                    return Err(Error::config("Invalid update channel. Must be: stable, beta, or nightly"));
                }
                self.update_channel = value.to_string();
            }
            "auto_update" => {
                self.auto_update = value.parse()
                    .map_err(|_| Error::config("Invalid boolean value for auto_update"))?;
            }
            "max_file_lines" => {
                self.max_file_lines = value.parse()
                    .map_err(|_| Error::config("Invalid number for max_file_lines"))?;
            }
            "max_function_lines" => {
                self.max_function_lines = value.parse()
                    .map_err(|_| Error::config("Invalid number for max_function_lines"))?;
            }
            "enforce_edition_2024" => {
                self.enforce_edition_2024 = value.parse()
                    .map_err(|_| Error::config("Invalid boolean value for enforce_edition_2024"))?;
            }
            "ban_underscore_bandaid" => {
                self.ban_underscore_bandaid = value.parse()
                    .map_err(|_| Error::config("Invalid boolean value for ban_underscore_bandaid"))?;
            }
            "require_documentation" => {
                self.require_documentation = value.parse()
                    .map_err(|_| Error::config("Invalid boolean value for require_documentation"))?;
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
            ("max_file_lines".to_string(), self.max_file_lines.to_string()),
            ("max_function_lines".to_string(), self.max_function_lines.to_string()),
            ("enforce_edition_2024".to_string(), self.enforce_edition_2024.to_string()),
            ("ban_underscore_bandaid".to_string(), self.ban_underscore_bandaid.to_string()),
            ("require_documentation".to_string(), self.require_documentation.to_string()),
        ]
    }

    /// Reset configuration to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
        self.initialized = true; // Keep initialized state
    }
}
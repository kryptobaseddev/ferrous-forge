//! Configuration file I/O operations

use super::types::Config;
use crate::{Error, Result};
use std::path::PathBuf;
use tokio::fs;

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
        Ok(())
    }
}

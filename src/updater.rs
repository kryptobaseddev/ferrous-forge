//! Self-update system for Ferrous Forge
//!
//! This module handles automatic updates of the Ferrous Forge binary and
//! configuration rules from remote sources.

use crate::{Result, Error};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Update channels available for Ferrous Forge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateChannel {
    /// Stable releases (recommended)
    Stable,
    /// Beta releases with new features
    Beta,
    /// Nightly builds with latest changes
    Nightly,
}

impl std::str::FromStr for UpdateChannel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "stable" => Ok(Self::Stable),
            "beta" => Ok(Self::Beta),
            "nightly" => Ok(Self::Nightly),
            _ => Err(Error::update("Invalid update channel")),
        }
    }
}

impl std::fmt::Display for UpdateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Beta => write!(f, "beta"),
            Self::Nightly => write!(f, "nightly"),
        }
    }
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Version of the update
    pub version: Version,
    /// Update channel
    pub channel: UpdateChannel,
    /// Release notes/changelog
    pub changelog: String,
    /// Download URL for the binary
    pub download_url: String,
    /// SHA256 checksum for verification
    pub checksum: String,
    /// Whether this is a security update
    pub is_security_update: bool,
}

/// Update manager for Ferrous Forge
pub struct UpdateManager {
    /// Current version
    current_version: Version,
    /// Update channel to use
    channel: UpdateChannel,
    /// HTTP client for downloads
    client: reqwest::Client,
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new(channel: UpdateChannel) -> Result<Self> {
        let current_version = Version::parse(crate::VERSION)
            .map_err(|e| Error::update(format!("Invalid current version: {}", e)))?;

        let client = reqwest::Client::builder()
            .user_agent(format!("ferrous-forge/{}", crate::VERSION))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::update(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            current_version,
            channel,
            client,
        })
    }

    /// Check if an update is available
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        let url = self.get_releases_url();
        
        tracing::info!("Checking for updates from: {}", url);
        
        // For now, return None since we don't have a real update server
        // TODO: Implement actual update checking
        Ok(None)
    }

    /// Download and install an update
    pub async fn install_update(&self, update_info: &UpdateInfo) -> Result<()> {
        tracing::info!("Installing update to version {}", update_info.version);

        // TODO: Implement actual update installation
        // 1. Download the binary
        // 2. Verify checksum
        // 3. Replace current binary (using self_update crate)
        // 4. Update configuration if needed

        Err(Error::update("Update installation not yet implemented"))
    }

    /// Update only the rules/configuration without updating the binary
    pub async fn update_rules(&self) -> Result<()> {
        tracing::info!("Updating rules for channel: {}", self.channel);

        // TODO: Implement rules update
        // 1. Fetch latest clippy.toml from repository
        // 2. Fetch latest standards configuration
        // 3. Update local files
        // 4. Validate new configuration

        Ok(())
    }

    /// Get the URL for checking releases based on the channel
    fn get_releases_url(&self) -> String {
        let base_url = "https://api.github.com/repos/yourusername/ferrous-forge";
        
        match self.channel {
            UpdateChannel::Stable => format!("{}/releases/latest", base_url),
            UpdateChannel::Beta => format!("{}/releases?prerelease=true", base_url),
            UpdateChannel::Nightly => format!("{}/actions/artifacts", base_url),
        }
    }

    /// Create a backup of the current installation
    async fn create_backup(&self) -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir_path()?;
        let backup_dir = config_dir.join("backups");
        let backup_path = backup_dir.join(format!("backup-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S")));

        tokio::fs::create_dir_all(&backup_path).await?;

        // Backup current binary
        let current_exe = std::env::current_exe()
            .map_err(|e| Error::update(format!("Failed to get current executable path: {}", e)))?;
        
        let backup_exe = backup_path.join("ferrous-forge");
        tokio::fs::copy(&current_exe, &backup_exe).await?;

        // Backup configuration
        let config_file = crate::config::Config::config_file_path()?;
        if config_file.exists() {
            let backup_config = backup_path.join("config.toml");
            tokio::fs::copy(&config_file, &backup_config).await?;
        }

        tracing::info!("Created backup at: {}", backup_path.display());
        Ok(backup_path)
    }

    /// Restore from a backup
    pub async fn restore_backup(&self, backup_path: &std::path::Path) -> Result<()> {
        if !backup_path.exists() {
            return Err(Error::update("Backup path does not exist"));
        }

        let backup_exe = backup_path.join("ferrous-forge");
        let backup_config = backup_path.join("config.toml");

        // Restore binary
        if backup_exe.exists() {
            let current_exe = std::env::current_exe()
                .map_err(|e| Error::update(format!("Failed to get current executable path: {}", e)))?;
            
            tokio::fs::copy(&backup_exe, &current_exe).await?;
            tracing::info!("Restored binary from backup");
        }

        // Restore configuration
        if backup_config.exists() {
            let config_file = crate::config::Config::config_file_path()?;
            tokio::fs::copy(&backup_config, &config_file).await?;
            tracing::info!("Restored configuration from backup");
        }

        Ok(())
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let config_dir = crate::config::Config::config_dir_path()?;
        let backup_dir = config_dir.join("backups");

        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();
        let mut entries = tokio::fs::read_dir(&backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_dir() {
                backups.push(entry.path());
            }
        }

        backups.sort();
        Ok(backups)
    }
}

/// Check if automatic updates are enabled and perform update check
pub async fn check_auto_update() -> Result<()> {
    let config = crate::config::Config::load_or_default().await?;
    
    if !config.auto_update {
        return Ok(());
    }

    let channel = config.update_channel.parse::<UpdateChannel>()?;
    let updater = UpdateManager::new(channel)?;

    if let Some(update_info) = updater.check_for_updates().await? {
        tracing::info!("Update available: {}", update_info.version);
        
        if update_info.is_security_update {
            // Auto-install security updates
            updater.install_update(&update_info).await?;
            tracing::warn!("Security update installed automatically");
        } else {
            // Just notify about regular updates
            tracing::info!("Update available. Run 'ferrous-forge update' to install.");
        }
    }

    Ok(())
}
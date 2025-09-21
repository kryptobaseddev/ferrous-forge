//! Self-update system for Ferrous Forge
//!
//! This module handles automatic updates of the Ferrous Forge binary and
//! configuration rules from remote sources.

use crate::{Error, Result};
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
    _current_version: Version,
    /// Update channel to use
    channel: UpdateChannel,
    /// HTTP client for downloads
    _client: reqwest::Client,
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
            _current_version: current_version,
            channel,
            _client: client,
        })
    }

    /// Check if an update is available
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        let url = self.get_releases_url();

        tracing::info!("Checking for updates from: {}", url);

        let response = self
            ._client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", format!("ferrous-forge/{}", crate::VERSION))
            .send()
            .await
            .map_err(|e| Error::update(format!("Failed to fetch releases: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::update(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| Error::update(format!("Failed to read response body: {}", e)))?;

        match self.channel {
            UpdateChannel::Stable => self.parse_latest_release(&body).await,
            UpdateChannel::Beta => self.parse_prerelease(&body).await,
            UpdateChannel::Nightly => self.parse_nightly_build().await,
        }
    }

    /// Download and install an update
    pub async fn install_update(&self, update_info: &UpdateInfo) -> Result<()> {
        tracing::info!("Installing update to version {}", update_info.version);

        // Create backup before update
        let backup_path = self._create_backup().await?;
        tracing::info!("Created backup at: {}", backup_path.display());

        #[cfg(feature = "update-system")]
        {
            // Download the binary from the GitHub release
            tracing::info!("Downloading update from: {}", update_info.download_url);

            let response = self
                ._client
                .get(&update_info.download_url)
                .send()
                .await
                .map_err(|e| Error::update(format!("Failed to download update: {}", e)))?;

            if !response.status().is_success() {
                return Err(Error::update(format!(
                    "Download failed with status: {}",
                    response.status()
                )));
            }

            let binary_data = response
                .bytes()
                .await
                .map_err(|e| Error::update(format!("Failed to read binary data: {}", e)))?;

            // Get current executable path
            let current_exe = std::env::current_exe().map_err(|e| {
                Error::update(format!("Failed to get current executable path: {}", e))
            })?;

            // Create a temporary file for the new binary
            let temp_path = current_exe.with_extension("new");
            tokio::fs::write(&temp_path, &binary_data)
                .await
                .map_err(|e| Error::update(format!("Failed to write new binary: {}", e)))?;

            // Make it executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = tokio::fs::metadata(&temp_path).await?.permissions();
                perms.set_mode(0o755);
                tokio::fs::set_permissions(&temp_path, perms).await?;
            }

            // Replace the current binary
            tokio::fs::rename(&temp_path, &current_exe)
                .await
                .map_err(|e| Error::update(format!("Failed to replace binary: {}", e)))?;

            tracing::info!("Successfully updated to version {}", update_info.version);
            Ok(())
        }

        #[cfg(not(feature = "update-system"))]
        {
            Err(Error::update("Update system disabled at compile time"))
        }
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
    async fn _create_backup(&self) -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir_path()?;
        let backup_dir = config_dir.join("backups");
        let backup_path = backup_dir.join(format!(
            "backup-{}",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        ));

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
            let current_exe = std::env::current_exe().map_err(|e| {
                Error::update(format!("Failed to get current executable path: {}", e))
            })?;

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

    /// Parse latest stable release from GitHub API response
    async fn parse_latest_release(&self, response_body: &str) -> Result<Option<UpdateInfo>> {
        #[derive(Deserialize)]
        struct GitHubRelease {
            tag_name: String,
            name: String,
            body: String,
            prerelease: bool,
            assets: Vec<GitHubAsset>,
        }

        #[derive(Deserialize)]
        struct GitHubAsset {
            name: String,
            browser_download_url: String,
        }

        let release: GitHubRelease = serde_json::from_str(response_body)
            .map_err(|e| Error::update(format!("Failed to parse GitHub release: {}", e)))?;

        if release.prerelease {
            return Ok(None); // Skip prereleases for stable channel
        }

        let version_str = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name);
        let version = Version::parse(version_str)
            .map_err(|e| Error::update(format!("Invalid version format: {}", e)))?;

        // Only suggest update if newer than current version
        if version <= self._current_version {
            return Ok(None);
        }

        // Find the binary asset for current platform
        let platform_suffix = self.get_platform_suffix();
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name.contains(&platform_suffix))
            .ok_or_else(|| {
                Error::update(format!("No binary found for platform: {}", platform_suffix))
            })?;

        Ok(Some(UpdateInfo {
            version,
            channel: self.channel.clone(),
            changelog: release.body,
            download_url: asset.browser_download_url.clone(),
            checksum: String::new(), // GitHub doesn't provide checksums in API
            is_security_update: release.name.to_lowercase().contains("security"),
        }))
    }

    /// Parse prerelease from GitHub API response
    async fn parse_prerelease(&self, response_body: &str) -> Result<Option<UpdateInfo>> {
        #[derive(Deserialize)]
        struct GitHubReleases {
            #[serde(flatten)]
            _releases: Vec<serde_json::Value>,
        }

        let releases: Vec<serde_json::Value> = serde_json::from_str(response_body)
            .map_err(|e| Error::update(format!("Failed to parse GitHub releases: {}", e)))?;

        // Find the latest prerelease
        for release_value in releases {
            if let Ok(release) = serde_json::from_value::<serde_json::Value>(release_value) {
                if release["prerelease"].as_bool().unwrap_or(false) {
                    return self
                        .parse_latest_release(&serde_json::to_string(&release)?)
                        .await;
                }
            }
        }

        Ok(None)
    }

    /// Parse nightly build from GitHub Actions artifacts
    async fn parse_nightly_build(&self) -> Result<Option<UpdateInfo>> {
        // Nightly builds would come from GitHub Actions artifacts
        // This is more complex and would require authentication
        // For now, return None as nightly builds aren't implemented
        tracing::warn!("Nightly builds not yet supported");
        Ok(None)
    }

    /// Get platform-specific binary suffix
    fn get_platform_suffix(&self) -> String {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        match (os, arch) {
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),
            ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
            ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
            ("windows", "x86_64") => "x86_64-pc-windows-msvc.exe".to_string(),
            _ => format!("{}-{}", arch, os),
        }
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

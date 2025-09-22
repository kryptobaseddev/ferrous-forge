//! Update manager implementation

use super::{
    github,
    types::{UpdateChannel, UpdateInfo, UpdateManager},
};
use crate::{Error, Result};
use console::style;
use semver::Version;
use std::path::PathBuf;
use tokio::fs;

impl UpdateManager {
    /// Create a new update manager
    pub fn new(channel: UpdateChannel) -> Result<Self> {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|e| Error::config(format!("Invalid current version: {}", e)))?;

        let binary_path = std::env::current_exe()
            .map_err(|e| Error::io(format!("Failed to get current executable path: {}", e)))?;

        Ok(Self {
            current_version,
            channel,
            binary_path,
        })
    }

    /// Check for available updates
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        println!("🔍 Checking for updates on {} channel...", self.channel);

        github::fetch_update_info(&self.current_version, &self.channel).await
    }

    /// Perform the actual update
    pub async fn perform_update(&self, update_info: &UpdateInfo) -> Result<()> {
        println!(
            "📥 Downloading update {} ({})",
            update_info.version,
            format_file_size(update_info.size)
        );

        // Download the new binary
        let temp_path = self.download_update(update_info).await?;

        // Verify the download if hash is provided
        if let Some(expected_hash) = &update_info.sha256 {
            self.verify_download(&temp_path, expected_hash).await?;
        }

        // Replace the current binary
        self.replace_binary(&temp_path).await?;

        println!("✅ Update completed successfully!");
        println!("🔄 Please restart Ferrous Forge to use the new version.");

        Ok(())
    }

    /// Download update to temporary location
    async fn download_update(&self, update_info: &UpdateInfo) -> Result<PathBuf> {
        let client = reqwest::Client::new();
        let response = client
            .get(&update_info.download_url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to download update: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::network(format!(
                "Download failed: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::network(format!("Failed to read download: {}", e)))?;

        // Create temporary file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("ferrous-forge-{}", update_info.version));

        fs::write(&temp_path, bytes)
            .await
            .map_err(|e| Error::io(format!("Failed to write temporary file: {}", e)))?;

        Ok(temp_path)
    }

    /// Verify downloaded file hash
    async fn verify_download(&self, path: &PathBuf, expected_hash: &str) -> Result<()> {
        let contents = fs::read(path)
            .await
            .map_err(|e| Error::io(format!("Failed to read downloaded file: {}", e)))?;

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let actual_hash = format!("{:x}", hasher.finalize());

        if actual_hash != expected_hash {
            return Err(Error::security(format!(
                "Hash verification failed. Expected: {}, Got: {}",
                expected_hash, actual_hash
            )));
        }

        Ok(())
    }

    /// Replace current binary with updated version
    async fn replace_binary(&self, temp_path: &PathBuf) -> Result<()> {
        // Create backup of current binary
        let backup_path = self.binary_path.with_extension("backup");
        fs::copy(&self.binary_path, &backup_path)
            .await
            .map_err(|e| Error::io(format!("Failed to create backup: {}", e)))?;

        // Replace with new binary
        fs::copy(temp_path, &self.binary_path).await.map_err(|e| {
            // Try to restore backup on failure
            // Attempt to restore backup, ignore errors since we're already in error state
            drop(std::fs::copy(&backup_path, &self.binary_path));
            Error::io(format!("Failed to replace binary: {}", e))
        })?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.binary_path)
                .await
                .map_err(|e| Error::io(format!("Failed to get file metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&self.binary_path, perms)
                .await
                .map_err(|e| Error::io(format!("Failed to set permissions: {}", e)))?;
        }

        // Clean up temporary file and backup
        // Clean up temporary files, ignore errors
        drop(fs::remove_file(temp_path).await);
        drop(fs::remove_file(&backup_path).await);

        Ok(())
    }

    /// Interactive update prompt
    pub async fn prompt_for_update(&self, update_info: &UpdateInfo) -> Result<bool> {
        println!();
        println!("{}", style("🆕 Update Available!").green().bold());
        println!("Current version: {}", self.current_version);
        println!("Latest version:  {}", update_info.version);
        println!();

        if !update_info.notes.is_empty() {
            println!("📝 Release Notes:");
            println!("{}", update_info.notes);
            println!();
        }

        print!("Would you like to update now? [y/N]: ");
        std::io::Write::flush(&mut std::io::stdout())
            .map_err(|e| Error::io(format!("Failed to flush stdout: {}", e)))?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| Error::io(format!("Failed to read input: {}", e)))?;

        Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
    }
}

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    const THRESHOLD: u64 = 1024;

    if bytes < THRESHOLD {
        return format!("{} B", bytes);
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

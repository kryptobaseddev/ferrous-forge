//! Rustup integration for toolchain management
//!
//! Provides commands to check, update, and manage Rust toolchains via rustup.
//! Enforces minimum/maximum versions set in locked config.
//!
//! @task T020
//! @epic T014

use crate::config::locking::HierarchicalLockManager;
use crate::rust_version::RustVersion;
use crate::rust_version::detector::{
    get_active_toolchain, get_installed_toolchains, is_rustup_available,
};
use crate::{Error, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info};

/// Toolchain channel types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolchainChannel {
    /// Stable releases
    Stable,
    /// Beta releases
    Beta,
    /// Nightly builds
    Nightly,
    /// Specific version (e.g., "1.70.0")
    Version(String),
    /// Custom toolchain
    Custom(String),
}

impl std::fmt::Display for ToolchainChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Beta => write!(f, "beta"),
            Self::Nightly => write!(f, "nightly"),
            Self::Version(v) => write!(f, "{}", v),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl ToolchainChannel {
    /// Parse a channel string into a ToolchainChannel
    pub fn parse(channel: &str) -> Self {
        match channel.to_lowercase().as_str() {
            "stable" => Self::Stable,
            "beta" => Self::Beta,
            "nightly" => Self::Nightly,
            s => {
                // Check if it looks like a version number
                if s.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    Self::Version(s.to_string())
                } else {
                    Self::Custom(s.to_string())
                }
            }
        }
    }
}

/// Information about an installed toolchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainInfo {
    /// Toolchain name/channel
    pub channel: ToolchainChannel,
    /// Whether this is the default toolchain
    pub is_default: bool,
    /// Whether the toolchain is installed
    pub is_installed: bool,
}

/// Version requirements from locked config
#[derive(Debug, Clone)]
pub struct VersionRequirements {
    /// Minimum required version (inclusive)
    pub minimum: Option<Version>,
    /// Maximum allowed version (inclusive)
    pub maximum: Option<Version>,
    /// Exact version requirement
    pub exact: Option<Version>,
}

impl VersionRequirements {
    /// Create empty requirements (no constraints)
    pub fn new() -> Self {
        Self {
            minimum: None,
            maximum: None,
            exact: None,
        }
    }

    /// Check if a version meets the requirements
    pub fn check(&self, version: &Version) -> bool {
        if let Some(exact) = &self.exact {
            return version == exact;
        }

        if let Some(minimum) = &self.minimum {
            if version < minimum {
                return false;
            }
        }

        if let Some(maximum) = &self.maximum {
            if version > maximum {
                return false;
            }
        }

        true
    }

    /// Get a human-readable description of the requirements
    pub fn description(&self) -> String {
        if let Some(exact) = &self.exact {
            return format!("exactly {}", exact);
        }

        match (&self.minimum, &self.maximum) {
            (Some(min), Some(max)) => format!("between {} and {}", min, max),
            (Some(min), None) => format!(">= {}", min),
            (None, Some(max)) => format!("<= {}", max),
            (None, None) => "any version".to_string(),
        }
    }
}

impl Default for VersionRequirements {
    fn default() -> Self {
        Self::new()
    }
}

/// Rustup manager for toolchain operations
pub struct RustupManager;

impl RustupManager {
    /// Create a new rustup manager
    pub fn new() -> Self {
        Self
    }

    /// Check if rustup is available on the system
    pub fn is_available(&self) -> bool {
        is_rustup_available()
    }

    /// Ensure rustup is available, returning an error if not
    fn ensure_rustup(&self) -> Result<()> {
        if !self.is_available() {
            return Err(Error::rust_not_found(
                "rustup not found. Please install rustup from https://rustup.rs",
            ));
        }
        Ok(())
    }

    /// Get the current Rust version with toolchain info
    ///
    /// # Errors
    ///
    /// Returns an error if `rustc` is not found or its version output cannot be parsed.
    pub async fn get_current_version(&self) -> Result<RustVersion> {
        crate::rust_version::detector::detect_rust_version()
    }

    /// List all installed toolchains
    ///
    /// # Errors
    ///
    /// Returns an error if rustup is not available or the toolchain list cannot be retrieved.
    pub fn list_toolchains(&self) -> Result<Vec<ToolchainInfo>> {
        self.ensure_rustup()?;

        let active = get_active_toolchain()?;
        let installed = get_installed_toolchains()?;

        let toolchains: Vec<ToolchainInfo> = installed
            .into_iter()
            .map(|name| {
                let channel = ToolchainChannel::parse(&name);
                let is_default = name == active;

                ToolchainInfo {
                    channel,
                    is_default,
                    is_installed: true,
                }
            })
            .collect();

        Ok(toolchains)
    }

    /// Install a specific toolchain
    ///
    /// # Errors
    ///
    /// Returns an error if rustup is not available or the installation fails.
    pub async fn install_toolchain(&self, channel: &ToolchainChannel) -> Result<()> {
        self.ensure_rustup()?;

        let channel_str = channel.to_string();
        info!("Installing toolchain: {}", channel_str);

        let output = Command::new("rustup")
            .args(["toolchain", "install", &channel_str, "--no-self-update"])
            .output()
            .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::command(format!(
                "Failed to install toolchain '{}': {}",
                channel_str, stderr
            )));
        }

        info!("Successfully installed toolchain: {}", channel_str);
        Ok(())
    }

    /// Uninstall a specific toolchain
    ///
    /// # Errors
    ///
    /// Returns an error if rustup is not available or the uninstallation fails.
    pub async fn uninstall_toolchain(&self, channel: &ToolchainChannel) -> Result<()> {
        self.ensure_rustup()?;

        let channel_str = channel.to_string();
        info!("Uninstalling toolchain: {}", channel_str);

        let output = Command::new("rustup")
            .args(["toolchain", "uninstall", &channel_str])
            .output()
            .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::command(format!(
                "Failed to uninstall toolchain '{}': {}",
                channel_str, stderr
            )));
        }

        info!("Successfully uninstalled toolchain: {}", channel_str);
        Ok(())
    }

    /// Switch to a different toolchain (set as default)
    ///
    /// # Errors
    ///
    /// Returns an error if rustup is not available or the switch fails.
    pub async fn switch_toolchain(&self, channel: &ToolchainChannel) -> Result<()> {
        self.ensure_rustup()?;

        let channel_str = channel.to_string();
        info!("Switching to toolchain: {}", channel_str);

        let output = Command::new("rustup")
            .args(["default", &channel_str])
            .output()
            .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::command(format!(
                "Failed to switch to toolchain '{}': {}",
                channel_str, stderr
            )));
        }

        info!("Successfully switched to toolchain: {}", channel_str);
        Ok(())
    }

    /// Update all installed toolchains
    ///
    /// # Errors
    ///
    /// Returns an error if rustup is not available or the update fails.
    pub async fn update_toolchains(&self) -> Result<UpdateResult> {
        self.ensure_rustup()?;

        info!("Updating toolchains...");

        let output = Command::new("rustup")
            .args(["update", "--no-self-update"])
            .output()
            .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(Error::command(format!(
                "Failed to update toolchains: {}",
                stderr
            )));
        }

        // Parse output to determine what was updated
        let updated = stdout
            .lines()
            .chain(stderr.lines())
            .filter(|line| line.contains("updated") || line.contains("installed"))
            .map(|s| s.to_string())
            .collect();

        info!("Toolchain update completed");

        Ok(UpdateResult {
            success: true,
            updated,
        })
    }

    /// Install a toolchain component (e.g., clippy, rustfmt)
    ///
    /// # Errors
    ///
    /// Returns an error if rustup is not available or the component installation fails.
    pub async fn install_component(&self, component: &str, toolchain: Option<&str>) -> Result<()> {
        self.ensure_rustup()?;

        let mut args = vec!["component", "add", component];
        if let Some(tc) = toolchain {
            args.push("--toolchain");
            args.push(tc);
        }

        info!("Installing component '{}'", component);

        let output = Command::new("rustup")
            .args(&args)
            .output()
            .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::command(format!(
                "Failed to install component '{}': {}",
                component, stderr
            )));
        }

        info!("Successfully installed component '{}'", component);
        Ok(())
    }

    /// Get version requirements from locked configuration
    ///
    /// # Errors
    ///
    /// Returns an error if loading the lock configuration fails.
    pub async fn get_version_requirements(&self) -> Result<VersionRequirements> {
        let lock_manager = HierarchicalLockManager::load().await?;
        let mut requirements = VersionRequirements::new();

        // Check for locked rust-version
        if let Some((_, entry)) = lock_manager.is_locked("rust-version") {
            debug!("Found locked rust-version: {}", entry.value);
            if let Ok(version) = Version::parse(&entry.value) {
                // For now, treat locked version as minimum requirement
                // This could be extended to support range syntax like ">=1.70.0, <1.80.0"
                requirements.minimum = Some(version);
            }
        }

        // Check for locked maximum version (if defined)
        if let Some((_, entry)) = lock_manager.is_locked("max-rust-version") {
            debug!("Found locked max-rust-version: {}", entry.value);
            if let Ok(version) = Version::parse(&entry.value) {
                requirements.maximum = Some(version);
            }
        }

        Ok(requirements)
    }

    /// Check if current Rust version meets locked requirements
    ///
    /// # Errors
    ///
    /// Returns an error if the current version cannot be determined or the lock configuration cannot be loaded.
    pub async fn check_version_requirements(&self) -> Result<VersionCheckResult> {
        let current = self.get_current_version().await?;
        let requirements = self.get_version_requirements().await?;

        let meets_requirements = requirements.check(&current.version);

        Ok(VersionCheckResult {
            current: current.version,
            requirements,
            meets_requirements,
        })
    }

    /// Run rustup self-update
    ///
    /// # Errors
    ///
    /// Returns an error if rustup is not available or the self-update fails.
    pub async fn self_update(&self) -> Result<()> {
        self.ensure_rustup()?;

        info!("Running rustup self-update...");

        let output = Command::new("rustup")
            .args(["self", "update"])
            .output()
            .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::command(format!(
                "Failed to self-update rustup: {}",
                stderr
            )));
        }

        info!("Rustup self-update completed");
        Ok(())
    }

    /// Show active toolchain information
    ///
    /// # Errors
    ///
    /// Returns an error if rustup is not available or the active toolchain cannot be determined.
    pub fn show_active_toolchain(&self) -> Result<String> {
        self.ensure_rustup()?;

        let output = Command::new("rustup")
            .args(["show", "active-toolchain"])
            .output()
            .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::command(format!(
                "Failed to show active toolchain: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl Default for RustupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of an update operation
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// Whether the update was successful
    pub success: bool,
    /// List of updated items
    pub updated: Vec<String>,
}

/// Result of a version check
#[derive(Debug, Clone)]
pub struct VersionCheckResult {
    /// Current installed version
    pub current: Version,
    /// Required version constraints
    pub requirements: VersionRequirements,
    /// Whether the current version meets requirements
    pub meets_requirements: bool,
}

impl VersionCheckResult {
    /// Format a human-readable status message
    pub fn status_message(&self) -> String {
        if self.meets_requirements {
            format!(
                "✅ Current version {} meets requirements ({})",
                self.current,
                self.requirements.description()
            )
        } else {
            format!(
                "❌ Current version {} does NOT meet requirements ({})",
                self.current,
                self.requirements.description()
            )
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchain_channel_display() {
        assert_eq!(ToolchainChannel::Stable.to_string(), "stable");
        assert_eq!(ToolchainChannel::Beta.to_string(), "beta");
        assert_eq!(ToolchainChannel::Nightly.to_string(), "nightly");
        assert_eq!(
            ToolchainChannel::Version("1.70.0".to_string()).to_string(),
            "1.70.0"
        );
        assert_eq!(
            ToolchainChannel::Custom("my-toolchain".to_string()).to_string(),
            "my-toolchain"
        );
    }

    #[test]
    fn test_toolchain_channel_parse() {
        assert!(matches!(
            ToolchainChannel::parse("stable"),
            ToolchainChannel::Stable
        ));
        assert!(matches!(
            ToolchainChannel::parse("beta"),
            ToolchainChannel::Beta
        ));
        assert!(matches!(
            ToolchainChannel::parse("nightly"),
            ToolchainChannel::Nightly
        ));
        assert!(matches!(
            ToolchainChannel::parse("1.70.0"),
            ToolchainChannel::Version(_)
        ));
        assert!(matches!(
            ToolchainChannel::parse("custom-toolchain"),
            ToolchainChannel::Custom(_)
        ));
    }

    #[test]
    fn test_version_requirements_check() {
        let mut req = VersionRequirements::new();
        let v170 = Version::new(1, 70, 0);
        let v180 = Version::new(1, 80, 0);
        let v190 = Version::new(1, 90, 0);

        // No constraints
        assert!(req.check(&v170));

        // Minimum version
        req.minimum = Some(v180.clone());
        assert!(!req.check(&v170));
        assert!(req.check(&v180));
        assert!(req.check(&v190));

        // Maximum version
        req = VersionRequirements::new();
        req.maximum = Some(v180.clone());
        assert!(req.check(&v170));
        assert!(req.check(&v180));
        assert!(!req.check(&v190));

        // Exact version
        req = VersionRequirements::new();
        req.exact = Some(v180.clone());
        assert!(!req.check(&v170));
        assert!(req.check(&v180));
        assert!(!req.check(&v190));
    }

    #[test]
    fn test_version_requirements_description() {
        let mut req = VersionRequirements::new();
        assert_eq!(req.description(), "any version");

        req.minimum = Some(Version::new(1, 70, 0));
        assert_eq!(req.description(), ">= 1.70.0");

        req.maximum = Some(Version::new(1, 80, 0));
        assert_eq!(req.description(), "between 1.70.0 and 1.80.0");

        req = VersionRequirements::new();
        req.exact = Some(Version::new(1, 75, 0));
        assert_eq!(req.description(), "exactly 1.75.0");
    }
}

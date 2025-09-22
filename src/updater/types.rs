//! Update system types

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
            _ => Err(Error::config(format!("Invalid update channel: {}", s))),
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
    /// Version being updated to
    pub version: Version,
    /// Download URL for the update
    pub download_url: String,
    /// Size of the download in bytes
    pub size: u64,
    /// SHA256 hash of the download
    pub sha256: Option<String>,
    /// Release notes or changelog
    pub notes: String,
    /// Whether this is a critical security update
    pub critical: bool,
}

/// Update manager for handling self-updates
#[derive(Debug)]
pub struct UpdateManager {
    /// Current version of Ferrous Forge
    pub current_version: Version,
    /// Channel to check for updates
    pub channel: UpdateChannel,
    /// Path to the current binary
    pub binary_path: PathBuf,
}
//! Rust version management and checking
//!
//! This module provides functionality to detect installed Rust versions,
//! check for updates from GitHub releases, and provide recommendations.

use crate::Result;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub mod cache;
pub mod detector;
pub mod github;

pub use detector::RustVersion;
pub use github::{GitHubClient, GitHubRelease};

/// Rust release channel
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Channel {
    /// Stable releases
    Stable,
    /// Beta releases
    Beta,
    /// Nightly builds
    Nightly,
    /// Custom or unknown channel
    Custom(String),
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Beta => write!(f, "beta"),
            Self::Nightly => write!(f, "nightly"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Version update recommendation
#[derive(Debug, Clone)]
pub enum UpdateRecommendation {
    /// Already on latest version
    UpToDate,
    /// Minor update available
    MinorUpdate {
        /// Current Rust version
        current: Version,
        /// Latest available version
        latest: Version,
        /// URL to the release page
        release_url: String,
    },
    /// Major update available
    MajorUpdate {
        /// Current Rust version
        current: Version,
        /// Latest available version
        latest: Version,
        /// URL to the release page
        release_url: String,
    },
    /// Security update available
    SecurityUpdate {
        /// Current Rust version
        current: Version,
        /// Latest available version
        latest: Version,
        /// URL to the release page
        release_url: String,
        /// Security update details
        details: String,
    },
}

/// Version manager for checking and recommending updates
pub struct VersionManager {
    github_client: GitHubClient,
    cache: Arc<RwLock<cache::Cache<String, Vec<u8>>>>,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new() -> Result<Self> {
        let github_client = GitHubClient::new(None)?;
        let cache = Arc::new(RwLock::new(cache::Cache::new(Duration::from_secs(3600))));
        
        Ok(Self {
            github_client,
            cache,
        })
    }
    
    /// Check current Rust installation
    pub async fn check_current(&self) -> Result<RustVersion> {
        detector::detect_rust_version()
    }
    
    /// Get latest stable release
    pub async fn get_latest_stable(&self) -> Result<GitHubRelease> {
        // Check cache first
        let cache_key = "latest_stable";
        
        {
            let cache = self.cache.read().await;
            if let Some(cached_bytes) = cache.get(&cache_key.to_string()) {
                if let Ok(release) = serde_json::from_slice::<GitHubRelease>(&cached_bytes) {
                    return Ok(release);
                }
            }
        }
        
        // Fetch from GitHub
        let release = self.github_client.get_latest_release().await?;
        
        // Cache the result
        if let Ok(bytes) = serde_json::to_vec(&release) {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key.to_string(), bytes);
        }
        
        Ok(release)
    }
    
    /// Get update recommendation
    pub async fn get_recommendation(&self) -> Result<UpdateRecommendation> {
        let current = self.check_current().await?;
        let latest = self.get_latest_stable().await?;
        
        // Compare versions
        if latest.version <= current.version {
            return Ok(UpdateRecommendation::UpToDate);
        }
        
        // Check if it's a security update
        let is_security = latest.body.to_lowercase().contains("security") ||
                         latest.name.to_lowercase().contains("security");
        
        if is_security {
            return Ok(UpdateRecommendation::SecurityUpdate {
                current: current.version.clone(),
                latest: latest.version.clone(),
                release_url: latest.html_url.clone(),
                details: self.extract_security_details(&latest.body),
            });
        }
        
        // Check if it's a major update
        if latest.version.major > current.version.major {
            return Ok(UpdateRecommendation::MajorUpdate {
                current: current.version.clone(),
                latest: latest.version.clone(),
                release_url: latest.html_url.clone(),
            });
        }
        
        // It's a minor/patch update
        Ok(UpdateRecommendation::MinorUpdate {
            current: current.version.clone(),
            latest: latest.version.clone(),
            release_url: latest.html_url.clone(),
        })
    }
    
    /// Get multiple recent releases
    pub async fn get_recent_releases(&self, count: usize) -> Result<Vec<GitHubRelease>> {
        self.github_client.get_releases(count).await
    }
    
    fn extract_security_details(&self, body: &str) -> String {
        // Extract security-related information from release notes
        body.lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.contains("security") || 
                lower.contains("vulnerability") ||
                lower.contains("cve-")
            })
            .take(3)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create version manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_channel_display() {
        assert_eq!(Channel::Stable.to_string(), "stable");
        assert_eq!(Channel::Beta.to_string(), "beta");
        assert_eq!(Channel::Nightly.to_string(), "nightly");
        assert_eq!(Channel::Custom("custom".to_string()).to_string(), "custom");
    }
}

//! Rust version management and checking
//!
//! This module provides functionality to detect installed Rust versions,
//! check for updates from GitHub releases, and provide recommendations.
//!
//! @task T024
//! @epic T014

use crate::Result;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Version information caching with TTL support.
pub mod cache;
/// Installed Rust version detection.
pub mod detector;
/// File-based cache for offline support.
pub mod file_cache;
/// GitHub API client for fetching Rust releases.
pub mod github;
/// Release notes parser for security/breaking changes.
pub mod parser;
/// Rustup integration for toolchain management.
pub mod rustup;
/// Security advisory checker.
pub mod security;

pub use detector::RustVersion;
pub use github::{GitHubClient, GitHubRelease};
pub use security::{SecurityCheckResult, SecurityChecker};

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

/// Update information for available version
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Current Rust version
    pub current: Version,
    /// Latest available version
    pub latest: Version,
    /// URL to the release page
    pub release_url: String,
    /// Security update details (if applicable)
    pub security_details: Option<String>,
}

/// Version update recommendation
#[derive(Debug, Clone)]
pub enum UpdateRecommendation {
    /// Already on latest version
    UpToDate,
    /// Minor update available
    MinorUpdate(UpdateInfo),
    /// Major update available
    MajorUpdate(UpdateInfo),
    /// Security update available
    SecurityUpdate(UpdateInfo),
}

/// Release notes with parsed details
#[derive(Debug, Clone)]
pub struct ReleaseNotes {
    /// Version string
    pub version: String,
    /// Full release notes
    pub full_notes: String,
    /// Parsed details
    pub parsed: parser::ParsedRelease,
}

/// Version manager for checking and recommending updates
pub struct VersionManager {
    github_client: GitHubClient,
    cache: Arc<RwLock<cache::Cache<String, Vec<u8>>>>,
    file_cache: file_cache::FileCache,
}

impl VersionManager {
    /// Create a new version manager
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn new() -> Result<Self> {
        let github_client = GitHubClient::new(None)?;
        let cache = Arc::new(RwLock::new(cache::Cache::new(Duration::from_secs(3600))));
        let file_cache = file_cache::FileCache::create_default()?;

        Ok(Self {
            github_client,
            cache,
            file_cache,
        })
    }

    /// Check current Rust installation
    ///
    /// # Errors
    ///
    /// Returns an error if `rustc` is not found or its output cannot be parsed.
    pub async fn check_current(&self) -> Result<RustVersion> {
        detector::detect_rust_version().await
    }

    /// Get latest stable release
    ///
    /// # Errors
    ///
    /// Returns an error if the GitHub API request fails or the response cannot be parsed.
    pub async fn get_latest_stable(&self) -> Result<GitHubRelease> {
        // Check file cache first for offline support
        let cache_key = "latest_stable";

        if let Some(entry) = self.file_cache.get(cache_key)
            && let Ok(release) = serde_json::from_slice::<GitHubRelease>(&entry.data)
        {
            tracing::debug!("Using cached latest stable release");
            return Ok(release);
        }

        // Check in-memory cache
        {
            let cache = self.cache.read().await;
            if let Some(cached_bytes) = cache.get(&cache_key.to_string())
                && let Ok(release) = serde_json::from_slice::<GitHubRelease>(&cached_bytes)
            {
                return Ok(release);
            }
        }

        // Fetch from GitHub
        let release = self.github_client.get_latest_release().await?;

        // Cache in both in-memory and file cache
        if let Ok(bytes) = serde_json::to_vec(&release) {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key.to_string(), bytes.clone());
            let _ = self.file_cache.set(cache_key, bytes, "application/json");
        }

        Ok(release)
    }

    /// Get update recommendation
    ///
    /// # Errors
    ///
    /// Returns an error if the current version cannot be detected or the latest
    /// release cannot be fetched from GitHub.
    pub async fn get_recommendation(&self) -> Result<UpdateRecommendation> {
        let current = self.check_current().await?;
        let latest = self.get_latest_stable().await?;

        // Check if already up to date
        if latest.version <= current.version {
            return Ok(UpdateRecommendation::UpToDate);
        }

        // Determine update type based on release content and version difference
        self.determine_update_type(&current, &latest)
    }

    /// Determine the type of update based on version comparison and release content
    fn determine_update_type(
        &self,
        current: &RustVersion,
        latest: &GitHubRelease,
    ) -> Result<UpdateRecommendation> {
        // Parse release notes for security advisories
        let parsed = parser::parse_release_notes(&latest.tag_name, &latest.body);

        if !parsed.security_advisories.is_empty() {
            Ok(self.create_security_update(current, latest, &parsed))
        } else if self.is_major_update(current, latest) {
            Ok(self.create_major_update(current, latest))
        } else {
            Ok(self.create_minor_update(current, latest))
        }
    }

    /// Check if the release contains security-related updates
    #[allow(dead_code)]
    fn is_security_update(&self, release: &GitHubRelease) -> bool {
        let body_lower = release.body.to_lowercase();
        let name_lower = release.name.to_lowercase();
        body_lower.contains("security") || name_lower.contains("security")
    }

    /// Check if this is a major version update
    fn is_major_update(&self, current: &RustVersion, latest: &GitHubRelease) -> bool {
        latest.version.major > current.version.major
    }

    /// Create a security update recommendation
    fn create_security_update(
        &self,
        current: &RustVersion,
        latest: &GitHubRelease,
        parsed: &parser::ParsedRelease,
    ) -> UpdateRecommendation {
        let security_summary = if !parsed.security_advisories.is_empty() {
            Some(
                parsed
                    .security_advisories
                    .iter()
                    .map(|a| {
                        if let Some(ref id) = a.id {
                            format!("{}: {}", id, a.description)
                        } else {
                            a.description.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        } else {
            Some(self.extract_security_details(&latest.body))
        };

        let info = UpdateInfo {
            current: current.version.clone(),
            latest: latest.version.clone(),
            release_url: latest.html_url.clone(),
            security_details: security_summary,
        };
        UpdateRecommendation::SecurityUpdate(info)
    }

    /// Create a major update recommendation
    fn create_major_update(
        &self,
        current: &RustVersion,
        latest: &GitHubRelease,
    ) -> UpdateRecommendation {
        let info = UpdateInfo {
            current: current.version.clone(),
            latest: latest.version.clone(),
            release_url: latest.html_url.clone(),
            security_details: None,
        };
        UpdateRecommendation::MajorUpdate(info)
    }

    /// Create a minor/patch update recommendation
    fn create_minor_update(
        &self,
        current: &RustVersion,
        latest: &GitHubRelease,
    ) -> UpdateRecommendation {
        let info = UpdateInfo {
            current: current.version.clone(),
            latest: latest.version.clone(),
            release_url: latest.html_url.clone(),
            security_details: None,
        };
        UpdateRecommendation::MinorUpdate(info)
    }

    /// Get multiple recent releases
    ///
    /// # Errors
    ///
    /// Returns an error if the GitHub API request fails or the response cannot be parsed.
    pub async fn get_recent_releases(&self, count: usize) -> Result<Vec<GitHubRelease>> {
        // Check file cache first
        let cache_key = format!("recent_releases_{}", count);

        if let Some(entry) = self.file_cache.get(&cache_key)
            && let Ok(releases) = serde_json::from_slice::<Vec<GitHubRelease>>(&entry.data)
        {
            tracing::debug!("Using cached releases");
            return Ok(releases);
        }

        // Fetch from GitHub
        let releases = self.github_client.get_releases(count).await?;

        // Cache the result
        if let Ok(data) = serde_json::to_vec(&releases) {
            let _ = self.file_cache.set(&cache_key, data, "application/json");
        }

        Ok(releases)
    }

    /// Get release notes for a specific version
    ///
    /// # Errors
    ///
    /// Returns an error if the release cannot be found or fetched.
    pub async fn get_release_notes(&self, version: &str) -> Result<ReleaseNotes> {
        // Check file cache first
        let cache_key = format!("release_notes_{}", version);

        if let Some(entry) = self.file_cache.get(&cache_key)
            && let Ok(release) = serde_json::from_slice::<GitHubRelease>(&entry.data)
        {
            let parsed = parser::parse_release_notes(&release.tag_name, &release.body);
            return Ok(ReleaseNotes {
                version: release.tag_name,
                full_notes: release.body,
                parsed,
            });
        }

        // Fetch from GitHub
        let release = self.github_client.get_release_by_tag(version).await?;

        // Cache the result
        if let Ok(data) = serde_json::to_vec(&release) {
            let _ = self.file_cache.set(&cache_key, data, "application/json");
        }

        let parsed = parser::parse_release_notes(&release.tag_name, &release.body);
        Ok(ReleaseNotes {
            version: release.tag_name,
            full_notes: release.body,
            parsed,
        })
    }

    /// Check for updates (returns true if updates available)
    ///
    /// # Errors
    ///
    /// Returns an error if the check fails.
    pub async fn check_updates(&self) -> Result<(bool, Option<Version>)> {
        let current = self.check_current().await?;
        let latest = self.get_latest_stable().await?;

        if latest.version > current.version {
            Ok((true, Some(latest.version)))
        } else {
            Ok((false, None))
        }
    }

    /// Check if offline mode should be used
    pub fn is_offline_mode(&self) -> bool {
        self.file_cache.should_use_offline()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> file_cache::CacheStats {
        self.file_cache.stats()
    }

    fn extract_security_details(&self, body: &str) -> String {
        // Extract security-related information from release notes
        body.lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.contains("security")
                    || lower.contains("vulnerability")
                    || lower.contains("cve-")
            })
            .take(3)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
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

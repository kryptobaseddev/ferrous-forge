//! GitHub API client for fetching Rust releases

use crate::{Error, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};

const GITHUB_API_BASE: &str = "https://api.github.com";
const RUST_REPO_OWNER: &str = "rust-lang";
const RUST_REPO_NAME: &str = "rust";

/// Default version for deserialization
fn default_version() -> Version {
    Version::new(0, 0, 0)
}

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    /// Release ID
    pub id: u64,
    /// Tag name (e.g., "1.90.0")
    pub tag_name: String,
    /// Release name
    pub name: String,
    /// Release description/notes
    pub body: String,
    /// Is this a draft?
    pub draft: bool,
    /// Is this a prerelease?
    pub prerelease: bool,
    /// Creation date
    pub created_at: DateTime<Utc>,
    /// Publication date
    pub published_at: Option<DateTime<Utc>>,
    /// HTML URL to the release page
    pub html_url: String,
    /// Parsed semantic version
    #[serde(skip, default = "default_version")]
    pub version: Version,
}

/// Simplified author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// GitHub username
    pub login: String,
    /// GitHub user ID
    pub id: u64,
}

/// GitHub API client
pub struct GitHubClient {
    client: Client,
    auth_token: Option<String>,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new(auth_token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(format!("ferrous-forge/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| Error::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, auth_token })
    }

    /// Get the latest stable release
    pub async fn get_latest_release(&self) -> Result<GitHubRelease> {
        let url = format!(
            "{}/repos/{}/{}/releases/latest",
            GITHUB_API_BASE, RUST_REPO_OWNER, RUST_REPO_NAME
        );

        let mut request = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to fetch release: {}", e)))?;

        // Check for rate limiting
        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("X-RateLimit-Reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60);

            return Err(Error::rate_limited(retry_after));
        }

        if !response.status().is_success() {
            return Err(Error::network(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let mut release: GitHubRelease = response
            .json()
            .await
            .map_err(|e| Error::parse(format!("Failed to parse release JSON: {}", e)))?;

        // Parse version from tag
        release.version = self.parse_version_from_tag(&release.tag_name)?;

        Ok(release)
    }

    /// Get multiple recent releases
    pub async fn get_releases(&self, count: usize) -> Result<Vec<GitHubRelease>> {
        let url = format!(
            "{}/repos/{}/{}/releases?per_page={}",
            GITHUB_API_BASE, RUST_REPO_OWNER, RUST_REPO_NAME, count
        );

        let mut request = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to fetch releases: {}", e)))?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("X-RateLimit-Reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60);

            return Err(Error::rate_limited(retry_after));
        }

        if !response.status().is_success() {
            return Err(Error::network(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let mut releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| Error::parse(format!("Failed to parse releases JSON: {}", e)))?;

        // Parse versions for all releases
        for release in &mut releases {
            release.version = self.parse_version_from_tag(&release.tag_name)?;
        }

        // Filter out pre-releases from stable channel
        Ok(releases.into_iter().filter(|r| !r.prerelease).collect())
    }

    /// Parse semantic version from tag name
    fn parse_version_from_tag(&self, tag: &str) -> Result<Version> {
        let version_str = tag.strip_prefix('v').unwrap_or(tag);
        Version::parse(version_str)
            .map_err(|e| Error::parse(format!("Failed to parse version '{}': {}", tag, e)))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_version_from_tag() {
        let client = GitHubClient::new(None).unwrap();

        assert_eq!(
            client.parse_version_from_tag("1.90.0").unwrap(),
            Version::new(1, 90, 0)
        );

        assert_eq!(
            client.parse_version_from_tag("v1.90.0").unwrap(),
            Version::new(1, 90, 0)
        );
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_latest_release() {
        let client = GitHubClient::new(None).unwrap();
        let release = client.get_latest_release().await.unwrap();

        assert!(!release.tag_name.is_empty());
        assert!(release.version.major >= 1);
    }
}

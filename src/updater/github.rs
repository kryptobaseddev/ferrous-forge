//! GitHub API integration for fetching updates

use super::types::{UpdateChannel, UpdateInfo};
use crate::{Error, Result};
use semver::Version;
use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: String,
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GitHubReleases {
    releases: Vec<GitHubRelease>,
}

/// Fetch update information from GitHub releases
pub async fn fetch_update_info(
    current_version: &Version,
    channel: &UpdateChannel,
) -> Result<Option<UpdateInfo>> {
    let client = reqwest::Client::new();
    let url = "https://api.github.com/repos/ferrous-systems/ferrous-forge/releases";

    let response = client
        .get(url)
        .header("User-Agent", "ferrous-forge")
        .send()
        .await
        .map_err(|e| Error::network(format!("Failed to fetch releases: {}", e)))?;

    if !response.status().is_success() {
        return Err(Error::network(format!(
            "GitHub API request failed: {}",
            response.status()
        )));
    }

    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .map_err(|e| Error::network(format!("Failed to parse GitHub response: {}", e)))?;

    for release in releases {
        // Skip prereleases for stable channel
        if matches!(channel, UpdateChannel::Stable) && release.prerelease {
            continue;
        }

        // Parse version from tag
        let tag_version = release.tag_name.trim_start_matches('v');
        let version = match Version::parse(tag_version) {
            Ok(v) => v,
            Err(_) => continue, // Skip invalid versions
        };

        // Only consider newer versions
        if version <= *current_version {
            continue;
        }

        // Find appropriate asset for current platform
        let platform_suffix = get_platform_suffix();
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name.contains(&platform_suffix));

        if let Some(asset) = asset {
            return Ok(Some(UpdateInfo {
                version,
                download_url: asset.browser_download_url.clone(),
                size: asset.size,
                sha256: None, // GitHub doesn't provide SHA256 in API
                notes: release.body,
                critical: false, // Would need to parse from release notes
            }));
        }
    }

    Ok(None)
}

/// Get platform-specific binary suffix
fn get_platform_suffix() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => "linux-x86_64",
        ("macos", "x86_64") => "darwin-x86_64",
        ("macos", "aarch64") => "darwin-aarch64",
        ("windows", "x86_64") => "windows-x86_64.exe",
        _ => "unknown",
    }
    .to_string()
}

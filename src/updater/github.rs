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
    let releases = fetch_github_releases().await?;
    find_suitable_update(current_version, channel, &releases)
}

/// Fetch releases from GitHub API
async fn fetch_github_releases() -> Result<Vec<GitHubRelease>> {
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

    response
        .json()
        .await
        .map_err(|e| Error::network(format!("Failed to parse GitHub response: {}", e)))
}

/// Find a suitable update from the list of releases
fn find_suitable_update(
    current_version: &Version,
    channel: &UpdateChannel,
    releases: &[GitHubRelease],
) -> Result<Option<UpdateInfo>> {
    for release in releases {
        if !is_release_suitable(release, channel) {
            continue;
        }

        if let Some(version) = parse_release_version(&release.tag_name) {
            if version <= *current_version {
                continue;
            }

            if let Some(update_info) = create_update_info(version, release) {
                return Ok(Some(update_info));
            }
        }
    }
    Ok(None)
}

/// Check if a release is suitable for the given channel
fn is_release_suitable(release: &GitHubRelease, channel: &UpdateChannel) -> bool {
    !matches!(channel, UpdateChannel::Stable) || !release.prerelease
}

/// Parse version from release tag
fn parse_release_version(tag_name: &str) -> Option<Version> {
    let tag_version = tag_name.trim_start_matches('v');
    Version::parse(tag_version).ok()
}

/// Create update info from release data
fn create_update_info(version: Version, release: &GitHubRelease) -> Option<UpdateInfo> {
    let platform_suffix = get_platform_suffix();
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name.contains(&platform_suffix))?;

    Some(UpdateInfo {
        version,
        download_url: asset.browser_download_url.clone(),
        size: asset.size,
        sha256: None, // GitHub doesn't provide SHA256 in API
        notes: release.body.clone(),
        critical: false, // Would need to parse from release notes
    })
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

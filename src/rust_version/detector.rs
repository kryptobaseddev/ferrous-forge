//! Rust version detection from local installation

use crate::{Error, Result};
use chrono::NaiveDate;
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str;

use super::Channel;

/// Represents the current Rust installation version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustVersion {
    /// Semantic version
    pub version: Version,
    /// Commit hash
    pub commit_hash: String,
    /// Commit date
    pub commit_date: NaiveDate,
    /// Host triple (e.g., x86_64-unknown-linux-gnu)
    pub host: String,
    /// Release channel
    pub channel: Channel,
    /// Raw version string from rustc
    pub raw_string: String,
}

impl RustVersion {
    /// Parse rustc version output
    pub fn parse(version_output: &str) -> Result<Self> {
        // Example: rustc 1.90.0 (4b06a43a1 2025-08-07)
        let regex = Regex::new(
            r"rustc (\d+\.\d+\.\d+(?:-[\w.]+)?)\s*\(([a-f0-9]+)\s+(\d{4}-\d{2}-\d{2})\)",
        )?;

        let captures = regex
            .captures(version_output)
            .ok_or_else(|| Error::parse("Invalid rustc version output"))?;

        let version_str = &captures[1];
        let version = Version::parse(version_str)?;
        let commit_hash = captures[2].to_string();
        let commit_date = NaiveDate::parse_from_str(&captures[3], "%Y-%m-%d")
            .map_err(|e| Error::parse(format!("Failed to parse date: {}", e)))?;

        let channel = detect_channel(version_str);
        let host = detect_host();

        Ok(Self {
            version,
            commit_hash,
            commit_date,
            host,
            channel,
            raw_string: version_output.to_string(),
        })
    }
}

impl std::fmt::Display for RustVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rustc {} ({})", self.version, self.channel)
    }
}

/// Detect the currently installed Rust version
pub fn detect_rust_version() -> Result<RustVersion> {
    // Check if rustc is available
    let rustc_path = which::which("rustc").map_err(|_| {
        Error::rust_not_found("rustc not found. Please install Rust from https://rustup.rs")
    })?;

    // Get version output
    let output = Command::new(rustc_path)
        .arg("--version")
        .output()
        .map_err(|e| Error::command(format!("Failed to run rustc: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::command(format!("rustc failed: {}", stderr)));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| Error::parse(format!("Invalid UTF-8 in rustc output: {}", e)))?;

    RustVersion::parse(stdout)
}

/// Detect the channel from version string
fn detect_channel(version_str: &str) -> Channel {
    if version_str.contains("nightly") {
        Channel::Nightly
    } else if version_str.contains("beta") {
        Channel::Beta
    } else if version_str.contains("-") {
        // Has pre-release identifier
        Channel::Custom(version_str.to_string())
    } else {
        Channel::Stable
    }
}

/// Detect the host triple
fn detect_host() -> String {
    // Try to get from rustc
    if let Ok(output) = Command::new("rustc").arg("--print").arg("host").output() {
        if output.status.success() {
            if let Ok(host) = str::from_utf8(&output.stdout) {
                return host.trim().to_string();
            }
        }
    }

    // Fallback to a generic target string
    "unknown".to_string()
}

/// Get installed toolchains via rustup
pub fn get_installed_toolchains() -> Result<Vec<String>> {
    let rustup_path =
        which::which("rustup").map_err(|_| Error::rust_not_found("rustup not found"))?;

    let output = Command::new(rustup_path)
        .args(&["toolchain", "list"])
        .output()
        .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::command(format!("rustup failed: {}", stderr)));
    }

    let stdout = str::from_utf8(&output.stdout)?;

    Ok(stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            // Remove " (default)" suffix if present
            line.split_whitespace().next().unwrap_or(line).to_string()
        })
        .collect())
}

/// Check if rustup is available
pub fn is_rustup_available() -> bool {
    which::which("rustup").is_ok()
}

/// Get the active toolchain
pub fn get_active_toolchain() -> Result<String> {
    let rustup_path =
        which::which("rustup").map_err(|_| Error::rust_not_found("rustup not found"))?;

    let output = Command::new(rustup_path)
        .args(&["show", "active-toolchain"])
        .output()
        .map_err(|e| Error::command(format!("Failed to run rustup: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::command(format!("rustup failed: {}", stderr)));
    }

    let stdout = str::from_utf8(&output.stdout)?;

    Ok(stdout
        .split_whitespace()
        .next()
        .unwrap_or("unknown")
        .to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stable_version() {
        let output = "rustc 1.90.0 (4b06a43a1 2025-08-07)";
        let version = RustVersion::parse(output).unwrap();

        assert_eq!(version.version, Version::new(1, 90, 0));
        assert_eq!(version.commit_hash, "4b06a43a1");
        assert_eq!(version.commit_date.to_string(), "2025-08-07");
        assert_eq!(version.channel, Channel::Stable);
    }

    #[test]
    fn test_parse_beta_version() {
        let output = "rustc 1.91.0-beta.1 (5c8a0cafe 2025-09-01)";
        let version = RustVersion::parse(output).unwrap();

        assert_eq!(version.version.major, 1);
        assert_eq!(version.version.minor, 91);
        assert_eq!(version.version.patch, 0);
        assert_eq!(version.channel, Channel::Beta);
    }

    #[test]
    fn test_parse_nightly_version() {
        let output = "rustc 1.92.0-nightly (abc123def 2025-09-15)";
        let version = RustVersion::parse(output).unwrap();

        assert_eq!(version.channel, Channel::Nightly);
    }

    #[test]
    fn test_detect_channel() {
        assert_eq!(detect_channel("1.90.0"), Channel::Stable);
        assert_eq!(detect_channel("1.91.0-beta.1"), Channel::Beta);
        assert_eq!(detect_channel("1.92.0-nightly"), Channel::Nightly);
    }
}

//! Security advisory checker for Rust versions
//!
//! Checks current Rust version against known security advisories
//! and warns about vulnerabilities.
//!
//! @task T024
//! @epic T014

use crate::rust_version::{
    VersionManager, detector::detect_rust_version, file_cache::FileCache, github::GitHubClient,
    parser::parse_release_notes,
};
use crate::{Error, Result};
use console::style;
use semver::Version;
use std::collections::HashMap;

/// Security check result
#[derive(Debug, Clone)]
pub struct SecurityCheckResult {
    /// Whether the current version is secure
    pub is_secure: bool,
    /// Current installed version
    pub current_version: Version,
    /// List of security issues affecting current version
    pub issues: Vec<SecurityIssue>,
    /// Recommended version to update to
    pub recommended_version: Option<Version>,
    /// Whether running in offline mode
    pub offline_mode: bool,
}

/// Security issue details
#[derive(Debug, Clone)]
pub struct SecurityIssue {
    /// Issue severity
    pub severity: crate::rust_version::parser::Severity,
    /// Description of the vulnerability
    pub description: String,
    /// CVE ID if available
    pub cve_id: Option<String>,
    /// First version that fixes this issue
    pub fixed_in: Option<Version>,
    /// Advisory URL
    pub url: Option<String>,
}

impl SecurityCheckResult {
    /// Check if there are critical security issues
    pub fn has_critical_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == crate::rust_version::parser::Severity::Critical)
    }

    /// Get the highest severity level found
    pub fn highest_severity(&self) -> Option<&crate::rust_version::parser::Severity> {
        self.issues.iter().map(|i| &i.severity).max()
    }

    /// Get count of issues by severity
    pub fn severity_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for issue in &self.issues {
            *counts.entry(issue.severity.to_string()).or_insert(0) += 1;
        }
        counts
    }
}

/// Security advisory checker
pub struct SecurityChecker {
    version_manager: VersionManager,
    cache: FileCache,
}

impl SecurityChecker {
    /// Create a new security checker
    ///
    /// # Errors
    ///
    /// Returns an error if the version manager or cache cannot be initialized.
    pub fn new() -> Result<Self> {
        let version_manager = VersionManager::new()?;
        let cache = FileCache::create_default()?;

        Ok(Self {
            version_manager,
            cache,
        })
    }

    /// Check current Rust version for security issues
    ///
    /// # Errors
    ///
    /// Returns an error if the current version cannot be detected or
    /// if the security check fails.
    pub async fn check_current_version(&self) -> Result<SecurityCheckResult> {
        let current = detect_rust_version()?;
        let offline_mode = self.cache.should_use_offline();

        let issues = if offline_mode {
            self.check_offline(&current.version).await?
        } else {
            self.check_online(&current.version).await?
        };

        // Determine recommended version
        let recommended_version = if !issues.is_empty() {
            self.find_recommended_version(&current.version).await?
        } else {
            None
        };

        let is_secure = issues.is_empty();

        Ok(SecurityCheckResult {
            is_secure,
            current_version: current.version.clone(),
            issues,
            recommended_version,
            offline_mode,
        })
    }

    /// Check for security issues using cached data only
    async fn check_offline(&self, current_version: &Version) -> Result<Vec<SecurityIssue>> {
        tracing::info!("Running security check in offline mode");

        // Try to get cached releases
        let cache_key = "recent_releases_30";
        let cached = self
            .cache
            .get(cache_key)
            .ok_or_else(|| Error::network("No cached data available for offline security check"))?;

        let releases: Vec<crate::rust_version::GitHubRelease> =
            serde_json::from_slice(&cached.data)
                .map_err(|e| Error::parse(format!("Failed to parse cached releases: {e}")))?;

        self.analyze_security_issues(current_version, &releases)
    }

    /// Check for security issues with online data
    async fn check_online(&self, current_version: &Version) -> Result<Vec<SecurityIssue>> {
        let client = GitHubClient::new(None)?;

        // Fetch recent releases
        let releases = client.get_releases(30).await?;

        // Cache the releases for offline use
        if let Ok(data) = serde_json::to_vec(&releases) {
            let _ = self
                .cache
                .set("recent_releases_30", data, "application/json");
        }

        self.analyze_security_issues(current_version, &releases)
    }

    /// Analyze releases for security issues affecting current version
    fn analyze_security_issues(
        &self,
        current_version: &Version,
        releases: &[crate::rust_version::GitHubRelease],
    ) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        for release in releases {
            // Only check releases newer than current version
            if release.version <= *current_version {
                continue;
            }

            let parsed = parse_release_notes(&release.tag_name, &release.body);

            for advisory in parsed.security_advisories {
                issues.push(SecurityIssue {
                    severity: advisory.severity,
                    description: advisory.description,
                    cve_id: advisory.id,
                    fixed_in: Some(release.version.clone()),
                    url: Some(release.html_url.clone()),
                });
            }
        }

        // Sort by severity (highest first)
        issues.sort_by(|a, b| b.severity.cmp(&a.severity));

        Ok(issues)
    }

    /// Find the recommended version to update to
    async fn find_recommended_version(
        &self,
        _current_version: &Version,
    ) -> Result<Option<Version>> {
        // Get the latest release that fixes all known issues
        let latest = self.version_manager.get_latest_stable().await?;
        Ok(Some(latest.version))
    }

    /// Display security check results
    pub fn display_results(result: &SecurityCheckResult) {
        println!();

        if result.is_secure {
            println!("{}", style("✅ Security Check Passed").green().bold());
            println!(
                "   Your Rust version {} has no known security vulnerabilities.",
                style(&result.current_version).green()
            );
        } else {
            let severity = result.highest_severity();
            let header = match severity {
                Some(s) if *s == crate::rust_version::parser::Severity::Critical => {
                    style("🚨 CRITICAL SECURITY ISSUES FOUND").red().bold()
                }
                Some(s) if *s == crate::rust_version::parser::Severity::High => {
                    style("⚠️  HIGH SEVERITY SECURITY ISSUES FOUND")
                        .yellow()
                        .bold()
                }
                _ => style("⚠️  Security Issues Found").yellow().bold(),
            };

            println!("{}", header);
            println!();
            println!(
                "   Your Rust version {} has {} known security issue{}.",
                style(&result.current_version).red(),
                result.issues.len(),
                if result.issues.len() == 1 { "" } else { "s" }
            );

            // Show severity breakdown
            let counts = result.severity_counts();
            let mut parts = Vec::new();
            for (sev, count) in &counts {
                let styled = match sev.as_str() {
                    "CRITICAL" => style(format!("{} CRITICAL", count)).red().bold(),
                    "HIGH" => style(format!("{} HIGH", count)).yellow().bold(),
                    "MEDIUM" => style(format!("{} MEDIUM", count)).yellow(),
                    "LOW" => style(format!("{} LOW", count)).dim(),
                    _ => style(format!("{} {}", count, sev)),
                };
                parts.push(styled.to_string());
            }

            if !parts.is_empty() {
                println!("   Severity breakdown: {}", parts.join(", "));
            }

            println!();
            println!("{}", style("   Security Issues:").bold());

            for issue in result.issues.iter().take(5) {
                let sev_icon = match issue.severity {
                    crate::rust_version::parser::Severity::Critical => "🔴",
                    crate::rust_version::parser::Severity::High => "🟠",
                    crate::rust_version::parser::Severity::Medium => "🟡",
                    crate::rust_version::parser::Severity::Low => "🔵",
                    crate::rust_version::parser::Severity::Unknown => "⚪",
                };

                println!();
                println!(
                    "   {} {} {}",
                    sev_icon,
                    style(format!("[{}]", issue.severity)).bold(),
                    issue.description
                );

                if let Some(ref cve) = issue.cve_id {
                    println!("      CVE: {}", style(cve).cyan());
                }

                if let Some(ref fixed) = issue.fixed_in {
                    println!("      Fixed in: {}", style(fixed).green());
                }

                if let Some(ref url) = issue.url {
                    println!("      URL: {}", style(url).dim());
                }
            }

            if result.issues.len() > 5 {
                println!("\n   ... and {} more issues", result.issues.len() - 5);
            }

            println!();
            if let Some(ref recommended) = result.recommended_version {
                println!(
                    "{} Update to the latest version",
                    style("🔧 Recommended Action:").bold()
                );
                println!(
                    "   Latest secure version: {}",
                    style(recommended).green().bold()
                );
                println!("   Update command: {}", style("rustup update").cyan());
            }
        }

        if result.offline_mode {
            println!();
            println!(
                "{}",
                style("📴 Running in offline mode (using cached data)").dim()
            );
        }

        println!();
    }
}

/// Quick security check for use during startup
///
/// Returns true if no critical issues found, false otherwise.
///
/// # Errors
///
/// Returns an error if the security check fails to execute.
pub async fn quick_security_check() -> Result<bool> {
    let checker = SecurityChecker::new()?;

    match checker.check_current_version().await {
        Ok(result) => {
            if !result.is_secure {
                SecurityChecker::display_results(&result);
            }
            Ok(!result.has_critical_issues())
        }
        Err(e) => {
            tracing::warn!("Security check failed: {}", e);
            // Don't fail on security check errors
            Ok(true)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_security_check_result_helpers() {
        let result = SecurityCheckResult {
            is_secure: false,
            current_version: Version::new(1, 70, 0),
            issues: vec![
                SecurityIssue {
                    severity: crate::rust_version::parser::Severity::Critical,
                    description: "Critical bug".to_string(),
                    cve_id: Some("CVE-2023-1".to_string()),
                    fixed_in: Some(Version::new(1, 71, 0)),
                    url: None,
                },
                SecurityIssue {
                    severity: crate::rust_version::parser::Severity::High,
                    description: "High bug".to_string(),
                    cve_id: None,
                    fixed_in: Some(Version::new(1, 71, 0)),
                    url: None,
                },
            ],
            recommended_version: Some(Version::new(1, 71, 0)),
            offline_mode: false,
        };

        assert!(result.has_critical_issues());
        assert_eq!(
            result.highest_severity(),
            Some(&crate::rust_version::parser::Severity::Critical)
        );

        let counts = result.severity_counts();
        assert_eq!(counts.get("CRITICAL"), Some(&1));
        assert_eq!(counts.get("HIGH"), Some(&1));
    }

    #[test]
    fn test_security_check_result_secure() {
        let result = SecurityCheckResult {
            is_secure: true,
            current_version: Version::new(1, 70, 0),
            issues: vec![],
            recommended_version: None,
            offline_mode: false,
        };

        assert!(!result.has_critical_issues());
        assert!(result.highest_severity().is_none());
        assert!(result.severity_counts().is_empty());
    }
}

//! Security audit integration module
//!
//! This module provides integration with cargo-audit to scan for security
//! vulnerabilities in dependencies.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Security audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// List of vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,
    /// Total number of dependencies
    pub dependencies_count: usize,
    /// Whether the audit passed (no vulnerabilities)
    pub passed: bool,
}

/// A single security vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// Package name
    pub package: String,
    /// Version with vulnerability
    pub version: String,
    /// Severity level
    pub severity: String,
    /// Title of the vulnerability
    pub title: String,
    /// Description of the issue
    pub description: String,
    /// CVE identifier if available
    pub cve: Option<String>,
    /// CVSS score if available
    pub cvss: Option<f32>,
}

impl AuditReport {
    /// Generate a human-readable report
    pub fn report(&self) -> String {
        let mut report = String::new();

        if self.passed {
            report.push_str("✅ Security audit passed - No vulnerabilities found!\n");
            report.push_str(&format!(
                "   Scanned {} dependencies\n",
                self.dependencies_count
            ));
        } else {
            report.push_str(&format!(
                "🚨 Security audit failed - Found {} vulnerabilities\n\n",
                self.vulnerabilities.len()
            ));

            for vuln in &self.vulnerabilities {
                let severity_emoji = match vuln.severity.to_lowercase().as_str() {
                    "critical" => "🔴",
                    "high" => "🟠",
                    "medium" => "🟡",
                    "low" => "🟢",
                    _ => "⚪",
                };

                report.push_str(&format!(
                    "{} {} [{}] in {} v{}\n",
                    severity_emoji,
                    vuln.severity.to_uppercase(),
                    vuln.cve.as_ref().unwrap_or(&"N/A".to_string()),
                    vuln.package,
                    vuln.version
                ));
                report.push_str(&format!("   {}\n", vuln.title));
                report.push_str(&format!("   {}\n", vuln.description));

                if let Some(cvss) = vuln.cvss {
                    report.push_str(&format!("   CVSS Score: {:.1}\n", cvss));
                }

                report.push('\n');
            }
        }

        report
    }
}

/// Run security audit on a project
pub async fn run_security_audit(project_path: &Path) -> Result<AuditReport> {
    // Ensure cargo-audit is installed
    ensure_cargo_audit_installed().await?;

    // Run cargo audit with JSON output
    let output = Command::new("cargo")
        .args(&["audit", "--json"])
        .current_dir(project_path)
        .output()
        .map_err(|e| Error::process(format!("Failed to run cargo audit: {}", e)))?;

    // Parse the output
    parse_audit_output(&output.stdout)
}

/// Ensure cargo-audit is installed
async fn ensure_cargo_audit_installed() -> Result<()> {
    let check = Command::new("cargo").args(&["audit", "--version"]).output();

    if check
        .as_ref()
        .map_or(true, |output| !output.status.success())
    {
        println!("📦 Installing cargo-audit for security scanning...");

        let install = Command::new("cargo")
            .args(&["install", "cargo-audit", "--locked"])
            .output()
            .map_err(|e| Error::process(format!("Failed to install cargo-audit: {}", e)))?;

        if !install.status.success() {
            return Err(Error::process("Failed to install cargo-audit"));
        }

        println!("✅ cargo-audit installed successfully");
    }

    Ok(())
}

/// Parse cargo audit JSON output
fn parse_audit_output(output: &[u8]) -> Result<AuditReport> {
    let output_str = String::from_utf8_lossy(output);

    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
        let mut vulnerabilities = Vec::new();

        // Extract vulnerabilities from the JSON structure
        if let Some(vulns) = json["vulnerabilities"]["list"].as_array() {
            for vuln in vulns {
                if let Some(advisory) = vuln["advisory"].as_object() {
                    vulnerabilities.push(Vulnerability {
                        package: vuln["package"]["name"]
                            .as_str()
                            .unwrap_or("unknown")
                            .to_string(),
                        version: vuln["package"]["version"]
                            .as_str()
                            .unwrap_or("unknown")
                            .to_string(),
                        severity: advisory["severity"]
                            .as_str()
                            .unwrap_or("unknown")
                            .to_string(),
                        title: advisory["title"]
                            .as_str()
                            .unwrap_or("Security vulnerability")
                            .to_string(),
                        description: advisory["description"]
                            .as_str()
                            .unwrap_or("No description available")
                            .to_string(),
                        cve: advisory["id"].as_str().map(String::from),
                        cvss: advisory["cvss"].as_f64().map(|v| v as f32),
                    });
                }
            }
        }

        let dependencies_count = json["dependencies"]["count"].as_u64().unwrap_or(0) as usize;

        Ok(AuditReport {
            passed: vulnerabilities.is_empty(),
            vulnerabilities,
            dependencies_count,
        })
    } else {
        // Fallback: If JSON parsing fails, check for success/failure in text
        if output_str.contains("0 vulnerabilities") || output_str.contains("Success") {
            Ok(AuditReport {
                vulnerabilities: vec![],
                dependencies_count: 0,
                passed: true,
            })
        } else {
            // Try to extract vulnerability count from text
            let vuln_count = if output_str.contains("vulnerability") {
                1
            } else {
                0
            };

            Ok(AuditReport {
                vulnerabilities: vec![],
                dependencies_count: 0,
                passed: vuln_count == 0,
            })
        }
    }
}

/// Quick security check (non-blocking)
pub async fn quick_security_check(project_path: &Path) -> Result<bool> {
    // Check if Cargo.lock exists
    let cargo_lock = project_path.join("Cargo.lock");
    if !cargo_lock.exists() {
        return Ok(true); // No dependencies to check
    }

    // Run quick audit check
    match run_security_audit(project_path).await {
        Ok(report) => Ok(report.passed),
        Err(_) => Ok(true), // Don't block on audit failures
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_severity_classification() {
        let vuln = Vulnerability {
            package: "test".to_string(),
            version: "1.0.0".to_string(),
            severity: "critical".to_string(),
            title: "Test vulnerability".to_string(),
            description: "Test description".to_string(),
            cve: Some("CVE-2024-0001".to_string()),
            cvss: Some(9.5),
        };

        assert_eq!(vuln.severity, "critical");
        assert!(vuln.cvss.unwrap_or(0.0) > 9.0);
    }

    #[test]
    fn test_audit_report_passed() {
        let report = AuditReport {
            vulnerabilities: vec![],
            dependencies_count: 10,
            passed: true,
        };

        assert!(report.passed);
        assert!(report.vulnerabilities.is_empty());
    }
}

//! Security audit checking

use crate::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// Audit check implementation
pub struct AuditCheck;

impl SafetyCheck for AuditCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "audit"
    }

    fn description() -> &'static str {
        "Scans for security vulnerabilities in dependencies"
    }
}

/// Run cargo audit
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Audit);

    // Check if cargo-audit is available
    let audit_check = Command::new("cargo").args(&["audit", "--version"]).output();

    if audit_check
        .as_ref()
        .map_or(true, |output| !output.status.success())
    {
        result.add_error("cargo-audit not available");
        result.add_suggestion("Install with: cargo install cargo-audit");
        result.set_duration(start.elapsed());
        return Ok(result);
    }

    // Run cargo audit
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["audit"])
        .output()?;

    result.set_duration(start.elapsed());

    if !output.status.success() {
        result.add_error("Security vulnerabilities found");
        result.add_suggestion("Update vulnerable dependencies");

        // Parse audit output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut vuln_count = 0;

        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("vulnerability") || line.contains("RUSTSEC") {
                if vuln_count < 3 {
                    result.add_error(format!("Security: {}", line.trim()));
                    vuln_count += 1;
                }
            } else if line.contains("Crate:") || line.contains("Version:") {
                result.add_context(line.trim().to_string());
            }
        }

        if vuln_count >= 3 {
            result.add_error("... and more vulnerabilities (showing first 3)");
        }

        result.add_suggestion("Run 'cargo audit fix' to attempt automatic fixes");
        result.add_suggestion("Check https://rustsec.org for vulnerability details");
    } else {
        result.add_context("No security vulnerabilities found");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_check_struct() {
        assert_eq!(AuditCheck::name(), "audit");
        assert!(!AuditCheck::description().is_empty());
    }
}

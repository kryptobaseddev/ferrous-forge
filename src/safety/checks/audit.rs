//! Security audit checking

use crate::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{CheckType, report::CheckResult};

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
    if let Err(error_msg) = check_audit_availability() {
        result.add_error(&error_msg);
        result.add_suggestion("Install with: cargo install cargo-audit");
        result.set_duration(start.elapsed());
        return Ok(result);
    }

    // Execute audit and process results
    let output = execute_audit(project_path)?;
    result.set_duration(start.elapsed());

    if output.status.success() {
        result.add_context("No security vulnerabilities found");
    } else {
        process_audit_failures(&mut result, &output);
    }

    Ok(result)
}

/// Check if cargo-audit is available on the system
fn check_audit_availability() -> std::result::Result<(), String> {
    let audit_check = Command::new("cargo").args(&["audit", "--version"]).output();

    if audit_check
        .as_ref()
        .map_or(true, |output| !output.status.success())
    {
        Err("cargo-audit not available".to_string())
    } else {
        Ok(())
    }
}

/// Execute cargo audit command
fn execute_audit(project_path: &Path) -> Result<std::process::Output> {
    Command::new("cargo")
        .current_dir(project_path)
        .args(&["audit"])
        .output()
        .map_err(Into::into)
}

/// Process audit command failures and parse vulnerabilities
fn process_audit_failures(result: &mut CheckResult, output: &std::process::Output) {
    result.add_error("Security vulnerabilities found");
    result.add_suggestion("Update vulnerable dependencies");

    parse_audit_output(result, output);
    add_remediation_suggestions(result);
}

/// Parse audit output and extract vulnerability information
fn parse_audit_output(result: &mut CheckResult, output: &std::process::Output) {
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
}

/// Add remediation suggestions for audit failures
fn add_remediation_suggestions(result: &mut CheckResult) {
    result.add_suggestion("Run 'cargo audit fix' to attempt automatic fixes");
    result.add_suggestion("Check https://rustsec.org for vulnerability details");
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_check_struct() {
        assert_eq!(AuditCheck::name(), "audit");
        assert!(!AuditCheck::description().is_empty());
    }
}

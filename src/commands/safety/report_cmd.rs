//! Safety report command implementation
//!
//! Displays recent safety check reports with filtering and formatting options.
//!
//! @task T019
//! @epic T014

use crate::{
    Error, Result, safety::bypass::BypassLogEntry, safety::bypass::BypassManager,
    safety::config::BypassConfig, safety::report::SafetyReport,
};
use console::style;
use std::path::PathBuf;
use tokio::fs;

/// Handle report command
///
/// Displays recent safety reports with optional filtering.
///
/// # Errors
///
/// Returns an error if reports cannot be read or parsed.
pub async fn handle_report(
    limit: usize,
    include_audit: bool,
    stage_filter: Option<String>,
) -> Result<()> {
    println!("{}", style("📊 Safety Reports").bold());
    println!("{}", "=".repeat(60));
    println!();

    // Load and display safety reports
    let reports = load_recent_reports(limit).await?;

    if reports.is_empty() {
        println!("{}", style("No safety reports found.").dim());
        println!();
        println!("Reports are generated when safety checks run.");
        println!("Try running: ferrous-forge safety check --stage=pre-commit");
    } else {
        let filtered_reports: Vec<_> = if let Some(ref stage) = stage_filter {
            reports
                .into_iter()
                .filter(|r| r.stage.name().to_lowercase() == stage.to_lowercase())
                .collect()
        } else {
            reports
        };

        println!("Showing last {} safety reports:\n", filtered_reports.len());

        for (i, report) in filtered_reports.iter().enumerate() {
            print_report_summary(i + 1, report);
        }
    }

    // Include audit log if requested
    if include_audit {
        println!();
        print_audit_section(limit).await?;
    }

    Ok(())
}

/// Load recent safety reports from the reports directory
///
/// # Errors
///
/// Returns an error if the reports directory cannot be read.
async fn load_recent_reports(limit: usize) -> Result<Vec<SafetyReport>> {
    let reports_dir = match get_reports_dir() {
        Ok(dir) => dir,
        Err(_) => return Ok(Vec::new()),
    };

    if !reports_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&reports_dir)
        .await
        .map_err(|e| Error::config(format!("Failed to read reports directory: {}", e)))?;

    let mut reports = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && let Ok(contents) = fs::read_to_string(&path).await
            && let Ok(report) = serde_json::from_str::<SafetyReport>(&contents)
        {
            reports.push(report);
        }
    }

    // Sort by timestamp (newest first) and limit
    reports.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
    reports.truncate(limit);

    Ok(reports)
}

/// Print a summary of a safety report
fn print_report_summary(index: usize, report: &SafetyReport) {
    let status = if report.passed {
        style("✅ PASSED").green()
    } else {
        style("❌ FAILED").red()
    };

    let stage_name = report.stage.display_name();
    let timestamp = report.timestamp.format("%Y-%m-%d %H:%M:%S UTC");

    println!("{}. {} - {}", index, status, style(stage_name).cyan());
    println!("   Time:     {}", style(timestamp).dim());
    println!("   Duration: {:.2}s", report.total_duration.as_secs_f64());
    println!(
        "   Checks:   {} passed, {} failed",
        style(report.checks.len() - report.failed_checks().len()).green(),
        style(report.failed_checks().len()).red()
    );

    // Show failed checks if any
    let failed = report.failed_checks();
    if !failed.is_empty() {
        println!("   {}:", style("Failed checks").red());
        for check in failed {
            println!("     • {}", style(check.check_type.display_name()).red());
        }
    }

    println!();
}

/// Print audit section with bypass log entries
///
/// # Errors
///
/// Returns an error if the audit log cannot be read.
async fn print_audit_section(limit: usize) -> Result<()> {
    println!("{}", style("🔒 Bypass Audit Log").bold());
    println!("{}", "=".repeat(60));
    println!();

    let config = BypassConfig::load_or_default().await?;
    let manager = BypassManager::new(&config)?;
    let entries = manager.get_audit_log(limit).await?;

    if entries.is_empty() {
        println!("{}", style("No bypass entries found.").dim());
    } else {
        println!("Showing last {} entries:\n", entries.len());

        for (i, entry) in entries.iter().enumerate() {
            print_audit_entry(i + 1, entry);
        }
    }

    Ok(())
}

/// Print a single audit entry
fn print_audit_entry(index: usize, entry: &BypassLogEntry) {
    let status = if entry.successful {
        style("✓ SUCCESS").green()
    } else {
        style("✗ FAILED").red()
    };

    println!("{}. {}", index, status);
    println!("   Stage:    {}", style(entry.stage.display_name()).cyan());
    println!("   User:     {}", style(&entry.user).yellow());
    println!(
        "   Time:     {}",
        style(entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC")).dim()
    );
    println!("   Reason:   {}", entry.reason);
    println!();
}

/// Get the reports directory path
fn get_reports_dir() -> Result<PathBuf> {
    let config_dir = crate::config::Config::config_dir_path()?;
    Ok(config_dir.join("safety-reports"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::safety::{CheckType, PipelineStage, report::CheckResult};
    use std::time::Duration;

    #[test]
    fn test_print_report_summary_passed() {
        let mut report = SafetyReport::new(PipelineStage::PreCommit);
        report.passed = true;
        report.total_duration = Duration::from_secs(5);

        let check = CheckResult::new(CheckType::Format);
        report.add_check(check);

        // Should not panic
        print_report_summary(1, &report);
    }

    #[test]
    fn test_print_report_summary_failed() {
        let mut report = SafetyReport::new(PipelineStage::PreCommit);
        let mut check = CheckResult::new(CheckType::Clippy);
        check.add_error("Test error");
        report.add_check(check);

        // Should not panic
        print_report_summary(1, &report);
    }
}

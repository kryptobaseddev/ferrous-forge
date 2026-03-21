//! Safety stats command implementation
//!
//! Displays safety pipeline metrics, trends, and statistics.
//!
//! @task T019
//! @epic T014

use crate::{
    Error, Result,
    safety::PipelineStage,
    safety::bypass::BypassManager,
    safety::config::{BypassConfig, SafetyConfig},
    safety::report::SafetyReport,
};
use chrono::{Duration, Utc};
use console::style;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Safety pipeline statistics
#[derive(Debug, Default)]
pub struct SafetyStats {
    /// Total number of reports
    pub total_reports: usize,
    /// Number of passed reports
    pub passed_reports: usize,
    /// Number of failed reports
    pub failed_reports: usize,
    /// Reports by stage
    pub reports_by_stage: HashMap<String, usize>,
    /// Pass rate by stage (0.0 - 1.0)
    pub pass_rate_by_stage: HashMap<String, f64>,
    /// Total bypass count
    pub total_bypasses: usize,
    /// Bypasses by stage
    pub bypasses_by_stage: HashMap<String, usize>,
    /// Bypasses by user
    pub bypasses_by_user: HashMap<String, usize>,
    /// Bypasses in last 24 hours
    pub bypasses_last_24h: usize,
    /// Bypasses in last 7 days
    pub bypasses_last_7d: usize,
    /// Average checks per report
    pub avg_checks_per_report: f64,
    /// Average report duration
    pub avg_report_duration_secs: f64,
    /// Success rate trend (last 7 days vs previous 7 days)
    pub trend_7d: f64,
}

/// Handle stats command
///
/// Displays comprehensive safety pipeline statistics.
///
/// # Errors
///
/// Returns an error if statistics cannot be computed.
pub async fn handle_stats(days: u32) -> Result<()> {
    println!("{}", style("📈 Safety Pipeline Statistics").bold());
    println!("{}", "=".repeat(60));
    println!();

    let stats = compute_stats(days).await?;

    // Overall Statistics
    println!("{}", style("Overall Statistics:").cyan().bold());
    println!(
        "  Total Reports:      {}",
        style(stats.total_reports).yellow()
    );
    println!(
        "  Passed:             {} ({}%)",
        style(stats.passed_reports).green(),
        calculate_percentage(stats.passed_reports, stats.total_reports)
    );
    println!(
        "  Failed:             {} ({}%)",
        style(stats.failed_reports).red(),
        calculate_percentage(stats.failed_reports, stats.total_reports)
    );
    if stats.total_reports > 0 {
        println!(
            "  Overall Pass Rate:  {:.1}%",
            (stats.passed_reports as f64 / stats.total_reports as f64) * 100.0
        );
    }
    println!();

    // Reports by Stage
    println!("{}", style("Reports by Stage:").cyan().bold());
    for stage in &[
        PipelineStage::PreCommit,
        PipelineStage::PrePush,
        PipelineStage::Publish,
    ] {
        let stage_name = stage.name();
        let count = stats.reports_by_stage.get(stage_name).copied().unwrap_or(0);
        let pass_rate = stats
            .pass_rate_by_stage
            .get(stage_name)
            .copied()
            .unwrap_or(0.0);

        let pass_rate_color = if pass_rate >= 0.9 {
            style(format!("{:.1}%", pass_rate * 100.0)).green()
        } else if pass_rate >= 0.7 {
            style(format!("{:.1}%", pass_rate * 100.0)).yellow()
        } else {
            style(format!("{:.1}%", pass_rate * 100.0)).red()
        };

        println!(
            "  {:12} {:>4} reports (pass rate: {})",
            style(stage_name).cyan(),
            style(count).yellow(),
            pass_rate_color
        );
    }
    println!();

    // Performance Metrics
    println!("{}", style("Performance Metrics:").cyan().bold());
    println!("  Avg Checks/Report:  {:.1}", stats.avg_checks_per_report);
    println!(
        "  Avg Duration:       {:.2}s",
        stats.avg_report_duration_secs
    );
    println!();

    // Bypass Statistics
    println!("{}", style("Bypass Statistics:").cyan().bold());
    println!(
        "  Total Bypasses:     {}",
        style(stats.total_bypasses).yellow()
    );
    println!(
        "  Last 24 Hours:      {}",
        style(stats.bypasses_last_24h).yellow()
    );
    println!(
        "  Last 7 Days:        {}",
        style(stats.bypasses_last_7d).yellow()
    );

    if stats.total_bypasses > 0 {
        println!();
        println!("  Bypasses by Stage:");
        for stage in &[
            PipelineStage::PreCommit,
            PipelineStage::PrePush,
            PipelineStage::Publish,
        ] {
            let stage_name = stage.name();
            let count = stats
                .bypasses_by_stage
                .get(stage_name)
                .copied()
                .unwrap_or(0);
            if count > 0 {
                println!(
                    "    {:12} {}",
                    style(stage_name).cyan(),
                    style(count).yellow()
                );
            }
        }

        // Top bypass users
        if !stats.bypasses_by_user.is_empty() {
            println!();
            println!("  Top Bypass Users:");
            let mut user_counts: Vec<_> = stats.bypasses_by_user.iter().collect();
            user_counts.sort_by(|a, b| b.1.cmp(a.1));
            for (user, count) in user_counts.iter().take(5) {
                println!("    {}: {}", style(user).cyan(), style(count).yellow());
            }
        }
    }
    println!();

    // Trends
    println!("{}", style("Trends (7-day):").cyan().bold());
    let trend_emoji = if stats.trend_7d > 0.05 {
        "📈"
    } else if stats.trend_7d < -0.05 {
        "📉"
    } else {
        "➡️"
    };
    println!(
        "  Pass Rate Change:   {} {:.1}%",
        trend_emoji,
        stats.trend_7d * 100.0
    );
    println!();

    // Configuration Summary
    let config = SafetyConfig::load_or_default().await?;
    println!("{}", style("Configuration Summary:").cyan().bold());
    println!(
        "  Strict Mode:        {}",
        if config.strict_mode {
            style("enabled").green()
        } else {
            style("disabled").red()
        }
    );
    println!(
        "  Bypass System:      {}",
        if config.bypass.enabled {
            style("enabled").green()
        } else {
            style("disabled").red()
        }
    );
    println!(
        "  Max Bypasses/Day:   {}",
        style(config.bypass.max_bypasses_per_day).yellow()
    );
    println!();

    // Recommendations
    print_recommendations(&stats);

    Ok(())
}

/// Compute safety pipeline statistics
///
/// # Errors
///
/// Returns an error if reports or audit logs cannot be read.
async fn compute_stats(days: u32) -> Result<SafetyStats> {
    let mut stats = SafetyStats::default();

    // Load all reports
    let reports = load_all_reports().await?;
    let cutoff_date = Utc::now() - Duration::days(days as i64);

    // Filter reports within time window
    let recent_reports: Vec<_> = reports
        .into_iter()
        .filter(|r| r.timestamp >= cutoff_date)
        .collect();

    stats.total_reports = recent_reports.len();

    // Calculate report statistics
    let mut total_checks = 0;
    let mut total_duration_secs = 0.0;

    for report in &recent_reports {
        if report.passed {
            stats.passed_reports += 1;
        } else {
            stats.failed_reports += 1;
        }

        let stage_name = report.stage.name().to_string();
        *stats
            .reports_by_stage
            .entry(stage_name.clone())
            .or_insert(0) += 1;

        total_checks += report.checks.len();
        total_duration_secs += report.total_duration.as_secs_f64();
    }

    // Calculate pass rates by stage
    let mut stage_reports: HashMap<String, (usize, usize)> = HashMap::new(); // (total, passed)
    for report in &recent_reports {
        let stage_name = report.stage.name().to_string();
        let entry = stage_reports.entry(stage_name).or_insert((0, 0));
        entry.0 += 1;
        if report.passed {
            entry.1 += 1;
        }
    }

    for (stage, (total, passed)) in stage_reports {
        let pass_rate = if total > 0 {
            passed as f64 / total as f64
        } else {
            0.0
        };
        stats.pass_rate_by_stage.insert(stage, pass_rate);
    }

    // Calculate averages
    if stats.total_reports > 0 {
        stats.avg_checks_per_report = total_checks as f64 / stats.total_reports as f64;
        stats.avg_report_duration_secs = total_duration_secs / stats.total_reports as f64;
    }

    // Load bypass statistics
    let config = BypassConfig::load_or_default().await?;
    let manager = BypassManager::new(&config)?;
    let bypass_entries = manager.get_audit_log(1000).await?;

    stats.total_bypasses = bypass_entries.len();

    let now = Utc::now();
    let last_24h = now - Duration::hours(24);
    let last_7d = now - Duration::days(7);
    let prev_7d_start = now - Duration::days(14);

    let mut current_7d_passed = 0;
    let mut current_7d_total = 0;
    let mut prev_7d_passed = 0;
    let mut prev_7d_total = 0;

    for entry in &bypass_entries {
        let stage_name = entry.stage.name().to_string();
        *stats.bypasses_by_stage.entry(stage_name).or_insert(0) += 1;
        *stats
            .bypasses_by_user
            .entry(entry.user.clone())
            .or_insert(0) += 1;

        if entry.timestamp >= last_24h {
            stats.bypasses_last_24h += 1;
        }

        if entry.timestamp >= last_7d {
            stats.bypasses_last_7d += 1;
        }
    }

    // Calculate trend
    for report in &recent_reports {
        if report.timestamp >= last_7d {
            current_7d_total += 1;
            if report.passed {
                current_7d_passed += 1;
            }
        } else if report.timestamp >= prev_7d_start {
            prev_7d_total += 1;
            if report.passed {
                prev_7d_passed += 1;
            }
        }
    }

    let current_rate = if current_7d_total > 0 {
        current_7d_passed as f64 / current_7d_total as f64
    } else {
        0.0
    };

    let prev_rate = if prev_7d_total > 0 {
        prev_7d_passed as f64 / prev_7d_total as f64
    } else {
        0.0
    };

    stats.trend_7d = current_rate - prev_rate;

    Ok(stats)
}

/// Load all safety reports from disk
///
/// # Errors
///
/// Returns an error if reports cannot be read.
async fn load_all_reports() -> Result<Vec<SafetyReport>> {
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
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(contents) = fs::read_to_string(&path).await {
                if let Ok(report) = serde_json::from_str::<SafetyReport>(&contents) {
                    reports.push(report);
                }
            }
        }
    }

    Ok(reports)
}

/// Calculate percentage with safe division
fn calculate_percentage(part: usize, total: usize) -> String {
    if total == 0 {
        "0.0".to_string()
    } else {
        format!("{:.1}", (part as f64 / total as f64) * 100.0)
    }
}

/// Print recommendations based on statistics
fn print_recommendations(stats: &SafetyStats) {
    println!("{}", style("💡 Recommendations:").cyan().bold());

    let mut recommendations = Vec::new();

    if (stats.passed_reports as f64 / stats.total_reports.max(1) as f64) < 0.7 {
        recommendations.push(
            "Pass rate is below 70%. Consider reviewing failing checks or adjusting strictness.",
        );
    }

    if stats.bypasses_last_24h > 5 {
        recommendations
            .push("High bypass activity in last 24h. Investigate root causes of bypasses.");
    }

    if stats.trend_7d < -0.1 {
        recommendations
            .push("Pass rate is declining. Review recent changes that may be causing failures.");
    }

    if stats.avg_report_duration_secs > 60.0 {
        recommendations.push("Average check duration is high (>60s). Consider optimizing checks or enabling parallel execution.");
    }

    if recommendations.is_empty() {
        println!("  ✅ Safety pipeline is performing well!");
    } else {
        for rec in recommendations {
            println!("  • {}", rec);
        }
    }
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

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(50, 100), "50.0");
        assert_eq!(calculate_percentage(0, 100), "0.0");
        assert_eq!(calculate_percentage(75, 0), "0.0");
    }

    #[test]
    fn test_safety_stats_default() {
        let stats = SafetyStats::default();
        assert_eq!(stats.total_reports, 0);
        assert_eq!(stats.passed_reports, 0);
        assert_eq!(stats.failed_reports, 0);
    }
}

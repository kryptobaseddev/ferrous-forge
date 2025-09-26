//! Edition analyze command

use crate::Result;
use crate::edition::{Edition, EditionAnalyzer};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

/// Handle edition analyze command
pub async fn handle_analyze(path: &Path, edition_str: &str) -> Result<()> {
    let target_edition = Edition::parse_edition(edition_str)?;

    print_analysis_header(path, &target_edition);

    let report = run_analysis_with_progress(path, target_edition).await?;

    display_analysis_results(&report);
    display_analysis_sections(&report);
    display_migration_readiness(&report, edition_str);

    Ok(())
}

/// Print the analysis header with project and target information
fn print_analysis_header(path: &Path, target_edition: &Edition) {
    println!("🔍 Edition Compatibility Analysis\n");
    println!("  Project:  {}", style(path.display()).dim());
    println!("  Target:   {}", style(target_edition.to_string()).cyan());
}

/// Run the analysis with a progress spinner
async fn run_analysis_with_progress(
    path: &Path,
    target_edition: Edition,
) -> Result<crate::edition::analyzer::AnalysisReport> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message("Analyzing project...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let analyzer = EditionAnalyzer::new(path);
    let report = analyzer.analyze(target_edition).await?;

    spinner.finish_and_clear();
    Ok(report)
}

/// Display the main analysis results summary
fn display_analysis_results(report: &crate::edition::analyzer::AnalysisReport) {
    println!("\n📊 Analysis Results\n");
    println!("  Files analyzed: {}", style(report.total_files).cyan());
    println!(
        "  Issues found:   {}",
        if report.issues.is_empty() {
            style(0).green()
        } else {
            style(report.issues.len()).yellow()
        }
    );
    println!(
        "  Warnings:       {}",
        if report.warnings.is_empty() {
            style(0).green()
        } else {
            style(report.warnings.len()).yellow()
        }
    );
}

/// Display all analysis sections (issues, warnings, suggestions)
fn display_analysis_sections(report: &crate::edition::analyzer::AnalysisReport) {
    display_issues(&report.issues);
    display_warnings(&report.warnings);
    display_suggestions(&report.suggestions);
}

/// Display issues found during analysis
fn display_issues(issues: &[crate::edition::analyzer::EditionIssue]) {
    if issues.is_empty() {
        return;
    }

    println!("\n⚠️  Issues to address:");
    for issue in issues {
        let severity_style = match issue.severity {
            crate::edition::analyzer::Severity::Error => style("ERROR").red(),
            crate::edition::analyzer::Severity::Warning => style("WARN").yellow(),
            crate::edition::analyzer::Severity::Info => style("INFO").blue(),
        };

        print!("  [{:>5}]", severity_style);

        if let Some(file) = &issue.file {
            print!(" {}", file);
            if let Some(line) = issue.line {
                print!(":{}", line);
            }
        }

        println!("\n    {}", issue.message);
    }
}

/// Display general warnings
fn display_warnings(warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    println!("\n📝 General warnings:");
    for warning in warnings {
        println!("  • {}", warning);
    }
}

/// Display suggestions for improvement
fn display_suggestions(suggestions: &[String]) {
    if suggestions.is_empty() {
        return;
    }

    println!("\n💡 Suggestions:");
    for suggestion in suggestions {
        println!("  • {}", style(suggestion).cyan());
    }
}

/// Display migration readiness status and instructions
fn display_migration_readiness(
    report: &crate::edition::analyzer::AnalysisReport,
    edition_str: &str,
) {
    println!();
    if report.is_ready_for_migration() {
        println!(
            "{}",
            style("✅ Project is ready for migration!").green().bold()
        );
        println!(
            "Run {} to start",
            style(format!("ferrous-forge edition migrate {}", edition_str)).cyan()
        );
    } else {
        println!(
            "{}",
            style("⚠️  Project needs attention before migration")
                .yellow()
                .bold()
        );
        println!("Fix the errors listed above before attempting migration.");
    }
}

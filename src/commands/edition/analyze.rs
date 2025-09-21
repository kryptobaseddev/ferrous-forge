//! Edition analyze command

use crate::edition::{Edition, EditionAnalyzer};
use crate::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

/// Handle edition analyze command
pub async fn handle_analyze(path: &Path, edition_str: &str) -> Result<()> {
    let target_edition = Edition::parse_edition(edition_str)?;

    println!("🔍 Edition Compatibility Analysis\n");
    println!("  Project:  {}", style(path.display()).dim());
    println!("  Target:   {}", style(target_edition.to_string()).cyan());

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

    // Display issues
    if !report.issues.is_empty() {
        println!("\n⚠️  Issues to address:");
        for issue in &report.issues {
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

    // Display warnings
    if !report.warnings.is_empty() {
        println!("\n📝 General warnings:");
        for warning in &report.warnings {
            println!("  • {}", warning);
        }
    }

    // Display suggestions
    if !report.suggestions.is_empty() {
        println!("\n💡 Suggestions:");
        for suggestion in &report.suggestions {
            println!("  • {}", style(suggestion).cyan());
        }
    }

    // Migration readiness
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

    Ok(())
}

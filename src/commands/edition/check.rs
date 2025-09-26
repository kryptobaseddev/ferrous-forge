//! Edition check command

use crate::Result;
use crate::edition::{check_compliance, get_migration_recommendations};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

/// Handle edition check command
pub async fn handle_check(path: &Path) -> Result<()> {
    let status = run_compliance_check_with_progress(path).await?;

    display_compliance_header(path, &status);
    display_edition_status(&status);
    display_migration_status(&status);
    display_recommendations(&status);

    Ok(())
}

/// Run compliance check with progress indicator
async fn run_compliance_check_with_progress(path: &Path) -> Result<crate::edition::EditionStatus> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message("Checking edition compliance...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let status = check_compliance(path).await?;
    spinner.finish_and_clear();

    Ok(status)
}

/// Display compliance check header information
fn display_compliance_header(path: &Path, status: &crate::edition::EditionStatus) {
    println!("📚 Edition Compliance Status\n");
    println!("  Project:  {}", style(path.display()).dim());
    println!(
        "  Manifest: {}",
        style(status.manifest_path.display()).dim()
    );
    println!();
}

/// Display current and latest edition status
fn display_edition_status(status: &crate::edition::EditionStatus) {
    let current_style = if status.is_latest {
        style(status.current.to_string()).green()
    } else {
        style(status.current.to_string()).yellow()
    };

    println!("  Current:  {}", current_style);
    println!("  Latest:   {}", style(status.latest.to_string()).green());
    println!();
}

/// Display migration status and path if available
fn display_migration_status(status: &crate::edition::EditionStatus) {
    if status.is_latest {
        println!(
            "{}",
            style("✅ Your project is using the latest edition!")
                .green()
                .bold()
        );
    } else {
        println!(
            "{}",
            style("⚠️  An edition update is available").yellow().bold()
        );

        if !status.migration_path.is_empty() {
            println!("\nMigration path:");
            let path_str = status
                .migration_path
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(" → ");
            println!("  {} → {}", status.current, style(path_str).cyan());
        }
    }
}

/// Display migration recommendations
fn display_recommendations(status: &crate::edition::EditionStatus) {
    println!("\n📋 Recommendations:");
    for recommendation in get_migration_recommendations(status) {
        println!("  • {}", recommendation);
    }
}

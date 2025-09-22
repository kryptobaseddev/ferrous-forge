//! Rust version management commands

use crate::rust_version::{UpdateRecommendation, VersionManager};
use crate::{Error, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use semver::Version;

/// Handle rust version check command
pub async fn handle_check(verbose: bool) -> Result<()> {
    let spinner = create_spinner("Checking Rust version...");
    let manager = VersionManager::new()?;

    let current = manager.check_current().await?;
    spinner.set_message("Fetching latest release information...");

    let latest = match fetch_latest_version(&manager, &spinner, &current).await {
        Some(release) => release,
        None => return Ok(()), // Error already handled in fetch_latest_version
    };

    spinner.finish_and_clear();

    display_version_status(&current, &latest, verbose);
    let recommendation = manager.get_recommendation().await?;
    display_recommendation(&recommendation);

    Ok(())
}

/// Create a styled progress spinner
fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

/// Fetch the latest Rust version, handling errors gracefully
async fn fetch_latest_version(
    manager: &VersionManager,
    spinner: &ProgressBar,
    current: &crate::rust_version::RustVersion,
) -> Option<crate::rust_version::GitHubRelease> {
    match manager.get_latest_stable().await {
        Ok(release) => Some(release),
        Err(Error::RateLimited(retry_after)) => {
            spinner.finish_and_clear();
            println!(
                "{}",
                style(format!(
                    "⚠️  GitHub API rate limited. Please try again in {} seconds.",
                    retry_after
                ))
                .yellow()
            );
            None
        }
        Err(e) => {
            spinner.finish_and_clear();
            println!(
                "{}",
                style(format!(
                    "⚠️  Could not fetch latest release: {}. Using cached data if available.",
                    e
                ))
                .yellow()
            );
            println!("\nCurrent version: {}", style(current.to_string()).cyan());
            None
        }
    }
}

/// Display current and latest version information
fn display_version_status(
    current: &crate::rust_version::RustVersion,
    latest: &crate::rust_version::GitHubRelease,
    verbose: bool,
) {
    println!("🦀 Rust Version Status\n");
    println!(
        "  Current: {}",
        style(format!("rustc {}", current.version)).cyan()
    );
    println!("  Channel: {}", style(current.channel.to_string()).dim());
    println!("  Host:    {}", style(&current.host).dim());

    if verbose {
        println!("  Commit:  {}", style(&current.commit_hash).dim());
        println!(
            "  Date:    {}",
            style(current.commit_date.to_string()).dim()
        );
    }

    println!(
        "\n  Latest:  {}",
        style(format!("rustc {}", latest.version)).green()
    );
}

/// Display update recommendation based on analysis
fn display_recommendation(recommendation: &UpdateRecommendation) {
    println!();
    match recommendation {
        UpdateRecommendation::UpToDate => {
            println!("{}", style("✅ You're up to date!").green().bold());
        }
        UpdateRecommendation::MinorUpdate {
            current: _,
            latest,
            release_url,
        } => {
            display_minor_update(latest, release_url);
        }
        UpdateRecommendation::MajorUpdate {
            current: _,
            latest,
            release_url,
        } => {
            display_major_update(latest, release_url);
        }
        UpdateRecommendation::SecurityUpdate {
            current: _,
            latest,
            release_url,
            details,
        } => {
            display_security_update(latest, release_url, details);
        }
    }
}

/// Display minor update information
fn display_minor_update(latest: &Version, release_url: &str) {
    println!("{}", style("📦 Update available!").yellow().bold());
    println!("\n  Version {} is now available", style(latest).yellow());
    println!("  Release: {}", release_url);
    println!("\n  Run {} to update", style("rustup update stable").cyan());
}

/// Display major update information
fn display_major_update(latest: &Version, release_url: &str) {
    println!("{}", style("🚀 Major update available!").yellow().bold());
    println!(
        "\n  Version {} includes major changes",
        style(latest).yellow()
    );
    println!("  Release: {}", release_url);
    println!("\n  Review release notes before updating:");
    println!("  {}", release_url);
    println!("\n  Run {} to update", style("rustup update stable").cyan());
}

/// Display security update information
fn display_security_update(latest: &Version, release_url: &str, details: &str) {
    println!("{}", style("🚨 SECURITY UPDATE AVAILABLE!").red().bold());
    println!(
        "\n  Version {} includes security fixes",
        style(latest).red()
    );

    if !details.is_empty() {
        println!("\n  Security details:");
        for line in details.lines() {
            println!("    {}", line);
        }
    }

    println!("\n  Release: {}", release_url);
    println!(
        "\n  {} {}",
        style("Strongly recommended:").red().bold(),
        style("rustup update stable").cyan()
    );
}

/// Handle rust version recommend command
pub async fn handle_recommend(stable_only: bool) -> Result<()> {
    let spinner = create_spinner("Analyzing Rust versions...");
    let manager = VersionManager::new()?;
    let current = manager.check_current().await?;
    let recommendation = manager.get_recommendation().await?;

    spinner.finish_and_clear();

    display_recommendation_header(&current);
    display_recommendation_details(&recommendation, stable_only);
    display_general_recommendations();

    Ok(())
}

/// Display the recommendation header with current version
fn display_recommendation_header(current: &crate::rust_version::RustVersion) {
    println!("🎯 Rust Version Recommendations\n");
    println!(
        "Current version: {}",
        style(format!("{}", current.version)).cyan()
    );
}

/// Display specific recommendation based on update status
fn display_recommendation_details(recommendation: &UpdateRecommendation, stable_only: bool) {
    match recommendation {
        UpdateRecommendation::UpToDate => {
            display_up_to_date_recommendation(stable_only);
        }
        UpdateRecommendation::MinorUpdate { latest, .. }
        | UpdateRecommendation::MajorUpdate { latest, .. } => {
            display_update_recommendation(latest);
        }
        UpdateRecommendation::SecurityUpdate { latest, .. } => {
            display_security_recommendation(latest);
        }
    }
}

/// Display recommendation when already up to date
fn display_up_to_date_recommendation(stable_only: bool) {
    println!(
        "\n{}",
        style("✅ Recommendation: Stay on current version")
            .green()
            .bold()
    );
    println!("\nYou're already using the latest stable Rust version.");

    if !stable_only {
        println!("\nFor experimental features, consider:");
        println!(
            "  • {} - Test upcoming features",
            style("beta channel").yellow()
        );
        println!(
            "  • {} - Bleeding edge development",
            style("nightly channel").red()
        );
    }
}

/// Display recommendation to update to newer version
fn display_update_recommendation(latest: &Version) {
    println!(
        "\n{}",
        style("📦 Recommendation: Update to latest stable")
            .yellow()
            .bold()
    );
    println!("\nVersion {} is available with:", style(latest).green());
    println!("  • Bug fixes and performance improvements");
    println!("  • New stable features");
    println!("  • Enhanced diagnostics");

    println!("\nUpdate command:");
    println!("  {}", style("rustup update stable").cyan());
}

/// Display urgent security update recommendation
fn display_security_recommendation(latest: &Version) {
    println!(
        "\n{}",
        style("🚨 URGENT: Security update required").red().bold()
    );
    println!(
        "\nVersion {} contains critical security fixes.",
        style(latest).red()
    );
    println!("Update immediately with:");
    println!("  {}", style("rustup update stable").cyan().bold());
}

/// Display general recommendations for Rust maintenance
fn display_general_recommendations() {
    println!("\n📋 General Recommendations:");
    println!("  • Keep Rust updated for best performance and security");
    println!("  • Review release notes before major updates");
    println!("  • Test your projects after updating");
    println!(
        "  • Use {} for production code",
        style("stable channel").green()
    );
}

/// Handle rust version list command
pub async fn handle_list(count: usize) -> Result<()> {
    let spinner = create_spinner("Fetching Rust releases...");
    let manager = VersionManager::new()?;
    let releases = manager.get_recent_releases(count).await?;

    spinner.finish_and_clear();

    display_releases_list(&releases);
    Ok(())
}

/// Display the list of recent Rust releases
fn display_releases_list(releases: &[crate::rust_version::GitHubRelease]) {
    println!("📜 Recent Rust Releases\n");

    if releases.is_empty() {
        println!("No releases found.");
        return;
    }

    for (i, release) in releases.iter().enumerate() {
        display_release_entry(i, release);
    }
}

/// Display a single release entry with formatting
fn display_release_entry(index: usize, release: &crate::rust_version::GitHubRelease) {
    let version_style = if index == 0 {
        style(format!("{}", release.version)).green().bold()
    } else {
        style(format!("{}", release.version)).cyan()
    };

    let published_date = release
        .published_at
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "Not published".to_string());

    println!(
        "{}. {} - {}",
        index + 1,
        version_style,
        published_date
    );

    if release.name != release.tag_name {
        println!("   {}", style(&release.name).dim());
    }

    display_release_summary(&release.body);
    println!("   {}", style(&release.html_url).blue().underlined());
    println!();
}

/// Display a truncated summary of release notes
fn display_release_summary(body: &str) {
    if let Some(first_line) = body.lines().next() {
        let truncated = if first_line.len() > 70 {
            format!("{}...", &first_line[..70])
        } else {
            first_line.to_string()
        };
        println!("   {}", style(truncated).dim());
    }
}

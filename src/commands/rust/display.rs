//! Display functions for Rust version commands

use crate::rust_version::{GitHubRelease, RustVersion, UpdateRecommendation};
use console::style;
use semver::Version;

/// Display current and latest version status
pub fn display_version_status(current: &RustVersion, latest: &Version, verbose: bool) {
    println!();
    println!("{}", style("🔍 Rust Version Status").bold().cyan());
    println!();

    // Current version
    println!(
        "📦 Current Version: {}",
        style(&current.version).green().bold()
    );

    if verbose {
        println!("   Channel: {}", current.channel);
        println!("   Date: {}", current.commit_date);
        println!("   Commit: {}", &current.commit_hash[..8]);
    }

    // Latest version
    println!("🚀 Latest Version: {}", style(latest).blue().bold());

    if current.version == *latest {
        println!("{}", style("✅ You're using the latest version!").green());
    } else {
        println!("{}", style("⚠️  A newer version is available").yellow());
    }
    println!();
}

/// Display update recommendation
pub fn display_recommendation(recommendation: &UpdateRecommendation) {
    match recommendation {
        UpdateRecommendation::UpToDate => {
            println!("{}", style("✅ No update needed").green().bold());
        }
        UpdateRecommendation::MinorUpdate(info) => {
            display_minor_update(&info.latest, &info.release_url);
        }
        UpdateRecommendation::MajorUpdate(info) => {
            display_major_update(&info.latest, &info.release_url);
        }
        UpdateRecommendation::SecurityUpdate(info) => {
            let security_details = info
                .security_details
                .as_deref()
                .unwrap_or("Security update available");
            display_security_update(&info.latest, &info.release_url, security_details);
        }
    }
}

/// Display minor update recommendation
fn display_minor_update(latest: &Version, release_url: &str) {
    println!("{}", style("💡 Minor update available").yellow().bold());
    println!("   Current: {}", latest);
    println!("   Update command: {}", style("rustup update").cyan());
    println!("   Release notes: {}", release_url);
}

/// Display major update recommendation
fn display_major_update(latest: &Version, release_url: &str) {
    println!("{}", style("🎯 Major update available").blue().bold());
    println!("   Version: {}", style(latest).bold());
    println!();
    println!("   ⚠️  This is a major release that may include breaking changes.");
    println!("   📚 Please review the release notes before updating:");
    println!("   🔗 {}", release_url);
    println!();
    println!("   Update command: {}", style("rustup update").cyan());
}

/// Display security update recommendation
fn display_security_update(latest: &Version, release_url: &str, details: &str) {
    println!("{}", style("🚨 SECURITY UPDATE AVAILABLE").red().bold());
    println!("   Version: {}", style(latest).bold());
    println!();
    println!(
        "   {} {}",
        style("⚠️").red(),
        style("Security Issue:").red().bold()
    );

    // Format details with proper indentation
    for line in details.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            println!("   {}", trimmed);
        }
    }

    println!();
    println!(
        "   {} {}",
        style("🔧").yellow(),
        style("Update immediately with:").yellow().bold()
    );
    println!("   {}", style("rustup update").cyan().bold());
    println!();
    println!("   📚 Full details: {}", release_url);
}

/// Display recommendation header
pub fn display_recommendation_header(current: &RustVersion) {
    println!();
    println!("{}", style("💡 Rust Update Recommendations").bold().cyan());
    println!();
    println!("Current version: {}", style(&current.version).green());
    println!();
}

/// Display recommendation details based on update status
pub fn display_recommendation_details(recommendation: &UpdateRecommendation, stable_only: bool) {
    match recommendation {
        UpdateRecommendation::UpToDate => {
            display_up_to_date_recommendation(stable_only);
        }
        UpdateRecommendation::MinorUpdate(info) | UpdateRecommendation::MajorUpdate(info) => {
            display_update_recommendation(&info.latest);
        }
        UpdateRecommendation::SecurityUpdate(info) => {
            display_security_recommendation(&info.latest);
            if let Some(details) = &info.security_details {
                println!("Security details: {}", details);
            }
        }
    }

    display_general_recommendations();
}

/// Display up-to-date recommendation
fn display_up_to_date_recommendation(stable_only: bool) {
    println!(
        "{}",
        style("✅ You're running the latest version!")
            .green()
            .bold()
    );
    println!();

    if stable_only {
        println!("🔒 Staying on stable releases is recommended for:");
        println!("   • Production environments");
        println!("   • Critical projects");
        println!("   • Maximum stability");
    } else {
        println!("💡 Consider these update channels:");
        println!(
            "   • {} - Most stable, recommended for production",
            style("stable").green()
        );
        println!(
            "   • {} - Preview of next stable release",
            style("beta").yellow()
        );
        println!(
            "   • {} - Latest features, may be unstable",
            style("nightly").red()
        );
    }
    println!();
}

/// Display update recommendation when newer version is available
fn display_update_recommendation(latest: &Version) {
    println!("{}", style("🎯 Update recommended!").blue().bold());
    println!("Latest version: {}", style(latest).bold());
    println!();
    println!("Benefits of updating:");
    println!("   • 🚀 Performance improvements");
    println!("   • 🐛 Bug fixes");
    println!("   • ✨ New language features");
    println!("   • 🔒 Security patches");
    println!();
    println!("Update command: {}", style("rustup update").cyan());
    println!();
}

/// Display security-specific recommendation
fn display_security_recommendation(latest: &Version) {
    println!("{}", style("🚨 SECURITY UPDATE REQUIRED").red().bold());
    println!("Update to: {}", style(latest).bold());
    println!();
    println!(
        "{} {}",
        style("⚠️").red(),
        style("Your current version has known security vulnerabilities.")
    );
    println!("{} {}", style("🔧").yellow(), style("Update immediately:"));
    println!("   {}", style("rustup update").cyan().bold());
    println!();
}

/// Display general recommendations
fn display_general_recommendations() {
    println!("{}", style("📋 General Recommendations:").bold());
    println!("   • Keep Rust updated for security and performance");
    println!(
        "   • Use {} for production environments",
        style("stable").green()
    );
    println!("   • Test updates in development before deploying");
    println!("   • Follow Rust release notes for breaking changes");
    println!();
}

/// Display list of releases
pub fn display_releases_list(releases: &[GitHubRelease]) {
    println!();
    println!("{}", style("📦 Recent Rust Releases").bold().cyan());
    println!();

    for (index, release) in releases.iter().enumerate() {
        display_release_entry(index + 1, release);

        if index < releases.len() - 1 {
            println!();
        }
    }
}

/// Display a single release entry
fn display_release_entry(index: usize, release: &GitHubRelease) {
    let version = &release.tag_name;
    let is_prerelease = release.prerelease;

    let version_style = if is_prerelease {
        style(version).yellow()
    } else {
        style(version).green().bold()
    };

    println!("{}. {}", index, version_style);

    if is_prerelease {
        println!("   {}", style("(Pre-release)").dim());
    }

    if let Some(date) = &release.published_at {
        println!("   Published: {}", style(date).dim());
    }

    if !release.body.is_empty() {
        display_release_summary(&release.body);
    }
}

/// Display a summary of release notes
fn display_release_summary(body: &str) {
    let summary_lines: Vec<&str> = body
        .lines()
        .take(3)
        .filter(|line| !line.trim().is_empty())
        .collect();

    if !summary_lines.is_empty() {
        println!("   Summary:");
        for line in summary_lines {
            let trimmed = line.trim();
            let display_line = if trimmed.len() > 80 {
                format!("{}...", &trimmed[..77])
            } else {
                trimmed.to_string()
            };
            println!("     {}", style(display_line).dim());
        }
    }
}

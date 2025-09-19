//! Rust version management commands

use crate::rust_version::{UpdateRecommendation, VersionManager};
use crate::{Error, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

/// Handle rust version check command
pub async fn handle_check(verbose: bool) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message("Checking Rust version...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    
    let manager = VersionManager::new()?;
    
    // Check current version
    let current = manager.check_current().await?;
    spinner.set_message("Fetching latest release information...");
    
    // Get latest version
    let latest = match manager.get_latest_stable().await {
        Ok(release) => release,
        Err(Error::RateLimited(retry_after)) => {
            spinner.finish_and_clear();
            println!(
                "{}",
                style(format!(
                    "⚠️  GitHub API rate limited. Please try again in {} seconds.",
                    retry_after
                )).yellow()
            );
            return Ok(());
        }
        Err(e) => {
            spinner.finish_and_clear();
            println!(
                "{}",
                style(format!(
                    "⚠️  Could not fetch latest release: {}. Using cached data if available.",
                    e
                )).yellow()
            );
            println!("\nCurrent version: {}", style(current.to_string()).cyan());
            return Ok(());
        }
    };
    
    spinner.finish_and_clear();
    
    // Display results
    println!("🦀 Rust Version Status\n");
    println!("  Current: {}", style(format!("rustc {}", current.version)).cyan());
    println!("  Channel: {}", style(current.channel.to_string()).dim());
    println!("  Host:    {}", style(current.host).dim());
    
    if verbose {
        println!("  Commit:  {}", style(current.commit_hash).dim());
        println!("  Date:    {}", style(current.commit_date.to_string()).dim());
    }
    
    println!("\n  Latest:  {}", style(format!("rustc {}", latest.version)).green());
    
    // Get recommendation
    let recommendation = manager.get_recommendation().await?;
    
    println!();
    match recommendation {
        UpdateRecommendation::UpToDate => {
            println!("{}", style("✅ You're up to date!").green().bold());
        }
        UpdateRecommendation::MinorUpdate { current: _, latest, release_url } => {
            println!("{}", style("📦 Update available!").yellow().bold());
            println!("\n  Version {} is now available", style(latest).yellow());
            println!("  Release: {}", release_url);
            println!("\n  Run {} to update", style("rustup update stable").cyan());
        }
        UpdateRecommendation::MajorUpdate { current: _, latest, release_url } => {
            println!("{}", style("🚀 Major update available!").yellow().bold());
            println!("\n  Version {} includes major changes", style(latest).yellow());
            println!("  Release: {}", release_url);
            println!("\n  Review release notes before updating:");
            println!("  {}", release_url);
            println!("\n  Run {} to update", style("rustup update stable").cyan());
        }
        UpdateRecommendation::SecurityUpdate { current: _, latest, release_url, details } => {
            println!("{}", style("🚨 SECURITY UPDATE AVAILABLE!").red().bold());
            println!("\n  Version {} includes security fixes", style(latest).red());
            
            if !details.is_empty() {
                println!("\n  Security details:");
                for line in details.lines() {
                    println!("    {}", line);
                }
            }
            
            println!("\n  Release: {}", release_url);
            println!("\n  {} {}", 
                style("Strongly recommended:").red().bold(),
                style("rustup update stable").cyan()
            );
        }
    }
    
    Ok(())
}

/// Handle rust version recommend command
pub async fn handle_recommend(stable_only: bool) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message("Analyzing Rust versions...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    
    let manager = VersionManager::new()?;
    let current = manager.check_current().await?;
    let recommendation = manager.get_recommendation().await?;
    
    spinner.finish_and_clear();
    
    println!("🎯 Rust Version Recommendations\n");
    println!("Current version: {}", style(format!("{}", current.version)).cyan());
    
    match recommendation {
        UpdateRecommendation::UpToDate => {
            println!("\n{}", style("✅ Recommendation: Stay on current version").green().bold());
            println!("\nYou're already using the latest stable Rust version.");
            
            if !stable_only {
                println!("\nFor experimental features, consider:");
                println!("  • {} - Test upcoming features", style("beta channel").yellow());
                println!("  • {} - Bleeding edge development", style("nightly channel").red());
            }
        }
        UpdateRecommendation::MinorUpdate { latest, .. } |
        UpdateRecommendation::MajorUpdate { latest, .. } => {
            println!("\n{}", style("📦 Recommendation: Update to latest stable").yellow().bold());
            println!("\nVersion {} is available with:", style(latest).green());
            println!("  • Bug fixes and performance improvements");
            println!("  • New stable features");
            println!("  • Enhanced diagnostics");
            
            println!("\nUpdate command:");
            println!("  {}", style("rustup update stable").cyan());
        }
        UpdateRecommendation::SecurityUpdate { latest, .. } => {
            println!("\n{}", style("🚨 URGENT: Security update required").red().bold());
            println!("\nVersion {} contains critical security fixes.", style(latest).red());
            println!("Update immediately with:");
            println!("  {}", style("rustup update stable").cyan().bold());
        }
    }
    
    // Additional recommendations
    println!("\n📋 General Recommendations:");
    println!("  • Keep Rust updated for best performance and security");
    println!("  • Review release notes before major updates");
    println!("  • Test your projects after updating");
    println!("  • Use {} for production code", style("stable channel").green());
    
    Ok(())
}

/// Handle rust version list command
pub async fn handle_list(count: usize) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message("Fetching Rust releases...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    
    let manager = VersionManager::new()?;
    let releases = manager.get_recent_releases(count).await?;
    
    spinner.finish_and_clear();
    
    println!("📜 Recent Rust Releases\n");
    
    if releases.is_empty() {
        println!("No releases found.");
        return Ok(());
    }
    
    for (i, release) in releases.iter().enumerate() {
        let version_style = if i == 0 {
            style(format!("{}", release.version)).green().bold()
        } else {
            style(format!("{}", release.version)).cyan()
        };
        
        println!("{}. {} - {}", 
            i + 1,
            version_style,
            release.published_at
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "Not published".to_string())
        );
        
        if release.name != release.tag_name {
            println!("   {}", style(&release.name).dim());
        }
        
        // Show first line of release notes
        if let Some(first_line) = release.body.lines().next() {
            let truncated = if first_line.len() > 70 {
                format!("{}...", &first_line[..70])
            } else {
                first_line.to_string()
            };
            println!("   {}", style(truncated).dim());
        }
        
        println!("   {}", style(&release.html_url).blue().underlined());
        println!();
    }
    
    Ok(())
}

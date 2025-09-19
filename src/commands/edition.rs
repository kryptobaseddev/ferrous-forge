//! Edition management commands

use crate::edition::{
    check_compliance, get_migration_recommendations, Edition, EditionAnalyzer, EditionMigrator,
    migrator::MigrationOptions,
};
use crate::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

/// Handle edition check command
pub async fn handle_check(path: &Path) -> Result<()> {
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
    
    println!("📚 Edition Compliance Status\n");
    println!("  Project:  {}", style(path.display()).dim());
    println!("  Manifest: {}", style(status.manifest_path.display()).dim());
    println!();
    
    let current_style = if status.is_latest {
        style(status.current.to_string()).green()
    } else {
        style(status.current.to_string()).yellow()
    };
    
    println!("  Current:  {}", current_style);
    println!("  Latest:   {}", style(status.latest.to_string()).green());
    
    println!();
    
    if status.is_latest {
        println!("{}", style("✅ Your project is using the latest edition!").green().bold());
    } else {
        println!("{}", style("⚠️  An edition update is available").yellow().bold());
        
        if !status.migration_path.is_empty() {
            println!("\nMigration path:");
            let path_str = status.migration_path
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(" → ");
            println!("  {} → {}", status.current, style(path_str).cyan());
        }
    }
    
    println!("\n📋 Recommendations:");
    for recommendation in get_migration_recommendations(&status) {
        println!("  • {}", recommendation);
    }
    
    Ok(())
}

/// Handle edition migrate command
pub async fn handle_migrate(
    edition_str: &str,
    no_backup: bool,
    test: bool,
    idioms: bool,
) -> Result<()> {
    let target_edition = Edition::parse_edition(edition_str)?;
    let project_path = std::env::current_dir()?;
    
    println!("🚀 Edition Migration Assistant\n");
    println!("  Target:   {}", style(target_edition.to_string()).cyan());
    println!("  Project:  {}", style(project_path.display()).dim());
    
    // Check current edition
    let current_edition = crate::edition::detect_edition(&project_path.join("Cargo.toml")).await?;
    
    if current_edition >= target_edition {
        println!("\n{}", style("✅ Already on target edition or newer!").green().bold());
        return Ok(());
    }
    
    println!("  Current:  {}", style(current_edition.to_string()).yellow());
    println!();
    
    // Confirm migration
    println!("This will:");
    println!("  1. {} of your project", if no_backup { 
        style("Skip backup creation").yellow() 
    } else { 
        style("Create a backup").green() 
    });
    println!("  2. Run {} to fix edition issues", style("cargo fix --edition").cyan());
    println!("  3. Update {} to edition {}", style("Cargo.toml").cyan(), target_edition.as_str());
    if test {
        println!("  4. Run {} to verify", style("cargo test").cyan());
    }
    
    println!("\n{}", style("Starting migration...").bold());
    
    let migrator = EditionMigrator::new(&project_path);
    
    let options = MigrationOptions {
        create_backup: !no_backup,
        run_tests: test,
        fix_idioms: idioms,
        ..Default::default()
    };
    
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    
    if options.create_backup {
        spinner.set_message("Creating backup...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    }
    
    let result = migrator.migrate(target_edition, options).await?;
    
    spinner.finish_and_clear();
    
    match result.status {
        crate::edition::migrator::MigrationStatus::Success => {
            println!("\n{}", style("✅ Migration completed successfully!").green().bold());
        }
        crate::edition::migrator::MigrationStatus::PartialSuccess => {
            println!("\n{}", style("⚠️  Migration completed with warnings").yellow().bold());
        }
        crate::edition::migrator::MigrationStatus::AlreadyUpToDate => {
            println!("\n{}", style("✅ Already up to date!").green().bold());
            return Ok(());
        }
        _ => {
            println!("\n{}", style("❌ Migration failed").red().bold());
        }
    }
    
    // Display messages
    if !result.messages.is_empty() {
        println!("\n📝 Messages:");
        for msg in &result.messages {
            println!("  {}", msg);
        }
    }
    
    // Display warnings
    if !result.warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for warning in &result.warnings {
            println!("  • {}", style(warning).yellow());
        }
    }
    
    // Display errors
    if !result.errors.is_empty() {
        println!("\n❌ Errors:");
        for error in &result.errors {
            println!("  • {}", style(error).red());
        }
    }
    
    if let Some(backup_path) = result.backup_path {
        println!("\n💾 Backup saved to: {}", style(backup_path.display()).dim());
    }
    
    println!("\n📋 Next steps:");
    println!("  1. Review the changes made by the migration");
    println!("  2. Run {} to ensure everything compiles", style("cargo build").cyan());
    println!("  3. Run {} to verify functionality", style("cargo test").cyan());
    println!("  4. Commit the changes to version control");
    
    Ok(())
}

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
    println!("  Issues found:   {}", 
        if report.issues.is_empty() {
            style(0).green()
        } else {
            style(report.issues.len()).yellow()
        }
    );
    println!("  Warnings:       {}", 
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
        println!("{}", style("✅ Project is ready for migration!").green().bold());
        println!("\nRun {} to start the migration", 
            style(format!("ferrous-forge edition migrate {}", edition_str)).cyan()
        );
    } else {
        println!("{}", style("⚠️  Project needs attention before migration").yellow().bold());
        println!("\nFix the errors listed above before attempting migration.");
    }
    
    Ok(())
}

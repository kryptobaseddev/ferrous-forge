//! Edition migrate command

use crate::edition::{migrator::MigrationOptions, Edition, EditionMigrator};
use crate::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

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
        println!(
            "\n{}",
            style("✅ Already on target edition or newer!")
                .green()
                .bold()
        );
        return Ok(());
    }

    println!(
        "  Current:  {}",
        style(current_edition.to_string()).yellow()
    );
    println!();

    // Confirm migration
    println!("This will:");
    println!(
        "  1. {} of your project",
        if no_backup {
            style("Skip backup creation").yellow()
        } else {
            style("Create a backup").green()
        }
    );
    println!(
        "  2. Run {} to fix edition issues",
        style("cargo fix --edition").cyan()
    );
    println!(
        "  3. Update {} to edition {}",
        style("Cargo.toml").cyan(),
        target_edition.as_str()
    );
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
        crate::edition::migrator::MigrationStatus::Success
        | crate::edition::migrator::MigrationStatus::Completed => {
            println!(
                "\n{}",
                style("✅ Migration completed successfully!").green().bold()
            );
        }
        crate::edition::migrator::MigrationStatus::PartialSuccess
        | crate::edition::migrator::MigrationStatus::Partial => {
            println!(
                "\n{}",
                style("⚠️  Migration completed with warnings")
                    .yellow()
                    .bold()
            );
        }
        crate::edition::migrator::MigrationStatus::AlreadyUpToDate => {
            println!("\n{}", style("✅ Already up to date!").green().bold());
            return Ok(());
        }
        crate::edition::migrator::MigrationStatus::Failed => {
            println!("\n{}", style("❌ Migration failed").red().bold());
        }
        crate::edition::migrator::MigrationStatus::Pending => {
            println!("\n{}", style("⏳ Migration is pending").yellow().bold());
        }
        crate::edition::migrator::MigrationStatus::NotStarted => {
            println!("\n{}", style("❓ Migration not started").dim().bold());
        }
        crate::edition::migrator::MigrationStatus::InProgress => {
            println!("\n{}", style("🔄 Migration in progress").blue().bold());
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
        println!(
            "\n💾 Backup saved to: {}",
            style(backup_path.display()).dim()
        );
    }

    println!("\n📋 Next steps:");
    println!("  1. Review the changes made by the migration");
    println!(
        "  2. Run {} to ensure everything compiles",
        style("cargo build").cyan()
    );
    println!(
        "  3. Run {} to verify functionality",
        style("cargo test").cyan()
    );
    println!("  4. Commit the changes to version control");

    Ok(())
}

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

    // Validate current state and display initial info
    let current_edition = validate_and_display_intro(&target_edition, &project_path).await?;
    if current_edition >= target_edition {
        println!(
            "\n{}",
            style("✅ Already on target edition or newer!")
                .green()
                .bold()
        );
        return Ok(());
    }

    // Display migration plan
    display_migration_plan(&target_edition, no_backup, test);

    // Execute migration
    let options = create_migration_options(no_backup, test, idioms);
    let result = execute_migration(&project_path, target_edition, options).await?;

    // Process and display results
    process_migration_result(&result)
}

/// Validate current state and display introduction
async fn validate_and_display_intro(
    target_edition: &Edition,
    project_path: &std::path::Path,
) -> Result<Edition> {
    println!("🚀 Edition Migration Assistant\n");
    println!("  Target:   {}", style(target_edition.to_string()).cyan());
    println!("  Project:  {}", style(project_path.display()).dim());

    let current_edition = crate::edition::detect_edition(&project_path.join("Cargo.toml")).await?;
    println!(
        "  Current:  {}",
        style(current_edition.to_string()).yellow()
    );
    println!();

    Ok(current_edition)
}

/// Display the migration plan to the user
fn display_migration_plan(
    target_edition: &Edition,
    no_backup: bool,
    test: bool,
) {
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
}

/// Create migration options from command line flags
fn create_migration_options(
    no_backup: bool,
    test: bool,
    idioms: bool,
) -> MigrationOptions {
    MigrationOptions {
        create_backup: !no_backup,
        run_tests: test,
        fix_idioms: idioms,
        ..Default::default()
    }
}

/// Execute the migration with progress indication
async fn execute_migration(
    project_path: &std::path::Path,
    target_edition: Edition,
    options: MigrationOptions,
) -> Result<crate::edition::migrator::MigrationResult> {
    let migrator = EditionMigrator::new(project_path);

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

    Ok(result)
}

/// Process and display migration results
fn process_migration_result(
    result: &crate::edition::migrator::MigrationResult,
) -> Result<()> {
    display_migration_status(&result.status)?;
    display_result_messages(result);
    display_backup_info(result);
    display_next_steps();
    Ok(())
}

/// Display the migration status
fn display_migration_status(
    status: &crate::edition::migrator::MigrationStatus,
) -> Result<()> {
    use crate::edition::migrator::MigrationStatus;
    
    match status {
        MigrationStatus::Success | MigrationStatus::Completed => {
            println!(
                "\n{}",
                style("✅ Migration completed successfully!").green().bold()
            );
        }
        MigrationStatus::PartialSuccess | MigrationStatus::Partial => {
            println!(
                "\n{}",
                style("⚠️  Migration completed with warnings")
                    .yellow()
                    .bold()
            );
        }
        MigrationStatus::AlreadyUpToDate => {
            println!("\n{}", style("✅ Already up to date!").green().bold());
            return Ok(());
        }
        MigrationStatus::Failed => {
            println!("\n{}", style("❌ Migration failed").red().bold());
        }
        MigrationStatus::Pending => {
            println!("\n{}", style("⏳ Migration is pending").yellow().bold());
        }
        MigrationStatus::NotStarted => {
            println!("\n{}", style("❓ Migration not started").dim().bold());
        }
        MigrationStatus::InProgress => {
            println!("\n{}", style("🔄 Migration in progress").blue().bold());
        }
    }
    Ok(())
}

/// Display messages, warnings, and errors from migration result
fn display_result_messages(result: &crate::edition::migrator::MigrationResult) {
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
}

/// Display backup information if backup was created
fn display_backup_info(result: &crate::edition::migrator::MigrationResult) {
    if let Some(backup_path) = &result.backup_path {
        println!(
            "\n💾 Backup saved to: {}",
            style(backup_path.display()).dim()
        );
    }
}

/// Display next steps for the user
fn display_next_steps() {
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
}

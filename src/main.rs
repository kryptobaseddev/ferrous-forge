//! Ferrous Forge - The Type-Safe Rust Development Standards Enforcer
//!
//! This is the main binary entry point for the Ferrous Forge CLI tool.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use clap::Parser;
use ferrous_forge::{cli::Cli, commands, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    match cli.command {
        commands::Commands::Init { force } => commands::init::execute(force).await,
        commands::Commands::Status => commands::status::execute().await,
        commands::Commands::Update {
            channel,
            rules_only,
            dry_run,
        } => commands::update::execute(channel, rules_only, dry_run).await,
        commands::Commands::Config {
            set,
            get,
            list,
            reset,
        } => commands::config::execute(set, get, list, reset).await,
        commands::Commands::Validate {
            path,
            ai_report,
            compare_previous: _,
        } => commands::validate::execute(path, ai_report).await,
        commands::Commands::Rollback { version } => commands::rollback::execute(version).await,
        commands::Commands::Uninstall { confirm } => commands::uninstall::execute(confirm).await,
        commands::Commands::Rust { command } => match command {
            commands::RustCommand::Check { verbose } => commands::rust::handle_check(verbose).await,
            commands::RustCommand::Recommend { stable_only } => {
                commands::rust::handle_recommend(stable_only).await
            }
            commands::RustCommand::List { count } => commands::rust::handle_list(count).await,
        },
        commands::Commands::Edition { command } => match command {
            commands::EditionCommand::Check { path } => {
                commands::edition::handle_check(&path).await
            }
            commands::EditionCommand::Migrate {
                edition,
                no_backup,
                test,
                idioms,
            } => commands::edition::handle_migrate(&edition, no_backup, test, idioms).await,
            commands::EditionCommand::Analyze { path, edition } => {
                commands::edition::handle_analyze(&path, &edition).await
            }
        },
        commands::Commands::Template { command } => command.execute().await,
        commands::Commands::Safety { command } => match command {
            commands::SafetyCommand::Status => commands::safety::handle_status().await,
            commands::SafetyCommand::Install { force, path } => {
                commands::safety::handle_install(force, &path).await
            }
            commands::SafetyCommand::Check {
                stage,
                path,
                verbose,
            } => commands::safety::handle_check(&stage, &path, verbose).await,
            commands::SafetyCommand::Test { path } => {
                commands::safety::test_individual_checks(&path).await
            }
        },
        commands::Commands::Fix {
            path,
            only,
            skip,
            dry_run,
            limit,
            ai_analysis,
        } => commands::fix::execute_with_ai(path, only, skip, dry_run, limit, ai_analysis).await,
    }
}

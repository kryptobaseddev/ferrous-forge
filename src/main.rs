//! Ferrous Forge - The Type-Safe Rust Development Standards Enforcer
//!
//! This is the main binary entry point for the Ferrous Forge CLI tool.
//!
//! @task T016
//! @epic T014

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use clap::Parser;
use ferrous_forge::{Result, cli::Cli, commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    execute_command(cli.command).await
}

/// Execute the CLI command
async fn execute_command(command: commands::Commands) -> Result<()> {
    match command {
        commands::Commands::Init { force, project } => {
            if project {
                commands::init::execute_project().await
            } else {
                commands::init::execute(force).await
            }
        }
        commands::Commands::Status => commands::status::execute().await,
        commands::Commands::Update {
            channel,
            rules_only,
            dry_run,
        } => commands::update::execute(channel, rules_only, dry_run).await,
        commands::Commands::Config { command } => {
            if let Some(cmd) = command {
                commands::config::execute_with_subcommand(cmd).await
            } else {
                // Default: show help
                commands::config::show_help();
                Ok(())
            }
        }
        commands::Commands::Validate {
            path,
            ai_report,
            compare_previous: _,
            locked_only,
        } => commands::validate::execute(path, ai_report, locked_only).await,
        commands::Commands::Rollback { version } => commands::rollback::execute(version).await,
        commands::Commands::Uninstall { confirm } => commands::uninstall::execute(confirm).await,
        commands::Commands::Rust { command } => execute_rust_command(command).await,
        commands::Commands::Edition { command } => execute_edition_command(command).await,
        commands::Commands::Template { command } => command.execute().await,
        commands::Commands::Safety { command } => execute_safety_command(command).await,
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

/// Execute rust subcommands
async fn execute_rust_command(command: commands::RustCommand) -> Result<()> {
    match command {
        commands::RustCommand::Check { verbose, enforce } => {
            commands::rust::handle_check(verbose, enforce).await
        }
        commands::RustCommand::Recommend { stable_only } => {
            commands::rust::handle_recommend(stable_only).await
        }
        commands::RustCommand::List { count, toolchains } => {
            commands::rust::handle_list(count, toolchains).await
        }
        commands::RustCommand::Releases { count } => commands::rust::handle_releases(count).await,
        commands::RustCommand::CheckUpdates { verbose } => {
            commands::rust::handle_check_updates(verbose).await
        }
        commands::RustCommand::ReleaseNotes { version, detailed } => {
            commands::rust::handle_release_notes(version, detailed).await
        }
        commands::RustCommand::Security { fail_on_issues } => {
            commands::rust::handle_security(fail_on_issues).await
        }
        commands::RustCommand::Update {
            dry_run,
            yes,
            self_update,
        } => commands::rust::handle_update(dry_run, yes, self_update).await,
        commands::RustCommand::InstallToolchain { channel, default } => {
            commands::rust::handle_install_toolchain(channel, default).await
        }
        commands::RustCommand::UninstallToolchain { channel } => {
            commands::rust::handle_uninstall_toolchain(channel).await
        }
        commands::RustCommand::Switch { channel } => commands::rust::handle_switch(channel).await,
    }
}

/// Execute edition subcommands
async fn execute_edition_command(command: commands::EditionCommand) -> Result<()> {
    match command {
        commands::EditionCommand::Check { path } => commands::edition::handle_check(&path).await,
        commands::EditionCommand::Migrate {
            edition,
            no_backup,
            test,
            idioms,
        } => commands::edition::handle_migrate(&edition, no_backup, test, idioms).await,
        commands::EditionCommand::Analyze { path, edition } => {
            commands::edition::handle_analyze(&path, &edition).await
        }
    }
}

/// Execute safety subcommands
///
/// @task T019
/// @epic T014
async fn execute_safety_command(command: commands::SafetyCommand) -> Result<()> {
    match command {
        commands::SafetyCommand::Status => commands::safety::handle_status().await,
        commands::SafetyCommand::Install { force, path, cargo } => {
            commands::safety::handle_install(force, &path, cargo).await
        }
        commands::SafetyCommand::Check {
            stage,
            path,
            verbose,
        } => commands::safety::handle_check(&stage, &path, verbose).await,
        commands::SafetyCommand::Test { path } => {
            commands::safety::test_individual_checks(&path).await
        }
        commands::SafetyCommand::Bypass {
            stage,
            reason,
            duration,
            user,
        } => commands::safety::bypass_cmd::handle_bypass(stage, reason, duration, user).await,
        commands::SafetyCommand::Audit { limit } => {
            commands::safety::bypass_cmd::handle_audit(limit).await
        }
        commands::SafetyCommand::CheckBypass { stage } => {
            commands::safety::handle_check_bypass(stage).await
        }
        commands::SafetyCommand::Uninstall { path, confirm } => {
            commands::safety::handle_uninstall(&path, confirm).await
        }
        commands::SafetyCommand::Config { show, set, get } => {
            if show {
                commands::safety::config_cmd::handle_config_show().await
            } else if let Some(key_value) = set {
                // Parse key=value format
                let parts: Vec<&str> = key_value.splitn(2, '=').collect();
                if parts.len() == 2 {
                    commands::safety::config_cmd::handle_config_set(
                        parts[0].to_string(),
                        parts[1].to_string(),
                    )
                    .await
                } else {
                    Err(ferrous_forge::Error::config(
                        "Invalid format. Use: --set key=value".to_string(),
                    ))
                }
            } else if let Some(key) = get {
                commands::safety::config_cmd::handle_config_get(key).await
            } else {
                // Default to showing config if no flag specified
                commands::safety::config_cmd::handle_config_show().await
            }
        }
        commands::SafetyCommand::Report { last, audit, stage } => {
            commands::safety::report_cmd::handle_report(last, audit, stage).await
        }
        commands::SafetyCommand::Stats { days } => {
            commands::safety::stats_cmd::handle_stats(days).await
        }
    }
}

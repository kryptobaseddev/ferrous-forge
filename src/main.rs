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
            dry_run 
        } => commands::update::execute(channel, rules_only, dry_run).await,
        commands::Commands::Config { 
            set, 
            get, 
            list, 
            reset 
        } => commands::config::execute(set, get, list, reset).await,
        commands::Commands::Validate { path } => commands::validate::execute(path).await,
        commands::Commands::Rollback { version } => commands::rollback::execute(version).await,
        commands::Commands::Uninstall { confirm } => commands::uninstall::execute(confirm).await,
    }
}
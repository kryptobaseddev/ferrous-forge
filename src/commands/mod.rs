//! Command implementations for Ferrous Forge

use clap::Subcommand;

/// Available commands for Ferrous Forge
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Ferrous Forge system-wide
    Init {
        /// Force initialization even if already configured
        #[arg(short, long)]
        force: bool,
    },
    /// Show status of Ferrous Forge installation and configuration
    Status,
    /// Update Ferrous Forge to the latest version
    Update {
        /// Update channel to use (stable, beta, nightly)
        #[arg(short, long, default_value = "stable")]
        channel: String,
        /// Only update rules, not the binary
        #[arg(short, long)]
        rules_only: bool,
        /// Show what would be updated without actually updating
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Manage configuration settings
    Config {
        /// Set a configuration value (key=value)
        #[arg(short, long)]
        set: Option<String>,
        /// Get a configuration value
        #[arg(short, long)]
        get: Option<String>,
        /// List all configuration values
        #[arg(short, long)]
        list: bool,
        /// Reset configuration to defaults
        #[arg(short, long)]
        reset: bool,
    },
    /// Validate a Rust project against standards
    Validate {
        /// Path to the project to validate (defaults to current directory)
        path: Option<std::path::PathBuf>,
    },
    /// Rollback to a previous version
    Rollback {
        /// Version to rollback to
        version: String,
    },
    /// Uninstall Ferrous Forge from the system
    Uninstall {
        /// Confirm uninstallation without prompting
        #[arg(short, long)]
        confirm: bool,
    },
}

pub mod init;
pub mod status;
pub mod update;
pub mod config;
pub mod validate;
pub mod rollback;
pub mod uninstall;
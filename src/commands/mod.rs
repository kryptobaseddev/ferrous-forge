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
    /// Rust version management
    Rust {
        #[command(subcommand)]
        command: RustCommand,
    },
    /// Edition management
    Edition {
        #[command(subcommand)]
        command: EditionCommand,
    },
}

/// Rust version management subcommands
#[derive(Subcommand)]
pub enum RustCommand {
    /// Check current Rust version and available updates
    Check {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Get update recommendations
    Recommend {
        /// Only consider stable releases
        #[arg(short, long)]
        stable_only: bool,
    },
    /// List recent Rust releases
    List {
        /// Number of releases to show
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
}

/// Edition management subcommands
#[derive(Subcommand)]
pub enum EditionCommand {
    /// Check edition compliance
    Check {
        /// Project path
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
    },
    /// Migrate to a new edition
    Migrate {
        /// Target edition (2018, 2021, 2024)
        #[arg(default_value = "2024")]
        edition: String,
        /// Skip backup creation
        #[arg(long)]
        no_backup: bool,
        /// Run tests after migration
        #[arg(long)]
        test: bool,
        /// Apply edition idioms
        #[arg(long)]
        idioms: bool,
    },
    /// Analyze edition compatibility
    Analyze {
        /// Project path
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
        /// Target edition
        #[arg(default_value = "2024")]
        edition: String,
    },
}

pub mod config;
pub mod edition;
pub mod init;
pub mod rollback;
pub mod rust;
pub mod status;
pub mod uninstall;
pub mod update;
pub mod validate;

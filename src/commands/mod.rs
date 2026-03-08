//! Command implementations for Ferrous Forge

use clap::Subcommand;

/// Available commands for Ferrous Forge
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Ferrous Forge system-wide, or set up a project with --project
    Init {
        /// Force initialization even if already configured
        #[arg(short, long)]
        force: bool,
        /// Set up project-level tooling (rustfmt.toml, clippy.toml, Cargo.toml lints,
        /// .vscode/settings.json, CI workflow, and git hooks) in the current directory.
        /// Requires an existing Rust project with Cargo.toml.
        #[arg(short, long)]
        project: bool,
    },
    /// Show status of Ferrous Forge installation and configuration
    Status,
    /// Update Ferrous Forge to the latest version
    Update {
        /// Update channel to use (stable, beta, nightly)
        #[arg(long, default_value = "stable")]
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
        /// Show configuration sources from hierarchy
        #[arg(long)]
        sources: bool,
        /// Migrate old configuration to hierarchical system
        #[arg(long)]
        migrate: bool,
        /// Configuration level to use (system, user, project)
        #[arg(long)]
        level: Option<String>,
    },
    /// Validate a Rust project against standards
    Validate {
        /// Path to the project to validate (defaults to current directory)
        path: Option<std::path::PathBuf>,
        /// Generate AI-friendly compliance report
        #[arg(long)]
        ai_report: bool,
        /// Compare with previous report
        #[arg(long)]
        compare_previous: bool,
        /// Only check locked settings (edition, rust-version) — exits 1 if any locked violation
        #[arg(long)]
        locked_only: bool,
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
        /// Rust version management subcommand
        #[command(subcommand)]
        command: RustCommand,
    },
    /// Edition management
    Edition {
        /// Edition management subcommand
        #[command(subcommand)]
        command: EditionCommand,
    },
    /// Safety pipeline management
    Safety {
        /// Safety pipeline subcommand
        #[command(subcommand)]
        command: SafetyCommand,
    },
    /// Project template management
    Template {
        /// Template subcommand
        #[command(subcommand)]
        command: template::TemplateCommand,
    },
    /// Automatically fix code violations
    Fix {
        /// Path to the project to fix (defaults to current directory)
        path: Option<std::path::PathBuf>,
        /// Only fix specific violation types (comma-separated)
        #[arg(long)]
        only: Option<String>,
        /// Skip specific violation types (comma-separated)
        #[arg(long)]
        skip: Option<String>,
        /// Show what would be fixed without making changes
        #[arg(long)]
        dry_run: bool,
        /// Fix at most this many violations (for testing)
        #[arg(long)]
        limit: Option<usize>,
        /// Enable AI-powered analysis for complex violations
        #[arg(long)]
        ai_analysis: bool,
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

/// Safety pipeline management subcommands
#[derive(Subcommand)]
pub enum SafetyCommand {
    /// Check safety pipeline status
    Status,
    /// Install git hooks for safety pipeline
    Install {
        /// Force reinstall even if hooks already exist
        #[arg(short, long)]
        force: bool,
        /// Project path
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
        /// Install cargo publish interception
        #[arg(long)]
        cargo: bool,
    },
    /// Run safety checks manually
    Check {
        /// Pipeline stage to check
        #[arg(long, default_value = "pre-commit")]
        stage: String,
        /// Project path
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Test individual safety checks
    Test {
        /// Project path
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
    },
}

/// Configuration management command handlers.
pub mod config;
/// Edition management command handlers.
pub mod edition;
/// Automatic code violation fix command handlers.
pub mod fix;
/// Project and system initialization command handlers.
pub mod init;
/// Version rollback command handlers.
pub mod rollback;
/// Rust version management command handlers.
pub mod rust;
/// Safety pipeline command handlers.
pub mod safety;
/// Installation status display command handlers.
pub mod status;
/// Project template command handlers.
pub mod template;
/// Uninstall command handlers.
pub mod uninstall;
/// Self-update command handlers.
pub mod update;
/// Project validation command handlers.
pub mod validate;

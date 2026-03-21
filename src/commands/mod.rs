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
        /// Config subcommand
        #[command(subcommand)]
        command: Option<ConfigCommand>,
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
        /// Exit with error if version doesn't meet locked requirements
        #[arg(long)]
        enforce: bool,
    },
    /// Get update recommendations
    Recommend {
        /// Only consider stable releases
        #[arg(short, long)]
        stable_only: bool,
    },
    /// List recent Rust releases or installed toolchains
    List {
        /// Number of releases to show
        #[arg(short, long, default_value = "10")]
        count: usize,
        /// List installed toolchains instead of releases
        #[arg(long)]
        toolchains: bool,
    },
    /// List recent Rust releases (alias for 'list')
    Releases {
        /// Number of releases to show
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    /// Check if Rust updates are available
    CheckUpdates {
        /// Show detailed information about available updates
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show release notes for a specific version
    ReleaseNotes {
        /// Version to show notes for (e.g., "1.70.0" or "v1.70.0")
        version: String,
        /// Show full parsed details including security/breaking changes
        #[arg(short, long)]
        detailed: bool,
    },
    /// Check for security advisories affecting current Rust version
    Security {
        /// Exit with error if security issues found
        #[arg(long)]
        fail_on_issues: bool,
    },
    /// Update Rust via rustup
    Update {
        /// Show what would be updated without making changes
        #[arg(long)]
        dry_run: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
        /// Also update rustup itself
        #[arg(long)]
        self_update: bool,
    },
    /// Install a specific toolchain
    InstallToolchain {
        /// Toolchain channel to install (stable, beta, nightly, or version like 1.70.0)
        channel: String,
        /// Set as default toolchain after installation
        #[arg(short, long)]
        default: bool,
    },
    /// Uninstall a toolchain
    UninstallToolchain {
        /// Toolchain channel to uninstall
        channel: String,
    },
    /// Switch to a different toolchain
    Switch {
        /// Toolchain channel to switch to
        channel: String,
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
    /// Create emergency bypass for safety checks
    ///
    /// Use this to temporarily skip safety checks with audit logging.
    /// Requires explicit justification. Bypasses expire after 24 hours.
    Bypass {
        /// Pipeline stage to bypass (pre-commit, pre-push, publish)
        #[arg(long, value_enum)]
        stage: SafetyBypassStage,
        /// Reason for bypass (required)
        #[arg(long)]
        reason: String,
        /// Bypass duration in hours (default: 24)
        #[arg(long, default_value = "24")]
        duration: u64,
        /// User creating the bypass (defaults to current user)
        #[arg(long)]
        user: Option<String>,
    },
    /// View bypass audit log
    Audit {
        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Check if a bypass is active (used by git hooks)
    CheckBypass {
        /// Pipeline stage to check
        #[arg(long, value_enum)]
        stage: SafetyBypassStage,
    },
    /// Uninstall git hooks
    Uninstall {
        /// Project path
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
        /// Confirm uninstall without prompting
        #[arg(short, long)]
        confirm: bool,
    },
    /// Manage safety configuration
    Config {
        /// Show current configuration
        #[arg(long, group = "config_action")]
        show: bool,
        /// Set a configuration value (key=value)
        #[arg(long, group = "config_action")]
        set: Option<String>,
        /// Get a specific configuration value
        #[arg(long, group = "config_action")]
        get: Option<String>,
    },
    /// View safety reports
    Report {
        /// Number of reports to show
        #[arg(short, long, default_value = "10")]
        last: usize,
        /// Include bypass audit log
        #[arg(long)]
        audit: bool,
        /// Filter by stage (pre-commit, pre-push, publish)
        #[arg(long)]
        stage: Option<String>,
    },
    /// Display safety statistics and metrics
    Stats {
        /// Number of days to include in statistics
        #[arg(short, long, default_value = "30")]
        days: u32,
    },
}

/// Safety bypass stage options
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SafetyBypassStage {
    /// Pre-commit checks
    PreCommit,
    /// Pre-push checks
    PrePush,
    /// Publish checks
    Publish,
}

impl SafetyBypassStage {
    /// Convert to `PipelineStage`
    pub fn to_pipeline_stage(&self) -> crate::safety::PipelineStage {
        match self {
            Self::PreCommit => crate::safety::PipelineStage::PreCommit,
            Self::PrePush => crate::safety::PipelineStage::PrePush,
            Self::Publish => crate::safety::PipelineStage::Publish,
        }
    }
}

/// Configuration subcommands
#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Get a configuration value
    Get {
        /// The configuration key to retrieve
        key: String,
    },
    /// Set a configuration value (key=value)
    Set {
        /// The key=value pair to set
        value: String,
    },
    /// List all configuration values
    List,
    /// Reset configuration to defaults
    Reset,
    /// Show configuration sources from hierarchy
    Sources,
    /// Migrate old configuration to hierarchical system
    Migrate,
    /// Lock a configuration value to prevent changes
    Lock {
        /// The configuration key to lock
        key: String,
        /// The value to lock (defaults to current value)
        #[arg(long)]
        value: Option<String>,
        /// Reason for locking (required)
        #[arg(long)]
        reason: String,
        /// Configuration level to lock at (project, user, system)
        #[arg(long, default_value = "project")]
        level: ConfigLevelArg,
    },
    /// Unlock a configuration value to allow changes
    Unlock {
        /// The configuration key to unlock
        key: String,
        /// Reason for unlocking (required)
        #[arg(long)]
        reason: String,
        /// Configuration level to unlock at
        #[arg(long, default_value = "project")]
        level: ConfigLevelArg,
    },
    /// Show lock status for all configuration values
    LockStatus,
    /// View lock audit log
    LockAudit {
        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Export configuration for sharing
    Export {
        /// Configuration level to export (project, user, system)
        #[arg(long, default_value = "user")]
        level: ConfigLevelArg,
        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
        /// Description of this shared config
        #[arg(long)]
        description: Option<String>,
    },
    /// Import configuration from shared file
    Import {
        /// Path to the shared config file
        file: std::path::PathBuf,
        /// Target level to import to (project, user, system)
        #[arg(long, default_value = "project")]
        level: ConfigLevelArg,
        /// Skip importing locks
        #[arg(long)]
        no_locks: bool,
        /// Force overwrite of existing values
        #[arg(long)]
        force: bool,
    },
}

/// Configuration level argument for CLI
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ConfigLevelArg {
    /// System-wide configuration
    System,
    /// User-specific configuration
    User,
    /// Project-specific configuration
    Project,
}

impl ConfigLevelArg {
    /// Convert to `ConfigLevel`
    pub fn to_config_level(&self) -> crate::config::ConfigLevel {
        match self {
            Self::System => crate::config::ConfigLevel::System,
            Self::User => crate::config::ConfigLevel::User,
            Self::Project => crate::config::ConfigLevel::Project,
        }
    }
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

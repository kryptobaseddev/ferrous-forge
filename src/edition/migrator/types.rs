//! Types for edition migration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Options for migration process
#[derive(Debug, Clone, Default)]
pub struct MigrationOptions {
    /// Create backup before migration
    pub create_backup: bool,
    /// Update dependencies to latest compatible versions
    pub update_dependencies: bool,
    /// Run tests after migration
    pub run_tests: bool,
    /// Apply rustfmt after migration
    pub apply_rustfmt: bool,
    /// Auto-commit changes
    pub auto_commit: bool,
    /// Fix edition-specific idioms
    pub fix_idioms: bool,
    /// Custom migration rules
    pub custom_rules: Vec<MigrationRule>,
}

/// Custom migration rule
#[derive(Debug, Clone)]
pub struct MigrationRule {
    /// Pattern to match
    pub pattern: String,
    /// Replacement
    pub replacement: String,
    /// Apply to file extensions
    pub file_extensions: Vec<String>,
}

/// Result of migration process
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Migration status
    pub status: MigrationStatus,
    /// Files changed
    pub files_changed: Vec<PathBuf>,
    /// Warnings encountered
    pub warnings: Vec<String>,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Messages (legacy field)
    pub messages: Vec<String>,
    /// Backup location if created
    pub backup_location: Option<PathBuf>,
    /// Backup path (legacy name)
    pub backup_path: Option<PathBuf>,
    /// Test results if run
    pub test_results: Option<TestResults>,
    /// Migration steps performed
    pub steps_performed: Vec<MigrationStep>,
    /// Dependencies updated
    pub dependencies_updated: HashMap<String, (String, String)>,
}

/// Migration status
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum MigrationStatus {
    #[default]
    /// Not started
    NotStarted,
    /// Already up to date
    AlreadyUpToDate,
    /// In progress
    InProgress,
    /// Completed successfully
    Completed,
    /// Completed successfully (legacy name)
    Success,
    /// Failed with errors
    Failed,
    /// Partially completed
    Partial,
    /// Partially successful (legacy name)
    PartialSuccess,
    /// Pending
    Pending,
}

/// Test results
#[derive(Debug, Serialize, Deserialize)]
pub struct TestResults {
    /// Total tests
    pub total: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// Ignored tests
    pub ignored: usize,
}

/// Migration step
#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStep {
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Success status
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
}

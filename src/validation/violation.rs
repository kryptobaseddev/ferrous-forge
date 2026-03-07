//! Violation types and reporting

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Types of violations that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationType {
    /// Underscore parameter or let assignment bandaid
    UnderscoreBandaid,
    /// Wrong Rust edition (locked by project config)
    WrongEdition,
    /// File exceeds size limit
    FileTooLarge,
    /// Function exceeds size limit
    FunctionTooLarge,
    /// Line exceeds length limit
    LineTooLong,
    /// Use of .unwrap() or .expect() in production code
    UnwrapInProduction,
    /// Missing documentation
    MissingDocs,
    /// Missing required dependencies
    MissingDependencies,
    /// Rust version too old (locked by project config)
    OldRustVersion,
    /// A project configuration value is locked and was violated
    LockedSetting,
    /// Module-level //! documentation is missing
    MissingModuleDoc,
    /// Cargo.toml is missing [lints.rustdoc] configuration
    MissingDocConfig,
}

/// Severity level of a violation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Violation that prevents code from compiling
    Error,
    /// Violation that should be fixed but doesn't break compilation
    Warning,
}

/// A single standards violation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Violation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// File where violation occurred
    pub file: PathBuf,
    /// Line number (1-based for display)
    pub line: usize,
    /// Human-readable message
    pub message: String,
    /// Severity of the violation
    pub severity: Severity,
}

impl Violation {
    /// Create a new violation
    pub fn new(
        violation_type: ViolationType,
        file: PathBuf,
        line: usize,
        message: String,
        severity: Severity,
    ) -> Self {
        Self {
            violation_type,
            file,
            line,
            message,
            severity,
        }
    }

    /// Returns true if this violation represents a locked setting (edition/version/config lock)
    pub fn is_locked_setting(&self) -> bool {
        matches!(
            self.violation_type,
            ViolationType::WrongEdition
                | ViolationType::OldRustVersion
                | ViolationType::LockedSetting
        )
    }
}

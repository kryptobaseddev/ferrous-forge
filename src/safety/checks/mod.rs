//! Safety check implementations
//!
//! This module contains individual check implementations for the safety pipeline.
//! Each check module implements a specific validation (format, clippy, tests, etc.)

use crate::Result;
use std::path::Path;

use super::report::CheckResult;

pub mod audit;
pub mod build;
pub mod clippy;
pub mod doc;
pub mod format;
pub mod license;
pub mod publish;
pub mod semver;
pub mod standards;
pub mod test;
pub mod test_runner;

/// Trait for implementing safety checks
pub trait SafetyCheck {
    /// Run the safety check
    async fn run(project_path: &Path) -> Result<CheckResult>;

    /// Get the name of this check
    fn name() -> &'static str;

    /// Get a description of what this check does
    fn description() -> &'static str;
}

/// Registry of all available safety checks
pub struct CheckRegistry;

impl CheckRegistry {
    /// Get all available check types
    pub fn all_checks() -> Vec<super::CheckType> {
        vec![
            super::CheckType::Format,
            super::CheckType::Clippy,
            super::CheckType::Build,
            super::CheckType::Test,
            super::CheckType::Audit,
            super::CheckType::Doc,
            super::CheckType::PublishDryRun,
            super::CheckType::Standards,
            super::CheckType::DocCoverage,
            super::CheckType::License,
            super::CheckType::Semver,
        ]
    }

    /// Get description for a check type
    pub fn get_description(check_type: super::CheckType) -> &'static str {
        match check_type {
            super::CheckType::Format => "Validates code formatting with rustfmt",
            super::CheckType::Clippy => "Runs clippy lints with strict warnings",
            super::CheckType::Build => "Ensures project builds successfully",
            super::CheckType::Test => "Runs the complete test suite",
            super::CheckType::Audit => "Scans for security vulnerabilities",
            super::CheckType::Doc => "Builds project documentation",
            super::CheckType::PublishDryRun => "Validates crates.io publication",
            super::CheckType::Standards => "Validates Ferrous Forge standards",
            super::CheckType::DocCoverage => "Checks documentation coverage",
            super::CheckType::License => "Validates license compatibility",
            super::CheckType::Semver => "Checks semantic versioning compliance",
        }
    }
}

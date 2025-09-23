//! Enhanced Safety Pipeline for Ferrous Forge
//!
//! This module implements a comprehensive safety pipeline that prevents broken code
//! from reaching GitHub or crates.io by running mandatory checks before git operations
//! and cargo publish commands.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod bypass;
pub mod checks;
pub mod config;
// pub mod installer;  // TODO: Implement installer
pub mod pipeline;
pub mod report;

pub use config::SafetyConfig;
pub use pipeline::SafetyPipeline;
pub use report::{CheckResult, SafetyReport};

/// Pipeline stage for safety checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStage {
    /// Pre-commit checks (fast, essential)
    PreCommit,
    /// Pre-push checks (comprehensive)
    PrePush,
    /// Publish checks (exhaustive)
    Publish,
}

impl PipelineStage {
    /// Get the stage name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Self::PreCommit => "pre-commit",
            Self::PrePush => "pre-push",
            Self::Publish => "publish",
        }
    }

    /// Get the display name for the stage
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PreCommit => "Pre-Commit",
            Self::PrePush => "Pre-Push",
            Self::Publish => "Publish",
        }
    }

    /// Get the timeout for this stage
    pub fn default_timeout(&self) -> Duration {
        match self {
            Self::PreCommit => Duration::from_secs(300), // 5 minutes
            Self::PrePush => Duration::from_secs(600),   // 10 minutes
            Self::Publish => Duration::from_secs(900),   // 15 minutes
        }
    }
}

impl std::str::FromStr for PipelineStage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "pre-commit" | "precommit" | "commit" => Ok(Self::PreCommit),
            "pre-push" | "prepush" | "push" => Ok(Self::PrePush),
            "publish" | "pub" => Ok(Self::Publish),
            _ => Err(Error::parse(format!("Unknown pipeline stage: {}", s))),
        }
    }
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Safety check type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckType {
    /// Format checking (cargo fmt --check)
    Format,
    /// Clippy linting (cargo clippy -- -D warnings)
    Clippy,
    /// Build checking (cargo build --release)
    Build,
    /// Test execution (cargo test --all-features)
    Test,
    /// Security audit (cargo audit)
    Audit,
    /// Documentation build (cargo doc)
    Doc,
    /// Publish dry run (cargo publish --dry-run)
    PublishDryRun,
    /// Ferrous Forge standards validation
    Standards,
    /// Documentation coverage check
    DocCoverage,
    /// License validation
    License,
    /// Semver compatibility check
    Semver,
}

impl CheckType {
    /// Get the check name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Self::Format => "format",
            Self::Clippy => "clippy",
            Self::Build => "build",
            Self::Test => "test",
            Self::Audit => "audit",
            Self::Doc => "doc",
            Self::PublishDryRun => "publish-dry-run",
            Self::Standards => "standards",
            Self::DocCoverage => "doc-coverage",
            Self::License => "license",
            Self::Semver => "semver",
        }
    }

    /// Get the display name for the check
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Format => "Format Check",
            Self::Clippy => "Clippy Check",
            Self::Build => "Build Check",
            Self::Test => "Test Check",
            Self::Audit => "Security Audit",
            Self::Doc => "Documentation Build",
            Self::PublishDryRun => "Publish Dry Run",
            Self::Standards => "Standards Check",
            Self::DocCoverage => "Documentation Coverage",
            Self::License => "License Check",
            Self::Semver => "Semver Check",
        }
    }

    /// Get the checks for a specific pipeline stage
    pub fn for_stage(stage: PipelineStage) -> Vec<Self> {
        match stage {
            PipelineStage::PreCommit => {
                vec![Self::Format, Self::Clippy, Self::Build, Self::Standards]
            }
            PipelineStage::PrePush => vec![
                Self::Format,
                Self::Clippy,
                Self::Build,
                Self::Standards,
                Self::Test,
                Self::Audit,
                Self::Doc,
            ],
            PipelineStage::Publish => vec![
                Self::Format,
                Self::Clippy,
                Self::Build,
                Self::Standards,
                Self::Test,
                Self::Audit,
                Self::Doc,
                Self::PublishDryRun,
                Self::DocCoverage,
                Self::License,
                Self::Semver,
            ],
        }
    }
}

/// Safety enforcement result
#[derive(Debug, Clone)]
pub enum SafetyResult {
    /// All checks passed - operation allowed
    Passed,
    /// Checks failed - operation blocked
    Blocked {
        /// Failed checks
        failures: Vec<String>,
        /// Suggestions for fixes
        suggestions: Vec<String>,
    },
    /// Checks bypassed - operation allowed with warning
    Bypassed {
        /// Reason for bypass
        reason: String,
        /// Who bypassed
        user: String,
    },
}

impl SafetyResult {
    /// Check if the operation should be allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Passed | Self::Bypassed { .. })
    }

    /// Get a user-friendly message
    pub fn message(&self) -> String {
        match self {
            Self::Passed => "🎉 All safety checks passed! Operation allowed.".to_string(),
            Self::Blocked {
                failures,
                suggestions,
            } => {
                let mut msg = "🚨 Safety checks FAILED - operation blocked!\n\n".to_string();

                if !failures.is_empty() {
                    msg.push_str("Failures:\n");
                    for failure in failures {
                        msg.push_str(&format!("  • {}\n", failure));
                    }
                }

                if !suggestions.is_empty() {
                    msg.push_str("\nSuggestions:\n");
                    for suggestion in suggestions {
                        msg.push_str(&format!("  • {}\n", suggestion));
                    }
                }

                msg
            }
            Self::Bypassed { reason, user } => {
                format!(
                    "⚠️  Safety checks bypassed by {} - reason: {}",
                    user, reason
                )
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_stage_from_str() {
        assert_eq!(
            "pre-commit".parse::<PipelineStage>().unwrap(),
            PipelineStage::PreCommit
        );
        assert_eq!(
            "pre-push".parse::<PipelineStage>().unwrap(),
            PipelineStage::PrePush
        );
        assert_eq!(
            "publish".parse::<PipelineStage>().unwrap(),
            PipelineStage::Publish
        );
        assert!("invalid".parse::<PipelineStage>().is_err());
    }

    #[test]
    fn test_check_type_for_stage() {
        let pre_commit_checks = CheckType::for_stage(PipelineStage::PreCommit);
        assert!(pre_commit_checks.contains(&CheckType::Format));
        assert!(pre_commit_checks.contains(&CheckType::Clippy));
        assert!(!pre_commit_checks.contains(&CheckType::Test)); // Not in pre-commit

        let publish_checks = CheckType::for_stage(PipelineStage::Publish);
        assert!(publish_checks.contains(&CheckType::PublishDryRun));
        assert!(publish_checks.contains(&CheckType::Semver));
    }

    #[test]
    fn test_safety_result_is_allowed() {
        assert!(SafetyResult::Passed.is_allowed());
        assert!(SafetyResult::Bypassed {
            reason: "test".to_string(),
            user: "test".to_string()
        }
        .is_allowed());
        assert!(!SafetyResult::Blocked {
            failures: vec![],
            suggestions: vec![]
        }
        .is_allowed());
    }
}

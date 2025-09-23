//! Safety pipeline execution logic
//!
//! This module contains the core execution logic for safety checks,
//! including progress display and parallel/sequential execution.

use super::{
    checks,
    config::StageConfig,
    report::{CheckResult, SafetyReport},
    CheckType, PipelineStage,
};
use crate::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Duration;

/// Progress display and execution coordinator
pub struct ExecutionManager {
    /// Whether to show progress indicators
    show_progress: bool,
    /// Whether to run checks in parallel
    parallel_checks: bool,
}

impl ExecutionManager {
    /// Create a new execution manager
    pub fn new(show_progress: bool, parallel_checks: bool) -> Self {
        Self {
            show_progress,
            parallel_checks,
        }
    }

    /// Setup progress indicators and display header
    pub fn setup_progress_display(&self, stage: PipelineStage) -> Option<MultiProgress> {
        let multi_progress = if self.show_progress {
            Some(MultiProgress::new())
        } else {
            None
        };

        println!("🛡️  Ferrous Forge Safety Pipeline - {}", stage.display_name());
        println!("{}", "=".repeat(50));

        multi_progress
    }

    /// Execute checks for the stage
    pub async fn execute_stage_checks(
        &self,
        stage_config: &StageConfig,
        report: &mut SafetyReport,
        multi_progress: Option<&MultiProgress>,
        project_path: &Path,
    ) -> Result<()> {
        let checks = &stage_config.checks;

        if self.parallel_checks && checks.len() > 1 {
            self.run_checks_parallel(checks, report, multi_progress, project_path)
                .await
        } else {
            self.run_checks_sequential(checks, report, multi_progress, project_path)
                .await
        }
    }

    /// Run checks sequentially with progress indicators
    async fn run_checks_sequential(
        &self,
        checks: &[CheckType],
        report: &mut SafetyReport,
        multi_progress: Option<&MultiProgress>,
        project_path: &Path,
    ) -> Result<()> {
        for check_type in checks {
            let pb = if let Some(mp) = multi_progress {
                let pb = mp.add(ProgressBar::new_spinner());
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.green} {msg}")
                        .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                );
                pb.set_message(format!("Running {}...", check_type.display_name()));
                pb.enable_steady_tick(Duration::from_millis(100));
                Some(pb)
            } else {
                None
            };

            let check_result = execute_check(*check_type, project_path).await?;

            if let Some(pb) = pb {
                pb.finish_with_message(format!(
                    "{} {} ({:.2}s)",
                    check_result.status_emoji(),
                    check_type.display_name(),
                    check_result.duration.as_secs_f64()
                ));
            } else {
                println!(
                    "  {} {} ({:.2}s)",
                    check_result.status_emoji(),
                    check_type.display_name(),
                    check_result.duration.as_secs_f64()
                );
            }

            report.add_check(check_result);
        }

        Ok(())
    }

    /// Run checks in parallel for better performance
    async fn run_checks_parallel(
        &self,
        checks: &[CheckType],
        report: &mut SafetyReport,
        _multi_progress: Option<&MultiProgress>,
        project_path: &Path,
    ) -> Result<()> {
        // For now, implement as sequential until we add proper parallel execution
        // Parallel execution requires careful handling of stdout/stderr
        self.run_checks_sequential(checks, report, _multi_progress, project_path)
            .await
    }
}

/// Execute a specific check type
pub async fn execute_check(check_type: CheckType, project_path: &Path) -> Result<CheckResult> {
    match check_type {
        CheckType::Format => checks::format::run(project_path).await,
        CheckType::Clippy => checks::clippy::run(project_path).await,
        CheckType::Build => checks::build::run(project_path).await,
        CheckType::Test => checks::test::run(project_path).await,
        CheckType::Audit => checks::audit::run(project_path).await,
        CheckType::Doc => checks::doc::run(project_path).await,
        CheckType::PublishDryRun => checks::publish::run(project_path).await,
        CheckType::Standards => checks::standards::run(project_path).await,
        CheckType::DocCoverage => checks::doc::coverage_check(project_path).await,
        CheckType::License => checks::license::run(project_path).await,
        CheckType::Semver => checks::semver::run(project_path).await,
    }
}

/// Handle check result and convert errors to check results
pub fn handle_check_result(
    check_result: Result<CheckResult>,
    check_type: CheckType,
) -> Result<CheckResult> {
    match check_result {
        Ok(result) => Ok(result),
        Err(e) => {
            let mut result = CheckResult::new(check_type);
            result.add_error(format!("Check failed: {}", e));
            result.add_suggestion("Check the error message above for details");
            Ok(result)
        }
    }
}

/// Get the appropriate stage for a check type
pub fn get_stage_for_check(check_type: CheckType) -> PipelineStage {
    // Find the first stage that includes this check
    for stage in [
        PipelineStage::PreCommit,
        PipelineStage::PrePush,
        PipelineStage::Publish,
    ] {
        if CheckType::for_stage(stage).contains(&check_type) {
            return stage;
        }
    }
    PipelineStage::PreCommit // Default fallback
}
//! Main safety pipeline execution engine

use crate::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::{
    bypass::BypassManager,
    checks,
    config::SafetyConfig,
    report::{CheckResult, SafetyReport},
    CheckType, PipelineStage, SafetyResult,
};

/// Main safety pipeline coordinator
pub struct SafetyPipeline {
    config: SafetyConfig,
    project_path: PathBuf,
    bypass_manager: BypassManager,
}

impl SafetyPipeline {
    /// Create a new safety pipeline
    pub async fn new(project_path: impl AsRef<Path>) -> Result<Self> {
        let config = SafetyConfig::load_or_default().await?;
        let bypass_manager = BypassManager::new(&config.bypass)?;

        Ok(Self {
            config,
            project_path: project_path.as_ref().to_path_buf(),
            bypass_manager,
        })
    }

    /// Run safety checks for a specific stage
    pub async fn run_checks(&self, stage: PipelineStage) -> Result<SafetyReport> {
        if !self.config.enabled {
            println!("⚠️  Safety pipeline is disabled");
            return Ok(SafetyReport::new(stage));
        }

        let stage_config = self.config.get_stage_config(stage);
        if !stage_config.enabled {
            println!("⚠️  {} stage is disabled", stage.display_name());
            return Ok(SafetyReport::new(stage));
        }

        let start_time = Instant::now();
        let mut report = SafetyReport::new(stage);

        // Setup progress indicators
        let multi_progress = if self.config.show_progress {
            Some(MultiProgress::new())
        } else {
            None
        };

        println!(
            "🛡️  Ferrous Forge Safety Pipeline - {}",
            stage.display_name()
        );
        println!("{}", "=".repeat(50));

        // Run checks based on stage
        let checks = &stage_config.checks;

        if self.config.parallel_checks && checks.len() > 1 {
            // Run checks in parallel for speed
            self.run_checks_parallel(checks, &mut report, multi_progress.as_ref())
                .await?;
        } else {
            // Run checks sequentially
            self.run_checks_sequential(checks, &mut report, multi_progress.as_ref())
                .await?;
        }

        report.total_duration = start_time.elapsed();

        // Save report for audit trail
        if let Err(e) = report.save_to_file().await {
            eprintln!("Warning: Failed to save safety report: {}", e);
        }

        Ok(report)
    }

    /// Enforce safety checks and block operation if they fail
    pub async fn enforce_safety(&self, stage: PipelineStage) -> Result<SafetyResult> {
        // Check for active bypass first
        if let Some(bypass) = self.bypass_manager.check_active_bypass(stage).await? {
            println!("⚠️  Safety checks bypassed: {}", bypass.reason);
            return Ok(SafetyResult::Bypassed {
                reason: bypass.reason,
                user: bypass.user,
            });
        }

        // Run the safety checks
        let report = self.run_checks(stage).await?;

        // Print detailed report
        report.print_detailed();

        if report.passed {
            Ok(SafetyResult::Passed)
        } else {
            let failures = report.all_errors();
            let suggestions = report.all_suggestions();

            Ok(SafetyResult::Blocked {
                failures,
                suggestions,
            })
        }
    }

    /// Run checks sequentially with progress indicators
    async fn run_checks_sequential(
        &self,
        checks: &[CheckType],
        report: &mut SafetyReport,
        multi_progress: Option<&MultiProgress>,
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

            let check_result = self.run_single_check(*check_type).await?;

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
    ) -> Result<()> {
        // For now, implement as sequential until we add proper parallel execution
        // Parallel execution requires careful handling of stdout/stderr
        self.run_checks_sequential(checks, report, _multi_progress)
            .await
    }

    /// Run a single safety check
    async fn run_single_check(&self, check_type: CheckType) -> Result<CheckResult> {
        let stage_config = self
            .config
            .get_stage_config(self.get_stage_for_check(check_type));
        let _check_timeout = Duration::from_secs(stage_config.timeout_seconds);

        // Run the check with timeout
        let check_result = match check_type {
            CheckType::Format => checks::format::run(&self.project_path).await,
            CheckType::Clippy => checks::clippy::run(&self.project_path).await,
            CheckType::Build => checks::build::run(&self.project_path).await,
            CheckType::Test => checks::test::run(&self.project_path).await,
            CheckType::Audit => checks::audit::run(&self.project_path).await,
            CheckType::Doc => checks::doc::run(&self.project_path).await,
            CheckType::PublishDryRun => checks::publish::run(&self.project_path).await,
            CheckType::Standards => checks::standards::run(&self.project_path).await,
            CheckType::DocCoverage => checks::doc::coverage_check(&self.project_path).await,
            CheckType::License => checks::license::run(&self.project_path).await,
            CheckType::Semver => checks::semver::run(&self.project_path).await,
        };

        // Since we already awaited the check, just handle the result
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
    fn get_stage_for_check(&self, check_type: CheckType) -> PipelineStage {
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

    /// Check if safety pipeline is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the current configuration
    pub fn config(&self) -> &SafetyConfig {
        &self.config
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let temp_dir = TempDir::new().unwrap();
        let pipeline = SafetyPipeline::new(temp_dir.path()).await;

        // Should succeed even if config doesn't exist
        assert!(pipeline.is_ok());
    }

    #[test]
    fn test_get_stage_for_check() {
        let config = SafetyConfig::default();
        let temp_dir = TempDir::new().unwrap();
        let bypass_manager = BypassManager::new(&config.bypass).unwrap();

        let pipeline = SafetyPipeline {
            config,
            project_path: temp_dir.path().to_path_buf(),
            bypass_manager,
        };

        assert_eq!(
            pipeline.get_stage_for_check(CheckType::Format),
            PipelineStage::PreCommit
        );
        assert_eq!(
            pipeline.get_stage_for_check(CheckType::Test),
            PipelineStage::PrePush
        );
        assert_eq!(
            pipeline.get_stage_for_check(CheckType::Semver),
            PipelineStage::Publish
        );
    }
}

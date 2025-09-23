//! Main safety pipeline execution engine

use crate::Result;
// Removed unused indicatif imports as they are now in execution module
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::{
    bypass::BypassManager, config::SafetyConfig, execution::ExecutionManager, report::SafetyReport,
    PipelineStage, SafetyResult,
};

/// Main safety pipeline coordinator
pub struct SafetyPipeline {
    config: SafetyConfig,
    project_path: PathBuf,
    bypass_manager: BypassManager,
    execution_manager: ExecutionManager,
}

impl SafetyPipeline {
    /// Create a new safety pipeline
    pub async fn new(project_path: impl AsRef<Path>) -> Result<Self> {
        let config = SafetyConfig::load_or_default().await?;
        let bypass_manager = BypassManager::new(&config.bypass)?;
        let execution_manager = ExecutionManager::new(config.show_progress, config.parallel_checks);

        Ok(Self {
            config,
            project_path: project_path.as_ref().to_path_buf(),
            bypass_manager,
            execution_manager,
        })
    }

    /// Run safety checks for a specific stage
    pub async fn run_checks(&self, stage: PipelineStage) -> Result<SafetyReport> {
        // Check if pipeline or stage is disabled
        if let Some(report) = self.check_disabled_pipeline(stage) {
            return Ok(report);
        }

        let start_time = Instant::now();
        let mut report = SafetyReport::new(stage);
        let stage_config = self.config.get_stage_config(stage);

        // Setup progress indicators and display header
        let multi_progress = self.execution_manager.setup_progress_display(stage);

        // Execute the checks
        self.execution_manager
            .execute_stage_checks(
                stage_config,
                &mut report,
                multi_progress.as_ref(),
                &self.project_path,
            )
            .await?;

        // Finalize the report
        self.finalize_report(&mut report, start_time).await;

        Ok(report)
    }

    /// Check if pipeline or stage is disabled, return early report if so
    fn check_disabled_pipeline(&self, stage: PipelineStage) -> Option<SafetyReport> {
        if !self.config.enabled {
            println!("⚠️  Safety pipeline is disabled");
            return Some(SafetyReport::new(stage));
        }

        let stage_config = self.config.get_stage_config(stage);
        if !stage_config.enabled {
            println!("⚠️  {} stage is disabled", stage.display_name());
            return Some(SafetyReport::new(stage));
        }

        None
    }

    /// Finalize the report with duration and save
    async fn finalize_report(&self, report: &mut SafetyReport, start_time: Instant) {
        report.total_duration = start_time.elapsed();

        if let Err(e) = report.save_to_file().await {
            eprintln!("Warning: Failed to save safety report: {}", e);
        }
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

        let execution_manager = ExecutionManager::new(false, false);
        let pipeline = SafetyPipeline {
            config,
            project_path: temp_dir.path().to_path_buf(),
            bypass_manager,
            execution_manager,
        };

        assert_eq!(
            get_stage_for_check(CheckType::Format),
            PipelineStage::PreCommit
        );
        assert_eq!(get_stage_for_check(CheckType::Test), PipelineStage::PrePush);
        assert_eq!(
            get_stage_for_check(CheckType::Semver),
            PipelineStage::Publish
        );
    }
}

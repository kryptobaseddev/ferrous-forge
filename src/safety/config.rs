//! Safety pipeline configuration

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;

use super::{CheckType, PipelineStage};

/// Safety pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Whether safety pipeline is enabled
    pub enabled: bool,
    /// Strict mode - block operations on failure
    pub strict_mode: bool,
    /// Show progress indicators
    pub show_progress: bool,
    /// Run checks in parallel when possible
    pub parallel_checks: bool,
    /// Pre-commit configuration
    pub pre_commit: StageConfig,
    /// Pre-push configuration
    pub pre_push: StageConfig,
    /// Publish configuration
    pub publish: StageConfig,
    /// Bypass configuration
    pub bypass: BypassConfig,
}

/// Configuration for a specific pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageConfig {
    /// Whether this stage is enabled
    pub enabled: bool,
    /// Timeout for this stage
    pub timeout_seconds: u64,
    /// Checks to run in this stage
    pub checks: Vec<CheckType>,
    /// Whether to continue on non-critical failures
    pub continue_on_warning: bool,
}

/// Bypass system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BypassConfig {
    /// Whether bypass is enabled
    pub enabled: bool,
    /// Require reason for bypass
    pub require_reason: bool,
    /// Require confirmation for bypass
    pub require_confirmation: bool,
    /// Log all bypasses for audit
    pub log_bypasses: bool,
    /// Maximum bypasses per day
    pub max_bypasses_per_day: u32,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strict_mode: true,
            show_progress: true,
            parallel_checks: true,
            pre_commit: StageConfig {
                enabled: true,
                timeout_seconds: 300, // 5 minutes
                checks: CheckType::for_stage(PipelineStage::PreCommit),
                continue_on_warning: false,
            },
            pre_push: StageConfig {
                enabled: true,
                timeout_seconds: 600, // 10 minutes
                checks: CheckType::for_stage(PipelineStage::PrePush),
                continue_on_warning: false,
            },
            publish: StageConfig {
                enabled: true,
                timeout_seconds: 900, // 15 minutes
                checks: CheckType::for_stage(PipelineStage::Publish),
                continue_on_warning: false,
            },
            bypass: BypassConfig {
                enabled: false, // Disabled by default for security
                require_reason: true,
                require_confirmation: true,
                log_bypasses: true,
                max_bypasses_per_day: 3,
            },
        }
    }
}

impl SafetyConfig {
    /// Load configuration from file, or return default if not found
    pub async fn load_or_default() -> Result<Self> {
        match Self::load().await {
            Ok(config) => Ok(config),
            Err(_) => Ok(Self::default()),
        }
    }

    /// Load configuration from file
    pub async fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        let contents = fs::read_to_string(&config_path)
            .await
            .map_err(|e| Error::config(format!("Failed to read safety config: {}", e)))?;

        let config: Self = toml::from_str(&contents)
            .map_err(|e| Error::config(format!("Failed to parse safety config: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize safety config: {}", e)))?;

        fs::write(&config_path, contents)
            .await
            .map_err(|e| Error::config(format!("Failed to write safety config: {}", e)))?;

        Ok(())
    }

    /// Get the path to the safety configuration file
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir_path()?;
        Ok(config_dir.join("safety.toml"))
    }

    /// Get configuration for a specific stage
    pub fn get_stage_config(&self, stage: PipelineStage) -> &StageConfig {
        match stage {
            PipelineStage::PreCommit => &self.pre_commit,
            PipelineStage::PrePush => &self.pre_push,
            PipelineStage::Publish => &self.publish,
        }
    }

    /// Get mutable configuration for a specific stage
    pub fn get_stage_config_mut(&mut self, stage: PipelineStage) -> &mut StageConfig {
        match stage {
            PipelineStage::PreCommit => &mut self.pre_commit,
            PipelineStage::PrePush => &mut self.pre_push,
            PipelineStage::Publish => &mut self.publish,
        }
    }

    /// Check if a specific check is enabled for a stage
    pub fn is_check_enabled(&self, stage: PipelineStage, check: CheckType) -> bool {
        let stage_config = self.get_stage_config(stage);
        stage_config.enabled && stage_config.checks.contains(&check)
    }

    /// Get timeout for a specific stage
    pub fn get_timeout(&self, stage: PipelineStage) -> Duration {
        Duration::from_secs(self.get_stage_config(stage).timeout_seconds)
    }

    /// Set a configuration value
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "enabled" => {
                self.enabled = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for enabled"))?;
            }
            "strict_mode" => {
                self.strict_mode = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for strict_mode"))?;
            }
            "show_progress" => {
                self.show_progress = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for show_progress"))?;
            }
            "parallel_checks" => {
                self.parallel_checks = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for parallel_checks"))?;
            }
            "pre_commit.enabled" => {
                self.pre_commit.enabled = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for pre_commit.enabled"))?;
            }
            "pre_commit.timeout_seconds" => {
                self.pre_commit.timeout_seconds = value
                    .parse()
                    .map_err(|_| Error::config("Invalid number for pre_commit.timeout_seconds"))?;
            }
            "pre_push.enabled" => {
                self.pre_push.enabled = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for pre_push.enabled"))?;
            }
            "pre_push.timeout_seconds" => {
                self.pre_push.timeout_seconds = value
                    .parse()
                    .map_err(|_| Error::config("Invalid number for pre_push.timeout_seconds"))?;
            }
            "publish.enabled" => {
                self.publish.enabled = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for publish.enabled"))?;
            }
            "publish.timeout_seconds" => {
                self.publish.timeout_seconds = value
                    .parse()
                    .map_err(|_| Error::config("Invalid number for publish.timeout_seconds"))?;
            }
            "bypass.enabled" => {
                self.bypass.enabled = value
                    .parse()
                    .map_err(|_| Error::config("Invalid boolean value for bypass.enabled"))?;
            }
            _ => return Err(Error::config(format!("Unknown safety config key: {}", key))),
        }

        Ok(())
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "enabled" => Some(self.enabled.to_string()),
            "strict_mode" => Some(self.strict_mode.to_string()),
            "show_progress" => Some(self.show_progress.to_string()),
            "parallel_checks" => Some(self.parallel_checks.to_string()),
            "pre_commit.enabled" => Some(self.pre_commit.enabled.to_string()),
            "pre_commit.timeout_seconds" => Some(self.pre_commit.timeout_seconds.to_string()),
            "pre_push.enabled" => Some(self.pre_push.enabled.to_string()),
            "pre_push.timeout_seconds" => Some(self.pre_push.timeout_seconds.to_string()),
            "publish.enabled" => Some(self.publish.enabled.to_string()),
            "publish.timeout_seconds" => Some(self.publish.timeout_seconds.to_string()),
            "bypass.enabled" => Some(self.bypass.enabled.to_string()),
            _ => None,
        }
    }
}

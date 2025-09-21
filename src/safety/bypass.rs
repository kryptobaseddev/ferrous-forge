//! Emergency bypass system for safety pipeline
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use super::{config::BypassConfig, PipelineStage};

/// Bypass manager for emergency situations
pub struct BypassManager {
    config: BypassConfig,
}

/// Active bypass record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBypass {
    /// Stage being bypassed
    pub stage: PipelineStage,
    /// Reason for bypass
    pub reason: String,
    /// User who created the bypass
    pub user: String,
    /// When the bypass was created
    pub created_at: DateTime<Utc>,
    /// When the bypass expires
    pub expires_at: DateTime<Utc>,
}

/// Bypass audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BypassLogEntry {
    /// Stage that was bypassed
    pub stage: PipelineStage,
    /// Reason for bypass
    pub reason: String,
    /// User who bypassed
    pub user: String,
    /// Timestamp of bypass
    pub timestamp: DateTime<Utc>,
    /// Whether bypass was successful
    pub successful: bool,
}

impl BypassManager {
    /// Create a new bypass manager
    pub fn new(config: &BypassConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Check if bypass is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Create a temporary bypass
    pub async fn create_bypass(
        &self,
        stage: PipelineStage,
        reason: String,
        user: String,
        duration_hours: u64,
    ) -> Result<ActiveBypass> {
        if !self.config.enabled {
            return Err(Error::safety("Bypass system is disabled"));
        }

        if self.config.require_reason && reason.trim().is_empty() {
            return Err(Error::safety("Bypass reason is required"));
        }

        // Check daily bypass limit
        if self.config.max_bypasses_per_day > 0 {
            let today_count = self.count_bypasses_today(&user).await?;
            if today_count >= self.config.max_bypasses_per_day {
                return Err(Error::safety(format!(
                    "Daily bypass limit reached ({}/{})",
                    today_count, self.config.max_bypasses_per_day
                )));
            }
        }

        let bypass = ActiveBypass {
            stage,
            reason: reason.clone(),
            user: user.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(duration_hours as i64),
        };

        // Save active bypass
        self.save_active_bypass(&bypass).await?;

        // Log the bypass
        if self.config.log_bypasses {
            self.log_bypass(&bypass, true).await?;
        }

        Ok(bypass)
    }

    /// Check for active bypass
    pub async fn check_active_bypass(&self, stage: PipelineStage) -> Result<Option<ActiveBypass>> {
        if !self.config.enabled {
            return Ok(None);
        }

        let bypass_path = self.get_active_bypass_path(stage)?;

        if !bypass_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&bypass_path).await?;
        let bypass: ActiveBypass = serde_json::from_str(&contents)
            .map_err(|e| Error::parse(format!("Failed to parse bypass: {}", e)))?;

        // Check if bypass has expired
        if Utc::now() > bypass.expires_at {
            // Remove expired bypass (ignore errors if file doesn't exist)
            if let Err(e) = fs::remove_file(&bypass_path).await {
                eprintln!("Warning: Failed to remove expired bypass file: {}", e);
            }
            return Ok(None);
        }

        Ok(Some(bypass))
    }

    /// Remove an active bypass
    pub async fn remove_bypass(&self, stage: PipelineStage) -> Result<()> {
        let bypass_path = self.get_active_bypass_path(stage)?;

        if bypass_path.exists() {
            fs::remove_file(&bypass_path).await?;
        }

        Ok(())
    }

    /// Get bypass audit log
    pub async fn get_audit_log(&self, limit: usize) -> Result<Vec<BypassLogEntry>> {
        let log_path = self.get_audit_log_path()?;

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&log_path).await?;
        let mut entries: Vec<BypassLogEntry> = contents
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        // Sort by timestamp (newest first) and limit
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);

        Ok(entries)
    }

    /// Count bypasses for a user today
    async fn count_bypasses_today(&self, user: &str) -> Result<u32> {
        let today = Utc::now().date_naive();
        let log = self.get_audit_log(100).await?; // Check last 100 entries

        let count = log
            .iter()
            .filter(|entry| entry.user == user && entry.timestamp.date_naive() == today)
            .count() as u32;

        Ok(count)
    }

    /// Save active bypass to file
    async fn save_active_bypass(&self, bypass: &ActiveBypass) -> Result<()> {
        let bypass_path = self.get_active_bypass_path(bypass.stage)?;

        if let Some(parent) = bypass_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let contents = serde_json::to_string_pretty(bypass)
            .map_err(|e| Error::parse(format!("Failed to serialize bypass: {}", e)))?;

        fs::write(&bypass_path, contents).await?;

        Ok(())
    }

    /// Log bypass to audit trail
    async fn log_bypass(&self, bypass: &ActiveBypass, successful: bool) -> Result<()> {
        let log_path = self.get_audit_log_path()?;

        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let entry = BypassLogEntry {
            stage: bypass.stage,
            reason: bypass.reason.clone(),
            user: bypass.user.clone(),
            timestamp: bypass.created_at,
            successful,
        };

        let log_line = serde_json::to_string(&entry)
            .map_err(|e| Error::parse(format!("Failed to serialize log entry: {}", e)))?;

        // Append to log file
        let mut contents = if log_path.exists() {
            fs::read_to_string(&log_path).await?
        } else {
            String::new()
        };

        contents.push_str(&log_line);
        contents.push('\n');

        fs::write(&log_path, contents).await?;

        Ok(())
    }

    /// Get path for active bypass file
    fn get_active_bypass_path(&self, stage: PipelineStage) -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir_path()?;
        Ok(config_dir
            .join("safety-bypasses")
            .join(format!("{}.json", stage.name())))
    }

    /// Get path for audit log
    fn get_audit_log_path(&self) -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir_path()?;
        Ok(config_dir.join("safety-bypasses").join("audit.log"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_bypass_manager_creation() {
        let config = BypassConfig {
            enabled: true,
            require_reason: true,
            require_confirmation: true,
            log_bypasses: true,
            max_bypasses_per_day: 3,
        };

        let manager = BypassManager::new(&config).unwrap();
        assert!(manager.is_enabled());
    }

    #[tokio::test]
    async fn test_bypass_creation() {
        let config = BypassConfig {
            enabled: true,
            require_reason: true,
            require_confirmation: false,
            log_bypasses: false,
            max_bypasses_per_day: 0, // No limit
        };

        let manager = BypassManager::new(&config).unwrap();

        let bypass = manager
            .create_bypass(
                PipelineStage::PreCommit,
                "test reason".to_string(),
                "test_user".to_string(),
                1, // 1 hour
            )
            .await
            .unwrap();

        assert_eq!(bypass.stage, PipelineStage::PreCommit);
        assert_eq!(bypass.reason, "test reason");
        assert_eq!(bypass.user, "test_user");
    }
}

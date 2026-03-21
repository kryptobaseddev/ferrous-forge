//! Audit logging for lock/unlock operations
//!
//! @task T015
//! @epic T014

use crate::config::hierarchy::ConfigLevel;
use crate::config::locking::LockEntry;
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::warn;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Operation type (lock, unlock, `attempt_blocked`)
    pub operation: String,
    /// Configuration key affected
    pub key: String,
    /// The value (for lock operations)
    pub value: Option<String>,
    /// Timestamp of the operation
    pub timestamp: DateTime<Utc>,
    /// User who performed the operation
    pub user: String,
    /// Configuration level
    pub level: String,
    /// Reason provided for the operation
    pub reason: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(
        operation: impl Into<String>,
        key: impl Into<String>,
        level: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            operation: operation.into(),
            key: key.into(),
            value: None,
            timestamp: Utc::now(),
            user: whoami::username(),
            level: level.into(),
            reason: reason.into(),
            success: true,
            error: None,
        }
    }

    /// Set the value field
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Mark as failed with error
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error = Some(error.into());
        self
    }
}

/// Get the audit log file path
fn audit_log_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| Error::config("Could not find config directory"))?;
    Ok(config_dir.join("ferrous-forge").join("audit.log"))
}

/// Log an unlock operation
///
/// # Errors
///
/// Returns an error if writing to the audit log fails.
pub async fn log_unlock(
    key: &str,
    entry: &LockEntry,
    level: ConfigLevel,
    reason: &str,
) -> Result<()> {
    let audit_entry =
        AuditEntry::new("unlock", key, level.display_name(), reason).with_value(&entry.value);

    append_to_audit_log(audit_entry).await
}

/// Log a blocked change attempt
///
/// # Errors
///
/// Returns an error if writing to the audit log fails.
pub async fn log_blocked_attempt(
    key: &str,
    attempted_value: &str,
    level: ConfigLevel,
    reason: &str,
) -> Result<()> {
    let audit_entry = AuditEntry::new("attempt_blocked", key, level.display_name(), reason)
        .with_value(attempted_value)
        .failed("Configuration key is locked");

    append_to_audit_log(audit_entry).await
}

/// Append entry to audit log
///
/// # Errors
///
/// Returns an error if writing to the audit log fails.
async fn append_to_audit_log(entry: AuditEntry) -> Result<()> {
    let path = audit_log_path()?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let line = serde_json::to_string(&entry)
        .map_err(|e| Error::config(format!("Failed to serialize audit entry: {}", e)))?;

    // Append newline
    let line = format!("{}\n", line);

    // Open file for appending (create if doesn't exist)
    use tokio::io::AsyncWriteExt;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|e| Error::config(format!("Failed to open audit log: {}", e)))?;

    file.write_all(line.as_bytes())
        .await
        .map_err(|e| Error::config(format!("Failed to write to audit log: {}", e)))?;

    Ok(())
}

/// Read the audit log
///
/// # Errors
///
/// Returns an error if reading the audit log fails.
pub async fn read_audit_log() -> Result<Vec<AuditEntry>> {
    let path = audit_log_path()?;

    if !path.exists() {
        return Ok(vec![]);
    }

    let contents = fs::read_to_string(&path)
        .await
        .map_err(|e| Error::config(format!("Failed to read audit log: {}", e)))?;

    let mut entries = Vec::new();
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<AuditEntry>(line) {
            Ok(entry) => entries.push(entry),
            Err(e) => warn!("Failed to parse audit log entry: {}", e),
        }
    }

    Ok(entries)
}

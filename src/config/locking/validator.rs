//! Configuration change validator that checks locks before allowing changes
//!
//! @task T015
//! @epic T014

use crate::config::locking::{HierarchicalLockManager, audit_log};
use crate::{Error, Result};

/// Configuration change validator that checks locks before allowing changes
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a configuration change against locks
    ///
    /// # Errors
    ///
    /// Returns an error if the change violates a lock.
    #[allow(clippy::collapsible_if)]
    pub async fn validate_change(key: &str, new_value: &str) -> Result<()> {
        let lock_manager = HierarchicalLockManager::load().await?;

        if let Some((level, entry)) = lock_manager.is_locked(key) {
            if entry.value != new_value {
                // Log the blocked attempt
                audit_log::log_blocked_attempt(
                    key,
                    new_value,
                    level,
                    "Attempted change while locked",
                )
                .await?;

                return Err(Error::config(format!(
                    "Cannot change '{}' - it is locked at {} level\nCurrent value: {}\nAttempted value: {}\nLock reason: {}\n\nTo unlock, run:\n  ferrous-forge config unlock {} --level={} --reason=\"...\"",
                    key,
                    level.display_name(),
                    entry.value,
                    new_value,
                    entry.reason,
                    key,
                    level.display_name().to_lowercase()
                )));
            }
        }

        Ok(())
    }

    /// Check if a key can be modified (not locked or same value)
    pub async fn can_modify(key: &str, new_value: &str) -> bool {
        matches!(Self::validate_change(key, new_value).await, Ok(()))
    }
}

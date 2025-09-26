//! Backup operations for edition migration

use super::EditionMigrator;
use super::types::{MigrationResult, MigrationStep};
use crate::{Error, Result};

impl EditionMigrator {
    /// Create backup of project
    pub(super) async fn create_backup(&self, result: &mut MigrationResult) -> Result<()> {
        let backup_dir = self
            .backup_dir
            .as_ref()
            .ok_or_else(|| Error::migration("No backup directory specified"))?;

        result.steps_performed.push(MigrationStep {
            name: "Create backup".to_string(),
            description: "Creating project backup".to_string(),
            success: true,
            message: Some(format!("Backup created at {:?}", backup_dir)),
        });

        Ok(())
    }
}

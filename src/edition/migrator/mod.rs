//! Edition migration assistance

pub mod types;
mod backup;
mod cargo_ops;
mod code_migration;
mod testing;

use crate::Result;
use std::path::{Path, PathBuf};

use super::Edition;
pub use types::{
    MigrationOptions, MigrationResult, MigrationRule, MigrationStatus, MigrationStep, TestResults,
};

/// Edition migrator for upgrading projects
pub struct EditionMigrator {
    project_path: PathBuf,
    backup_dir: Option<PathBuf>,
}

impl EditionMigrator {
    /// Create a new edition migrator
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
            backup_dir: None,
        }
    }

    /// Set backup directory
    pub fn with_backup(mut self, backup_dir: impl AsRef<Path>) -> Self {
        self.backup_dir = Some(backup_dir.as_ref().to_path_buf());
        self
    }

    /// Migrate project to target edition
    pub async fn migrate(
        &self,
        target_edition: Edition,
        options: MigrationOptions,
    ) -> Result<MigrationResult> {
        let mut result = MigrationResult::default();

        // Check if migration is needed
        if let Some(result) = self.check_migration_needed(target_edition).await? {
            return Ok(result);
        }

        result.status = MigrationStatus::InProgress;

        // Execute migration steps
        self.execute_migration_steps(target_edition, &options, &mut result).await?;

        result.status = MigrationStatus::Complete;
        Ok(result)
    }

    /// Check if migration is needed
    async fn check_migration_needed(
        &self,
        target_edition: Edition,
    ) -> Result<Option<MigrationResult>> {
        let manifest_path = self.project_path.join("Cargo.toml");
        let current_edition = super::detect_edition(&manifest_path).await?;

        if current_edition >= target_edition {
            let mut result = MigrationResult::default();
            result.status = MigrationStatus::AlreadyUpToDate;
            return Ok(Some(result));
        }

        Ok(None)
    }

    /// Execute all migration steps based on options
    async fn execute_migration_steps(
        &self,
        target_edition: Edition,
        options: &MigrationOptions,
        result: &mut MigrationResult,
    ) -> Result<()> {
        if options.create_backup {
            self.create_backup(result).await?;
        }

        self.update_cargo_toml(target_edition, result).await?;

        if options.apply_code_migrations {
            self.apply_code_migrations(target_edition, result).await?;
        }

        if options.update_dependencies {
            self.update_dependencies(result).await?;
        }

        if options.format_code {
            self.apply_rustfmt(result).await?;
        }

        if options.run_tests {
            self.run_tests(result).await?;
        }

        if options.commit_changes {
            self.commit_changes(result).await?;
        }

        Ok(())
    }
}
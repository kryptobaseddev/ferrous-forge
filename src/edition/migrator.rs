//! Edition migration assistance

use crate::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

use super::Edition;

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

        // Check current edition
        let manifest_path = self.project_path.join("Cargo.toml");
        let current_edition = super::detect_edition(&manifest_path).await?;

        if current_edition >= target_edition {
            result.status = MigrationStatus::AlreadyUpToDate;
            return Ok(result);
        }

        // Create backup if requested
        if options.create_backup {
            self.create_backup(&mut result).await?;
        }

        // Check for uncommitted changes if strict mode
        if options.check_git && self.has_uncommitted_changes()? {
            return Err(Error::migration(
                "Uncommitted changes detected. Please commit or stash changes before migration.",
            ));
        }

        // Run cargo fix --edition
        self.run_cargo_fix(target_edition, &options, &mut result)
            .await?;

        // Update Cargo.toml
        if options.update_manifest {
            self.update_manifest(target_edition, &mut result).await?;
        }

        // Run tests if requested
        if options.run_tests {
            self.run_tests(&mut result).await?;
        }

        result.status = MigrationStatus::Success;
        result.new_edition = Some(target_edition);

        Ok(result)
    }

    /// Create backup of the project
    async fn create_backup(&self, result: &mut MigrationResult) -> Result<()> {
        let backup_dir = self
            .backup_dir
            .clone()
            .unwrap_or_else(|| self.project_path.join(".ferrous-forge-backup"));

        // Create backup directory
        fs::create_dir_all(&backup_dir).await?;

        // Copy important files
        let manifest_src = self.project_path.join("Cargo.toml");
        let manifest_dst = backup_dir.join("Cargo.toml");
        fs::copy(&manifest_src, &manifest_dst).await?;

        result.backup_path = Some(backup_dir);
        result
            .messages
            .push("Backup created successfully".to_string());

        Ok(())
    }

    /// Check for uncommitted changes
    fn has_uncommitted_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .current_dir(&self.project_path)
            .args(&["status", "--porcelain"])
            .output()?;

        Ok(!output.stdout.is_empty())
    }

    /// Run cargo fix --edition
    async fn run_cargo_fix(
        &self,
        _target_edition: Edition,
        options: &MigrationOptions,
        result: &mut MigrationResult,
    ) -> Result<()> {
        let cargo_path =
            which::which("cargo").map_err(|_| Error::rust_not_found("cargo not found"))?;

        let mut args = vec!["fix", "--edition"];

        if options.fix_idioms {
            args.push("--edition-idioms");
        }

        if options.allow_dirty {
            args.push("--allow-dirty");
        }

        if options.allow_staged {
            args.push("--allow-staged");
        }

        if options.all_targets {
            args.push("--all-targets");
        }

        let output = Command::new(cargo_path)
            .current_dir(&self.project_path)
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            result.errors.push(format!("cargo fix failed: {}", stderr));
            result.status = MigrationStatus::PartialSuccess;

            if !options.continue_on_error {
                return Err(Error::migration(format!("cargo fix failed: {}", stderr)));
            }
        } else {
            result
                .messages
                .push("cargo fix completed successfully".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        result.messages.push(stdout.to_string());

        Ok(())
    }

    /// Update Cargo.toml with new edition
    async fn update_manifest(
        &self,
        target_edition: Edition,
        result: &mut MigrationResult,
    ) -> Result<()> {
        let manifest_path = self.project_path.join("Cargo.toml");
        let contents = fs::read_to_string(&manifest_path).await?;

        let mut manifest: toml::Value = toml::from_str(&contents)?;

        // Update edition field
        if let Some(package) = manifest.get_mut("package") {
            if let Some(table) = package.as_table_mut() {
                table.insert(
                    "edition".to_string(),
                    toml::Value::String(target_edition.as_str().to_string()),
                );
            }
        }

        let new_contents = toml::to_string_pretty(&manifest)
            .map_err(|e| Error::parse(format!("Failed to serialize TOML: {}", e)))?;
        fs::write(&manifest_path, new_contents).await?;

        result.messages.push(format!(
            "Updated Cargo.toml to edition {}",
            target_edition.as_str()
        ));

        Ok(())
    }

    /// Run tests after migration
    async fn run_tests(&self, result: &mut MigrationResult) -> Result<()> {
        let output = Command::new("cargo")
            .current_dir(&self.project_path)
            .arg("test")
            .output()?;

        if !output.status.success() {
            result
                .warnings
                .push("Tests failed after migration".to_string());
            result.status = MigrationStatus::PartialSuccess;
        } else {
            result
                .messages
                .push("All tests passed after migration".to_string());
        }

        Ok(())
    }

    /// Rollback migration using backup
    pub async fn rollback(&self) -> Result<()> {
        if let Some(backup_dir) = &self.backup_dir {
            // Restore Cargo.toml
            let backup_manifest = backup_dir.join("Cargo.toml");
            let project_manifest = self.project_path.join("Cargo.toml");

            if backup_manifest.exists() {
                fs::copy(&backup_manifest, &project_manifest).await?;
            }

            Ok(())
        } else {
            Err(Error::migration("No backup available for rollback"))
        }
    }
}

/// Migration options
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Create backup before migration
    pub create_backup: bool,
    /// Check for uncommitted git changes
    pub check_git: bool,
    /// Update Cargo.toml
    pub update_manifest: bool,
    /// Run tests after migration
    pub run_tests: bool,
    /// Apply edition idioms
    pub fix_idioms: bool,
    /// Allow migration with dirty working directory
    pub allow_dirty: bool,
    /// Allow migration with staged changes
    pub allow_staged: bool,
    /// Fix all targets
    pub all_targets: bool,
    /// Continue on error
    pub continue_on_error: bool,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            create_backup: true,
            check_git: true,
            update_manifest: true,
            run_tests: false,
            fix_idioms: false,
            allow_dirty: false,
            allow_staged: false,
            all_targets: true,
            continue_on_error: false,
        }
    }
}

/// Migration result
#[derive(Debug, Clone, Default)]
pub struct MigrationResult {
    /// Migration status
    pub status: MigrationStatus,
    /// New edition after migration
    pub new_edition: Option<Edition>,
    /// Backup path if created
    pub backup_path: Option<PathBuf>,
    /// Messages from the migration
    pub messages: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Errors
    pub errors: Vec<String>,
}

/// Migration status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStatus {
    /// Not started
    Pending,
    /// Already on target edition
    AlreadyUpToDate,
    /// Migration successful
    Success,
    /// Partially successful (with warnings)
    PartialSuccess,
    /// Migration failed
    Failed,
}

impl Default for MigrationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_migration_options_default() {
        let options = MigrationOptions::default();
        assert!(options.create_backup);
        assert!(options.check_git);
        assert!(options.update_manifest);
        assert!(!options.run_tests);
    }

    #[tokio::test]
    async fn test_migrator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let migrator =
            EditionMigrator::new(temp_dir.path()).with_backup(temp_dir.path().join("backup"));

        assert_eq!(migrator.project_path, temp_dir.path());
    }
}

//! Cargo.toml operations for edition migration

use super::types::{MigrationResult, MigrationStep};
use super::{Edition, EditionMigrator};
use crate::Result;
use tokio::fs;

impl EditionMigrator {
    /// Update Cargo.toml with new edition
    pub(super) async fn update_cargo_toml(
        &self,
        target_edition: Edition,
        result: &mut MigrationResult,
    ) -> Result<()> {
        let manifest_path = self.project_path.join("Cargo.toml");
        let content = fs::read_to_string(&manifest_path).await?;

        // Replace edition line
        let new_content = if content.contains("edition") {
            content
                .replace(
                    "edition = \"2015\"",
                    &format!("edition = \"{}\"", target_edition),
                )
                .replace(
                    "edition = \"2018\"",
                    &format!("edition = \"{}\"", target_edition),
                )
                .replace(
                    "edition = \"2021\"",
                    &format!("edition = \"{}\"", target_edition),
                )
        } else {
            // Add edition if not present
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut found_package = false;

            for line in lines {
                new_lines.push(line.to_string());
                if line.trim() == "[package]" {
                    found_package = true;
                } else if found_package && line.contains("version") {
                    new_lines.push(format!("edition = \"{}\"", target_edition));
                    found_package = false;
                }
            }

            new_lines.join("\n")
        };

        fs::write(&manifest_path, new_content).await?;

        result.steps_performed.push(MigrationStep {
            name: "Update Cargo.toml".to_string(),
            description: format!("Updated edition to {}", target_edition),
            success: true,
            message: Some("Edition updated successfully".to_string()),
        });

        Ok(())
    }

    /// Update dependencies for new edition
    pub(super) async fn update_dependencies(&self, result: &mut MigrationResult) -> Result<()> {
        let manifest_path = self.project_path.join("Cargo.toml");
        let content = fs::read_to_string(&manifest_path).await?;

        // Update common dependencies that need version bumps for newer editions
        let updated = content
            .replace("tokio = \"1.0\"", "tokio = \"1.20\"")
            .replace("serde = \"1.0\"", "serde = \"1.0.150\"");

        if updated != content {
            fs::write(&manifest_path, updated).await?;

            result.steps_performed.push(MigrationStep {
                name: "Update dependencies".to_string(),
                description: "Updated dependencies for new edition".to_string(),
                success: true,
                message: Some("Dependencies updated".to_string()),
            });
        }

        Ok(())
    }
}

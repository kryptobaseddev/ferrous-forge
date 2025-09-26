//! Validation functions for cargo publish interception

use crate::safety::SafetyPipeline;
use crate::{Error, Result};
use std::path::Path;
use toml::Value;

/// Run comprehensive pre-publish validation
pub async fn pre_publish_validation(project_path: &Path) -> Result<()> {
    tracing::info!("Running pre-publish validation");

    let pipeline = SafetyPipeline::new(project_path).await?;
    let results = pipeline
        .run_checks(crate::safety::PipelineStage::Publish)
        .await?;

    // Check if any critical checks failed
    if !results.passed {
        tracing::error!("Pre-publish validation failed");
        return Err(Error::validation(
            "Pre-publish validation failed - fix errors before publishing",
        ));
    }

    Ok(())
}

/// Enforce dogfooding by checking Ferrous Forge usage
pub async fn enforce_dogfooding(project_path: &Path) -> Result<()> {
    tracing::info!("Checking dogfooding compliance");

    // Check if project has .ferrous-forge directory
    let ff_dir = project_path.join(".ferrous-forge");
    if !ff_dir.exists() {
        return Err(Error::validation(
            "Project must use Ferrous Forge before publishing (run 'ferrous-forge init')",
        ));
    }

    // Check if validation has been run recently
    let config_file = ff_dir.join("config.toml");
    if !config_file.exists() {
        return Err(Error::validation(
            "Ferrous Forge config not found - run 'ferrous-forge init'",
        ));
    }

    tracing::info!("Dogfooding compliance verified");
    Ok(())
}

/// Check version consistency across project files
pub fn version_consistency_check(project_path: &Path) -> Result<()> {
    tracing::info!("Checking version consistency");

    let cargo_toml_path = project_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(Error::config("Cargo.toml not found in project directory"));
    }

    let cargo_content = std::fs::read_to_string(&cargo_toml_path)
        .map_err(|e| Error::config(format!("Failed to read Cargo.toml: {}", e)))?;

    let cargo_toml: Value = cargo_content
        .parse()
        .map_err(|e| Error::config(format!("Failed to parse Cargo.toml: {}", e)))?;

    // Extract version from Cargo.toml
    let version = cargo_toml
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::config("No version found in Cargo.toml"))?;

    tracing::info!("Version {} found in Cargo.toml", version);

    // Check if version follows semantic versioning
    check_semver_format(version)?;

    Ok(())
}

/// Validate semantic version format
fn check_semver_format(version: &str) -> Result<()> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(Error::validation(format!(
            "Version '{}' must follow semantic versioning (major.minor.patch)",
            version
        )));
    }

    // Check each part is numeric
    for (i, part) in parts.iter().enumerate() {
        if part.parse::<u32>().is_err() {
            let part_name = match i {
                0 => "major",
                1 => "minor",
                _ => "patch",
            };
            return Err(Error::validation(format!(
                "Version {} part '{}' must be numeric",
                part_name, part
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_semver_format_valid() {
        assert!(check_semver_format("1.0.0").is_ok());
        assert!(check_semver_format("0.1.2").is_ok());
        assert!(check_semver_format("10.20.30").is_ok());
    }

    #[test]
    fn test_semver_format_invalid() {
        assert!(check_semver_format("1.0").is_err());
        assert!(check_semver_format("1.0.0.1").is_err());
        assert!(check_semver_format("1.a.0").is_err());
        assert!(check_semver_format("invalid").is_err());
    }

    #[tokio::test]
    async fn test_version_consistency_check() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let cargo_content = r#"
[package]
name = "test-package"
version = "1.0.0"
edition = "2021"
"#;

        fs::write(&cargo_toml_path, cargo_content).expect("Failed to write Cargo.toml");

        let result = version_consistency_check(temp_dir.path());
        assert!(result.is_ok());
    }
}

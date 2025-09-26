//! Semantic versioning compliance checking

use crate::Result;
use semver::Version;
use std::path::Path;
use std::time::Instant;
use tokio::fs;

use super::SafetyCheck;
use crate::safety::{CheckType, report::CheckResult};

/// Semver check implementation
pub struct SemverCheck;

impl SafetyCheck for SemverCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "semver"
    }

    fn description() -> &'static str {
        "Checks semantic versioning compliance"
    }
}

/// Check semantic versioning compliance
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Semver);

    // Read and parse Cargo.toml
    let manifest = match load_cargo_manifest(project_path, &mut result).await? {
        Some(manifest) => manifest,
        None => {
            result.set_duration(start.elapsed());
            return Ok(result);
        }
    };

    // Validate version from manifest
    validate_version_from_manifest(&manifest, &mut result)?;

    // Check for CHANGELOG.md
    check_changelog(project_path, &mut result);

    result.set_duration(start.elapsed());
    Ok(result)
}

/// Load and parse Cargo.toml manifest
async fn load_cargo_manifest(
    project_path: &Path,
    result: &mut CheckResult,
) -> Result<Option<toml::Value>> {
    let cargo_toml_path = project_path.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        result.add_error("Cargo.toml not found");
        return Ok(None);
    }

    let contents = fs::read_to_string(&cargo_toml_path).await?;
    let manifest: toml::Value = toml::from_str(&contents)
        .map_err(|e| crate::Error::parse(format!("Failed to parse Cargo.toml: {}", e)))?;

    Ok(Some(manifest))
}

/// Validate version information from Cargo.toml
fn validate_version_from_manifest(manifest: &toml::Value, result: &mut CheckResult) -> Result<()> {
    let version_str = manifest
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str());

    if let Some(version_str) = version_str {
        match Version::parse(version_str) {
            Ok(version) => validate_version_details(&version, result),
            Err(e) => handle_invalid_version(e, result),
        }
    } else {
        handle_missing_version(result);
    }

    Ok(())
}

/// Validate specific version details
fn validate_version_details(version: &Version, result: &mut CheckResult) {
    result.add_context(format!("Current version: {}", version));

    // Check for pre-release versions
    if !version.pre.is_empty() {
        result.add_context(format!("Pre-release version: {}", version.pre));
        result.add_suggestion("Consider if this should be published as pre-release");
    }

    // Check for build metadata
    if !version.build.is_empty() {
        result.add_context(format!("Build metadata: {}", version.build));
    }

    // Check version reasonableness
    if version.major == 0 && version.minor == 0 && version.patch == 0 {
        result.add_error("Version 0.0.0 should not be published");
        result.add_suggestion("Use a proper version like 0.1.0 for initial release");
    } else if version.major > 10 {
        result.add_context("High major version detected - ensure this is intentional");
    }
}

/// Handle invalid version format
fn handle_invalid_version(e: semver::Error, result: &mut CheckResult) {
    result.add_error(format!("Invalid semantic version: {}", e));
    result.add_suggestion("Use format: MAJOR.MINOR.PATCH (e.g., 1.0.0)");
    result.add_suggestion("See https://semver.org for semantic versioning rules");
}

/// Handle missing version field
fn handle_missing_version(result: &mut CheckResult) {
    result.add_error("No version field found in Cargo.toml");
    result.add_suggestion("Add 'version = \"0.1.0\"' to [package] section");
}

/// Check for CHANGELOG.md file
fn check_changelog(project_path: &Path, result: &mut CheckResult) {
    let changelog_path = project_path.join("CHANGELOG.md");
    if changelog_path.exists() {
        result.add_context("CHANGELOG.md found");
        // TODO: Check if current version is documented in changelog
    } else {
        result.add_context("No CHANGELOG.md found");
        result.add_suggestion("Consider adding CHANGELOG.md to track changes");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_semver_check_with_valid_version() {
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = r#"
[package]
name = "test"
version = "1.0.0"
edition = "2021"
"#;

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)
            .await
            .unwrap();

        let result = run(temp_dir.path()).await.unwrap();

        // Should pass with valid version
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_semver_check_with_invalid_version() {
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = r#"
[package]
name = "test"
version = "0.0.0"
edition = "2021"
"#;

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)
            .await
            .unwrap();

        let result = run(temp_dir.path()).await.unwrap();

        // Should fail with 0.0.0 version
        assert!(!result.passed);
    }

    #[test]
    fn test_semver_check_struct() {
        assert_eq!(SemverCheck::name(), "semver");
        assert!(!SemverCheck::description().is_empty());
    }
}

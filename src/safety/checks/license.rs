//! License validation checking

use crate::Result;
use std::path::Path;
use std::time::Instant;
use tokio::fs;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// License check implementation
pub struct LicenseCheck;

impl SafetyCheck for LicenseCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }
    
    fn name() -> &'static str {
        "license"
    }
    
    fn description() -> &'static str {
        "Validates license compatibility and presence"
    }
}

/// Validate license configuration
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::License);
    
    // Check Cargo.toml for license field
    let cargo_toml_path = project_path.join("Cargo.toml");
    
    if !cargo_toml_path.exists() {
        result.add_error("Cargo.toml not found");
        result.set_duration(start.elapsed());
        return Ok(result);
    }
    
    let contents = fs::read_to_string(&cargo_toml_path).await?;
    let manifest: toml::Value = toml::from_str(&contents)
        .map_err(|e| crate::Error::parse(format!("Failed to parse Cargo.toml: {}", e)))?;
    
    result.set_duration(start.elapsed());
    
    // Check for license field
    let license = manifest
        .get("package")
        .and_then(|p| p.get("license"))
        .and_then(|l| l.as_str());
    
    let license_file = manifest
        .get("package")
        .and_then(|p| p.get("license-file"))
        .and_then(|l| l.as_str());
    
    if license.is_none() && license_file.is_none() {
        result.add_error("No license specified in Cargo.toml");
        result.add_suggestion("Add 'license = \"MIT OR Apache-2.0\"' to [package] section");
        result.add_suggestion("Or add 'license-file = \"LICENSE\"' if using custom license");
    } else if let Some(license_str) = license {
        // Validate license string
        let approved_licenses = [
            "MIT",
            "Apache-2.0", 
            "MIT OR Apache-2.0",
            "Apache-2.0 OR MIT",
            "BSD-3-Clause",
            "BSD-2-Clause",
            "ISC",
            "MPL-2.0",
        ];
        
        if !approved_licenses.iter().any(|&approved| license_str.contains(approved)) {
            result.add_error(format!("Uncommon license detected: {}", license_str));
            result.add_suggestion("Consider using a standard license like 'MIT OR Apache-2.0'");
            result.add_context("This may cause issues with some package managers");
        } else {
            result.add_context(format!("License: {}", license_str));
        }
        
        // Check if license file exists for certain licenses
        if license_str.contains("MIT") || license_str.contains("Apache") {
            let license_files = ["LICENSE", "LICENSE.txt", "LICENSE.md", "LICENSE-MIT", "LICENSE-APACHE"];
            let has_license_file = license_files.iter()
                .any(|&file| project_path.join(file).exists());
            
            if !has_license_file {
                result.add_error("License specified but no LICENSE file found");
                result.add_suggestion("Create a LICENSE file with the license text");
            }
        }
    }
    
    // Check for other important metadata
    let description = manifest
        .get("package")
        .and_then(|p| p.get("description"))
        .and_then(|d| d.as_str());
    
    if description.is_none() || description.unwrap().trim().is_empty() {
        result.add_error("Missing or empty description in Cargo.toml");
        result.add_suggestion("Add a clear description of what your crate does");
    }
    
    let repository = manifest
        .get("package")
        .and_then(|p| p.get("repository"))
        .and_then(|r| r.as_str());
    
    if repository.is_none() {
        result.add_error("Missing repository URL in Cargo.toml");
        result.add_suggestion("Add 'repository = \"https://github.com/user/repo\"' to [package]");
    }
    
    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_license_check_with_valid_license() {
        let temp_dir = TempDir::new().unwrap();
        
        let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "A test crate"
repository = "https://github.com/user/test"
"#;
        
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).await.unwrap();
        
        let result = run(temp_dir.path()).await.unwrap();
        
        // Should pass with valid license
        assert!(result.passed);
    }
    
    #[test]
    fn test_license_check_struct() {
        assert_eq!(LicenseCheck::name(), "license");
        assert!(!LicenseCheck::description().is_empty());
    }
}

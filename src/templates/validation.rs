//! Template validation for repository templates
//!
//! @task T021
//! @epic T014

use crate::error::{Error, Result};
use crate::templates::manifest::TemplateManifest;
use std::path::Path;

/// Validation result with detailed errors
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// List of validation errors
    pub errors: Vec<String>,
    /// List of validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new empty validation result
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
        self.valid = false;
    }

    /// Add a warning
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a template directory and its manifest
///
/// @task T021
/// @epic T014
pub async fn validate_template(template_dir: &Path, manifest: &TemplateManifest) -> Result<()> {
    let result = validate_template_detailed(template_dir, manifest).await;

    if !result.valid {
        let error_msg = format!("Template validation failed:\n{}", result.errors.join("\n"));
        return Err(Error::template(error_msg));
    }

    // Log warnings
    for warning in &result.warnings {
        tracing::warn!("Template warning: {}", warning);
    }

    Ok(())
}

/// Validate template with detailed results
///
/// @task T021
/// @epic T014
pub async fn validate_template_detailed(
    template_dir: &Path,
    manifest: &TemplateManifest,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Validate manifest fields
    validate_manifest_fields(manifest, &mut result);

    // Validate template files exist
    validate_template_files(template_dir, manifest, &mut result).await;

    // Validate template structure
    validate_template_structure(template_dir, &mut result).await;

    // Validate ferrous-forge compliance
    validate_ferrous_forge_compliance(template_dir, manifest, &mut result).await;

    result
}

/// Validate manifest fields
fn validate_manifest_fields(manifest: &TemplateManifest, result: &mut ValidationResult) {
    // Check name
    if manifest.name.is_empty() {
        result.add_error("Template name cannot be empty");
    }

    // Check version format
    if manifest.version.is_empty() {
        result.add_error("Template version cannot be empty");
    } else if !is_valid_version(&manifest.version) {
        result.add_warning(format!(
            "Version '{}' does not follow semantic versioning",
            manifest.version
        ));
    }

    // Check description
    if manifest.description.is_empty() {
        result.add_warning("Template description is empty");
    }

    // Check author
    if manifest.author.is_empty() {
        result.add_warning("Template author is empty");
    }

    // Check edition
    let valid_editions = ["2015", "2018", "2021", "2024"];
    if !valid_editions.contains(&manifest.edition.as_str()) {
        result.add_warning(format!("Edition '{}' may not be valid", manifest.edition));
    }

    // Validate variable names
    let mut seen_names = std::collections::HashSet::new();
    for var in &manifest.variables {
        if !seen_names.insert(&var.name) {
            result.add_error(format!("Duplicate variable name: {}", var.name));
        }

        // Check for reserved variable names
        if var.name == "project_name" || var.name == "author" {
            result.add_warning(format!(
                "Variable '{}' may conflict with default variables",
                var.name
            ));
        }
    }

    // Validate files
    if manifest.files.is_empty() {
        result.add_error("Template must have at least one file");
    }
}

/// Validate that template files exist
async fn validate_template_files(
    template_dir: &Path,
    manifest: &TemplateManifest,
    result: &mut ValidationResult,
) {
    for file in &manifest.files {
        let source_path = template_dir.join(&file.source);
        if !source_path.exists() {
            result.add_error(format!(
                "Template file not found: {}",
                file.source.display()
            ));
        }
    }

    // Check for required Ferrous Forge files
    let required_files = ["template.toml"];
    for required in &required_files {
        let path = template_dir.join(required);
        if !path.exists() {
            result.add_error(format!("Required file missing: {}", required));
        }
    }
}

/// Validate template structure
async fn validate_template_structure(template_dir: &Path, result: &mut ValidationResult) {
    // Check for Cargo.toml in template (for Rust projects)
    let cargo_toml = template_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        result.add_warning("No Cargo.toml found - template may not be a Rust project");
    } else {
        // Try to parse Cargo.toml
        match tokio::fs::read_to_string(&cargo_toml).await {
            Ok(content) => {
                if let Err(e) = toml::from_str::<toml::Value>(&content) {
                    result.add_error(format!("Invalid Cargo.toml: {}", e));
                }
            }
            Err(e) => {
                result.add_error(format!("Failed to read Cargo.toml: {}", e));
            }
        }
    }

    // Check for src directory
    let src_dir = template_dir.join("src");
    if !src_dir.exists() {
        result.add_warning("No src/ directory found");
    }
}

/// Validate Ferrous Forge compliance
async fn validate_ferrous_forge_compliance(
    template_dir: &Path,
    manifest: &TemplateManifest,
    result: &mut ValidationResult,
) {
    // Check if template includes ferrous-forge configuration
    let forge_config = template_dir.join(".ferrous-forge").join("config.toml");
    if !forge_config.exists() {
        result.add_warning("Template does not include Ferrous Forge configuration");
    }

    // Check for CI configuration
    let ci_dirs = [".github/workflows", ".ci"];
    let has_ci = ci_dirs.iter().any(|dir| template_dir.join(dir).exists());
    if !has_ci {
        result.add_warning("Template does not include CI configuration");
    }

    // Check edition compliance
    if manifest.edition != "2024" {
        result.add_warning(format!(
            "Template uses edition {} instead of recommended 2024",
            manifest.edition
        ));
    }
}

/// Check if version follows semantic versioning format
fn is_valid_version(version: &str) -> bool {
    // Basic semver check: major.minor.patch
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }

    parts.iter().all(|p| p.parse::<u64>().is_ok())
}

/// Validate template before installation
///
/// This is the main entry point for template validation.
/// @task T021
/// @epic T014
pub async fn validate_before_install(template_dir: &Path) -> Result<ValidationResult> {
    // Load manifest
    let manifest_path = template_dir.join("template.toml");
    if !manifest_path.exists() {
        return Err(Error::template(
            "Template manifest not found: template.toml",
        ));
    }

    let manifest_content = tokio::fs::read_to_string(&manifest_path)
        .await
        .map_err(|e| Error::template(format!("Failed to read manifest: {e}")))?;

    let manifest: TemplateManifest = toml::from_str(&manifest_content)
        .map_err(|e| Error::template(format!("Failed to parse manifest: {e}")))?;

    // Run validation
    let result = validate_template_detailed(template_dir, &manifest).await;

    Ok(result)
}

//! Version and changelog consistency validation
//!
//! Enforces Single Source of Truth for version numbers and changelog maintenance.
//! Supports both SemVer and CalVer version formats.

use crate::config::Config;
use crate::validation::{Severity, Violation, ViolationType};
use crate::{Error, Result};
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Version format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionFormat {
    /// Semantic Versioning (e.g., 1.2.3)
    SemVer,
    /// Calendar Versioning (e.g., 2025.03.21 or 2025.3)
    CalVer,
}

/// Changelog validation requirements
#[derive(Debug, Clone)]
pub struct ChangelogRequirements {
    /// Whether to enforce Keep a Changelog format
    pub enforce_keep_a_changelog: bool,
    /// Whether to require changelog entry for current version
    pub require_version_entry: bool,
    /// Whether to validate on git tag creation
    pub check_on_tag: bool,
    /// Required sections in changelog (e.g., ["Added", "Changed", "Fixed"])
    pub required_sections: Vec<String>,
}

impl Default for ChangelogRequirements {
    fn default() -> Self {
        Self {
            enforce_keep_a_changelog: true,
            require_version_entry: true,
            check_on_tag: true,
            required_sections: vec![
                "Added".to_string(),
                "Changed".to_string(),
                "Fixed".to_string(),
            ],
        }
    }
}

/// Validator for version consistency and changelog maintenance
pub struct VersionConsistencyValidator {
    /// Root directory of the project
    project_root: PathBuf,
    /// Version from Cargo.toml (`SSoT`)
    source_version: String,
    /// Detected version format
    version_format: VersionFormat,
    /// Regex to match version strings in code
    version_regex: Regex,
    /// Files/directories to exclude from checking
    exclusions: HashSet<PathBuf>,
    /// Config for validation settings
    config: Config,
    /// Changelog requirements
    changelog_requirements: ChangelogRequirements,
}

/// Result of version validation
#[derive(Debug, Clone)]
pub struct VersionValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// List of violations found
    pub violations: Vec<Violation>,
    /// The source version from Cargo.toml
    pub source_version: String,
    /// Detected version format
    pub version_format: VersionFormat,
    /// Changelog status
    pub changelog_status: ChangelogStatus,
}

/// Status of changelog validation
#[derive(Debug, Clone)]
pub struct ChangelogStatus {
    /// Whether changelog exists
    pub exists: bool,
    /// Whether current version is documented
    pub version_documented: bool,
    /// Whether Keep a Changelog format is followed
    pub follows_keep_a_changelog: bool,
    /// Missing required sections
    pub missing_sections: Vec<String>,
}

impl VersionConsistencyValidator {
    /// Create a new version consistency validator
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cargo.toml cannot be read
    /// - Version cannot be parsed from Cargo.toml
    /// - Regex compilation fails
    pub fn new(project_root: PathBuf, config: Config) -> Result<Self> {
        let source_version = Self::extract_version_from_cargo(&project_root)?;
        let version_format = Self::detect_version_format(&source_version);

        // Build version regex based on format
        let version_pattern = match version_format {
            VersionFormat::SemVer => r"(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?(?:\+[a-zA-Z0-9.-]+)?)",
            VersionFormat::CalVer => r"(\d{4}(?:\.\d{1,2}){1,2}(?:-[a-zA-Z0-9.-]+)?)",
        };

        let version_regex = Regex::new(&format!(
            r#"(?i)(?:version\s*[=:]\s*["']?|const\s+VERSION\s*[=:]\s*["']?|static\s+VERSION\s*[=:]\s*["']?){}"#,
            version_pattern
        )).map_err(|e| Error::validation(format!("Failed to compile version regex: {}", e)))?;

        let mut exclusions = HashSet::new();

        // Default exclusions — files
        exclusions.insert(project_root.join("Cargo.toml"));
        exclusions.insert(project_root.join("Cargo.lock"));
        exclusions.insert(project_root.join("CHANGELOG.md"));
        exclusions.insert(project_root.join("README.md"));
        exclusions.insert(project_root.join("CHANGELOG"));

        // Default exclusions — directories
        exclusions.insert(project_root.join("docs"));
        exclusions.insert(project_root.join(".github"));
        exclusions.insert(project_root.join("packaging"));
        exclusions.insert(project_root.join("target"));
        exclusions.insert(project_root.join("node_modules"));
        exclusions.insert(project_root.join(".git"));
        exclusions.insert(project_root.join(".claude"));
        exclusions.insert(project_root.join("dist"));
        exclusions.insert(project_root.join("build"));
        exclusions.insert(project_root.join(".next"));
        exclusions.insert(project_root.join(".turbo"));
        exclusions.insert(project_root.join("vendor"));
        exclusions.insert(project_root.join("__pycache__"));
        exclusions.insert(project_root.join(".venv"));

        // Add configured exclusions
        if let Some(user_exclusions) = config.validation.version_check_exclusions.as_ref() {
            for exclusion in user_exclusions {
                exclusions.insert(project_root.join(exclusion));
            }
        }

        // Get changelog requirements from config
        let changelog_requirements = ChangelogRequirements {
            enforce_keep_a_changelog: config.validation.enforce_keep_a_changelog.unwrap_or(true),
            require_version_entry: config.validation.require_changelog_entry.unwrap_or(true),
            check_on_tag: config.validation.check_changelog_on_tag.unwrap_or(true),
            required_sections: config
                .validation
                .changelog_required_sections
                .clone()
                .unwrap_or_else(|| {
                    vec![
                        "Added".to_string(),
                        "Changed".to_string(),
                        "Fixed".to_string(),
                    ]
                }),
        };

        Ok(Self {
            project_root,
            source_version,
            version_format,
            version_regex,
            exclusions,
            config,
            changelog_requirements,
        })
    }

    /// Detect version format from version string
    fn detect_version_format(version: &str) -> VersionFormat {
        // CalVer typically starts with 4-digit year
        if Regex::new(r"^\d{4}\.")
            .ok()
            .is_some_and(|re| re.is_match(version))
        {
            VersionFormat::CalVer
        } else {
            VersionFormat::SemVer
        }
    }

    /// Extract version from Cargo.toml (`SSoT`).
    ///
    /// Handles the full set of forms cargo supports:
    /// - `[package].version = "1.2.3"`
    /// - `[package].version.workspace = true` (walks up to the workspace root)
    /// - `[package].version = { workspace = true }` (inline table form)
    /// - Virtual manifests with `[workspace.package].version`
    fn extract_version_from_cargo(project_root: &Path) -> Result<String> {
        let cargo_path = project_root.join("Cargo.toml");
        let content = std::fs::read_to_string(&cargo_path)
            .map_err(|e| Error::io(format!("Failed to read Cargo.toml: {}", e)))?;

        let parsed: toml::Value = toml::from_str(&content)
            .map_err(|e| Error::validation(format!("Failed to parse Cargo.toml: {}", e)))?;

        // 1. [package].version — literal string
        if let Some(version) = parsed.get("package").and_then(|p| p.get("version")) {
            if let Some(s) = version.as_str()
                && Self::is_valid_version(s)
            {
                return Ok(s.to_string());
            }
            // 2. [package].version = { workspace = true } or version.workspace = true
            let is_inherited = version
                .as_table()
                .and_then(|t| t.get("workspace"))
                .and_then(toml::Value::as_bool)
                == Some(true);
            if is_inherited {
                return Self::resolve_workspace_version(project_root);
            }
        }

        // 3. Virtual manifest with [workspace.package].version defined here
        if let Some(s) = parsed
            .get("workspace")
            .and_then(|w| w.get("package"))
            .and_then(|p| p.get("version"))
            .and_then(toml::Value::as_str)
            && Self::is_valid_version(s)
        {
            return Ok(s.to_string());
        }

        Err(Error::validation(
            "Could not parse version from Cargo.toml".to_string(),
        ))
    }

    /// Walk up from `start` looking for a Cargo.toml whose `[workspace.package]`
    /// table defines a concrete `version`. This resolves workspace inheritance
    /// for members that declare `version.workspace = true`.
    fn resolve_workspace_version(start: &Path) -> Result<String> {
        let mut current: Option<&Path> = Some(start);
        while let Some(dir) = current {
            let cargo = dir.join("Cargo.toml");
            if cargo.exists()
                && let Ok(content) = std::fs::read_to_string(&cargo)
                && let Ok(parsed) = toml::from_str::<toml::Value>(&content)
                && let Some(s) = parsed
                    .get("workspace")
                    .and_then(|w| w.get("package"))
                    .and_then(|p| p.get("version"))
                    .and_then(toml::Value::as_str)
                && Self::is_valid_version(s)
            {
                return Ok(s.to_string());
            }
            current = dir.parent();
        }
        Err(Error::validation(
            "Could not resolve workspace-inherited version: no workspace root with [workspace.package].version found".to_string(),
        ))
    }

    /// Check if string is valid version (`SemVer` or `CalVer`)
    fn is_valid_version(version: &str) -> bool {
        // SemVer: x.y.z with optional pre-release and build metadata
        let semver_ok = Regex::new(r"^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?(?:\+[a-zA-Z0-9.-]+)?$")
            .ok()
            .is_some_and(|re| re.is_match(version));

        // CalVer: YYYY.MM.DD or YYYY.M.D or YYYY.MM
        let calver_ok = Regex::new(r"^\d{4}(?:\.\d{1,2}){1,2}(?:-[a-zA-Z0-9.-]+)?$")
            .ok()
            .is_some_and(|re| re.is_match(version));

        semver_ok || calver_ok
    }

    /// Validate version consistency across the codebase
    ///
    /// # Errors
    ///
    /// Returns an error if the project files cannot be read or analyzed.
    pub async fn validate(&self) -> Result<VersionValidationResult> {
        let mut violations = Vec::new();

        // Check if version consistency checking is enabled
        if !self
            .config
            .validation
            .check_version_consistency
            .unwrap_or(true)
        {
            return Ok(self.empty_result());
        }

        // Check for hardcoded versions in code
        self.check_hardcoded_versions(&mut violations).await?;

        // Check changelog
        let changelog_status = self.validate_changelog().await?;

        // Add changelog violations
        if self.changelog_requirements.require_version_entry && !changelog_status.version_documented
        {
            violations.push(Violation {
                violation_type: ViolationType::MissingChangelogEntry,
                file: self.project_root.join("CHANGELOG.md"),
                line: 1,
                message: format!(
                    "Version {} is not documented in CHANGELOG.md. Add entry following Keep a Changelog format.",
                    self.source_version
                ),
                severity: Severity::Error,
            });
        }

        if self.changelog_requirements.enforce_keep_a_changelog
            && !changelog_status.follows_keep_a_changelog
        {
            violations.push(Violation {
                violation_type: ViolationType::InvalidChangelogFormat,
                file: self.project_root.join("CHANGELOG.md"),
                line: 1,
                message: "CHANGELOG.md does not follow Keep a Changelog format. See https://keepachangelog.com/".to_string(),
                severity: Severity::Warning,
            });
        }

        // Check if we're creating a tag (via git hook or CI)
        if self.is_tagging_scenario().await?
            && self.changelog_requirements.check_on_tag
            && !changelog_status.version_documented
        {
            violations.push(Violation {
                    violation_type: ViolationType::MissingChangelogEntry,
                    file: self.project_root.join("CHANGELOG.md"),
                    line: 1,
                    message: format!(
                        "Cannot create tag for version {}: No changelog entry found. Document changes before tagging.",
                        self.source_version
                    ),
                    severity: Severity::Error,
                });
        }

        Ok(VersionValidationResult {
            passed: violations.is_empty(),
            violations,
            source_version: self.source_version.clone(),
            version_format: self.version_format,
            changelog_status,
        })
    }

    /// Check for hardcoded versions in source files
    async fn check_hardcoded_versions(&self, violations: &mut Vec<Violation>) -> Result<()> {
        let root = self.project_root.clone();
        let exclusions = self.exclusions.clone();

        // Collect file paths in a blocking context to avoid stalling the async
        // runtime (WalkDir is synchronous and holds directory handles).
        /// Directory names to skip at any nesting depth.
        const WALK_SKIP_DIRS: &[&str] = &[
            "target",
            "node_modules",
            ".git",
            ".claude",
            ".next",
            "dist",
            "build",
            ".turbo",
            ".pnpm",
            ".yarn",
            "__pycache__",
            ".venv",
            "vendor",
        ];

        let rs_paths: Vec<PathBuf> = tokio::task::spawn_blocking(move || {
            let walker = WalkDir::new(&root).into_iter();
            let mut paths = Vec::new();
            for entry in walker.filter_entry(|e| {
                // Skip non-Rust directories at any depth
                if e.file_type().is_dir()
                    && e.file_name()
                        .to_str()
                        .is_some_and(|name| WALK_SKIP_DIRS.contains(&name))
                {
                    return false;
                }
                true
            }) {
                let Ok(e) = entry else { continue };
                let p = e.path();
                if exclusions.iter().any(|ex| p.starts_with(ex)) {
                    continue;
                }
                let s = p.to_string_lossy();
                if s.contains("/tests/")
                    || s.contains("/test/")
                    || s.contains("/fixtures/")
                    || s.contains("/examples/")
                {
                    continue;
                }
                if p.extension().is_some_and(|ext| ext == "rs") {
                    paths.push(p.to_path_buf());
                }
            }
            paths
        })
        .await
        .map_err(|e| Error::process(format!("Task join error: {}", e)))?;

        for path in rs_paths {
            self.check_file(&path, violations).await?;
        }
        Ok(())
    }

    /// Check a single file for hardcoded versions
    async fn check_file(&self, path: &Path, violations: &mut Vec<Violation>) -> Result<()> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| Error::io(format!("Failed to read {}: {}", path.display(), e)))?;

        for (line_num, line) in content.lines().enumerate() {
            // Skip comments (but not doc comments which might need versions)
            let trimmed = line.trim();
            if trimmed.starts_with("//") && !trimmed.starts_with("///") {
                continue;
            }

            // Check for hardcoded version
            if let Some(captures) = self.version_regex.captures(line)
                && let Some(version_match) = captures.get(1)
            {
                let found_version = version_match.as_str();

                // If version matches Cargo.toml, check if it's properly sourced
                if found_version == self.source_version {
                    // Allow env! macro and CARGO_PKG_VERSION
                    if !line.contains("env!(\"CARGO_PKG_VERSION\")")
                        && !line.contains("CARGO_PKG_VERSION")
                        && !line.contains("clap::crate_version!")
                    {
                        violations.push(Violation {
                                violation_type: ViolationType::HardcodedVersion,
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                message: format!(
                                    "Hardcoded version '{}' found. Use env!(\"CARGO_PKG_VERSION\") or clap::crate_version!() for SSoT.",
                                    found_version
                                ),
                                severity: Severity::Error,
                            });
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate changelog format and content
    async fn validate_changelog(&self) -> Result<ChangelogStatus> {
        let changelog_path = self.project_root.join("CHANGELOG.md");

        if !changelog_path.exists() {
            return Ok(ChangelogStatus {
                exists: false,
                version_documented: false,
                follows_keep_a_changelog: false,
                missing_sections: vec![],
            });
        }

        let content = tokio::fs::read_to_string(&changelog_path)
            .await
            .map_err(|e| Error::io(format!("Failed to read CHANGELOG.md: {}", e)))?;

        let content_lower = content.to_lowercase();

        // Check for Keep a Changelog markers
        let has_keep_a_changelog_format = content.contains("## [Unreleased]")
            || content_lower.contains("all notable changes")
            || content.contains("Keep a Changelog");

        // Check if current version is documented
        let version_documented = content.contains(&format!("[{}]", self.source_version))
            || content.contains(&format!("## {}", self.source_version));

        // Check for required sections
        let mut missing_sections = Vec::new();
        for section in &self.changelog_requirements.required_sections {
            let section_lower = section.to_lowercase();
            if !content_lower.contains(&format!("### {}", section_lower))
                && !content_lower.contains(&format!("## {}", section_lower))
            {
                missing_sections.push(section.clone());
            }
        }

        Ok(ChangelogStatus {
            exists: true,
            version_documented,
            follows_keep_a_changelog: has_keep_a_changelog_format,
            missing_sections,
        })
    }

    /// Check if we're in a tagging scenario (creating a git tag)
    async fn is_tagging_scenario(&self) -> Result<bool> {
        // Check for tag-related environment variables (from CI or hooks)
        if std::env::var("GITHUB_REF_TYPE").unwrap_or_default() == "tag" {
            return Ok(true);
        }

        // Check if HEAD is being tagged
        let output = tokio::process::Command::new("git")
            .args(["describe", "--exact-match", "--tags", "HEAD"])
            .current_dir(&self.project_root)
            .output()
            .await;

        if let Ok(output) = output
            && output.status.success()
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Create empty result for when validation is disabled
    fn empty_result(&self) -> VersionValidationResult {
        VersionValidationResult {
            passed: true,
            violations: vec![],
            source_version: self.source_version.clone(),
            version_format: self.version_format,
            changelog_status: ChangelogStatus {
                exists: false,
                version_documented: false,
                follows_keep_a_changelog: false,
                missing_sections: vec![],
            },
        }
    }

    /// Get the source version
    pub fn source_version(&self) -> &str {
        &self.source_version
    }

    /// Get detected version format
    pub fn version_format(&self) -> VersionFormat {
        self.version_format
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_detects_semver() {
        assert_eq!(
            VersionConsistencyValidator::detect_version_format("1.2.3"),
            VersionFormat::SemVer
        );
        assert_eq!(
            VersionConsistencyValidator::detect_version_format("1.2.3-alpha"),
            VersionFormat::SemVer
        );
    }

    #[tokio::test]
    async fn test_detects_calver() {
        assert_eq!(
            VersionConsistencyValidator::detect_version_format("2025.03.21"),
            VersionFormat::CalVer
        );
        assert_eq!(
            VersionConsistencyValidator::detect_version_format("2025.3"),
            VersionFormat::CalVer
        );
    }

    #[tokio::test]
    async fn test_validates_changelog_format() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create Cargo.toml
        let cargo_toml = r#"
[package]
name = "test-project"
version = "1.2.3"
edition = "2021"
"#;
        fs::write(project_root.join("Cargo.toml"), cargo_toml)
            .await
            .unwrap();

        // Create proper Keep a Changelog format
        let changelog = r#"# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.3] - 2025-03-21

### Added
- New feature X

### Fixed
- Bug Y
"#;
        fs::write(project_root.join("CHANGELOG.md"), changelog)
            .await
            .unwrap();

        let config = Config::default();
        let validator =
            VersionConsistencyValidator::new(project_root.to_path_buf(), config).unwrap();

        let result = validator.validate().await.unwrap();
        assert!(result.changelog_status.follows_keep_a_changelog);
        assert!(result.changelog_status.version_documented);
    }

    #[tokio::test]
    async fn test_extract_version_with_workspace_inheritance_dotted() {
        // Virtual workspace root + a member that uses `version.workspace = true`.
        // Running the validator against the member directory must walk up to the
        // workspace root and resolve the version from [workspace.package].
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(
            root.join("Cargo.toml"),
            r#"
[workspace]
members = ["child"]

[workspace.package]
version = "1.2.3"
edition = "2024"
"#,
        )
        .await
        .unwrap();

        let child = root.join("child");
        fs::create_dir_all(&child).await.unwrap();
        fs::write(
            child.join("Cargo.toml"),
            r#"
[package]
name = "child"
version.workspace = true
edition.workspace = true
"#,
        )
        .await
        .unwrap();

        let version = VersionConsistencyValidator::extract_version_from_cargo(&child).unwrap();
        assert_eq!(version, "1.2.3");
    }

    #[tokio::test]
    async fn test_extract_version_with_workspace_inheritance_inline() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(
            root.join("Cargo.toml"),
            r#"
[workspace]
members = ["child"]

[workspace.package]
version = "2.0.0"
"#,
        )
        .await
        .unwrap();

        let child = root.join("child");
        fs::create_dir_all(&child).await.unwrap();
        fs::write(
            child.join("Cargo.toml"),
            r#"
[package]
name = "child"
version = { workspace = true }
"#,
        )
        .await
        .unwrap();

        let version = VersionConsistencyValidator::extract_version_from_cargo(&child).unwrap();
        assert_eq!(version, "2.0.0");
    }

    #[tokio::test]
    async fn test_extract_version_from_virtual_workspace_root() {
        // A pure virtual workspace (no [package]) should resolve its own
        // [workspace.package].version.
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(
            root.join("Cargo.toml"),
            r#"
[workspace]
members = ["a", "b"]

[workspace.package]
version = "0.9.0"
edition = "2024"
"#,
        )
        .await
        .unwrap();

        let version = VersionConsistencyValidator::extract_version_from_cargo(root).unwrap();
        assert_eq!(version, "0.9.0");
    }
}

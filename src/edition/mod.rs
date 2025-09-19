//! Rust edition detection and migration assistance
//!
//! This module provides functionality to detect the current edition used in
//! a project, check for available migrations, and assist with the migration process.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

pub mod analyzer;
pub mod migrator;

pub use analyzer::EditionAnalyzer;
pub use migrator::EditionMigrator;

/// Rust edition
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Edition {
    /// Rust 2015 edition
    Edition2015,
    /// Rust 2018 edition
    Edition2018,
    /// Rust 2021 edition
    Edition2021,
    /// Rust 2024 edition
    Edition2024,
}

impl Edition {
    /// Parse edition from string
    pub fn parse_edition(s: &str) -> Result<Self> {
        match s {
            "2015" => Ok(Self::Edition2015),
            "2018" => Ok(Self::Edition2018),
            "2021" => Ok(Self::Edition2021),
            "2024" => Ok(Self::Edition2024),
            _ => Err(Error::parse(format!("Unknown edition: {}", s))),
        }
    }

    /// Get the edition as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Edition2015 => "2015",
            Self::Edition2018 => "2018",
            Self::Edition2021 => "2021",
            Self::Edition2024 => "2024",
        }
    }

    /// Get the latest stable edition
    pub fn latest() -> Self {
        Self::Edition2024
    }

    /// Check if this edition is the latest
    pub fn is_latest(&self) -> bool {
        *self == Self::latest()
    }

    /// Get the next edition after this one
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Edition2015 => Some(Self::Edition2018),
            Self::Edition2018 => Some(Self::Edition2021),
            Self::Edition2021 => Some(Self::Edition2024),
            Self::Edition2024 => None,
        }
    }

    /// Get edition-specific lints for migration
    pub fn migration_lints(&self) -> Vec<String> {
        match self {
            Self::Edition2015 => vec!["rust_2018_compatibility".to_string()],
            Self::Edition2018 => vec!["rust_2021_compatibility".to_string()],
            Self::Edition2021 => vec!["rust_2024_compatibility".to_string()],
            Self::Edition2024 => vec![],
        }
    }
}

impl std::fmt::Display for Edition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Edition {}", self.as_str())
    }
}

/// Edition compliance status
#[derive(Debug, Clone)]
pub struct EditionStatus {
    /// Current edition in use
    pub current: Edition,
    /// Latest available edition
    pub latest: Edition,
    /// Is the project using the latest edition?
    pub is_latest: bool,
    /// Path to Cargo.toml
    pub manifest_path: PathBuf,
    /// Recommended migration path
    pub migration_path: Vec<Edition>,
}

impl EditionStatus {
    /// Create a new edition status
    pub fn new(current: Edition, manifest_path: PathBuf) -> Self {
        let latest = Edition::latest();
        let is_latest = current == latest;

        // Build migration path
        let mut migration_path = Vec::new();
        let mut current_edition = current;

        while let Some(next) = current_edition.next() {
            if next <= latest {
                migration_path.push(next);
                current_edition = next;
            } else {
                break;
            }
        }

        Self {
            current,
            latest,
            is_latest,
            manifest_path,
            migration_path,
        }
    }
}

/// Detect edition from Cargo.toml
pub async fn detect_edition(manifest_path: &Path) -> Result<Edition> {
    if !manifest_path.exists() {
        return Err(Error::file_not_found(format!(
            "Cargo.toml not found at {}",
            manifest_path.display()
        )));
    }

    let contents = fs::read_to_string(manifest_path).await?;
    let manifest: toml::Value = toml::from_str(&contents)
        .map_err(|e| Error::parse(format!("Failed to parse Cargo.toml: {}", e)))?;

    // Get edition from [package] section
    let edition_str = manifest
        .get("package")
        .and_then(|p| p.get("edition"))
        .and_then(|e| e.as_str())
        .unwrap_or("2015"); // Default to 2015 if not specified

    Edition::parse_edition(edition_str)
}

/// Check edition compliance for a project
pub async fn check_compliance(project_path: &Path) -> Result<EditionStatus> {
    let manifest_path = project_path.join("Cargo.toml");
    let edition = detect_edition(&manifest_path).await?;

    Ok(EditionStatus::new(edition, manifest_path))
}

/// Get edition migration recommendations
pub fn get_migration_recommendations(status: &EditionStatus) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !status.is_latest {
        recommendations.push(format!(
            "Your project is using {}, but {} is now available",
            status.current, status.latest
        ));

        if !status.migration_path.is_empty() {
            recommendations.push(format!(
                "Recommended migration path: {}",
                status
                    .migration_path
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(" → ")
            ));
        }

        recommendations
            .push("Run `ferrous-forge edition migrate` to start the migration process".to_string());
    } else {
        recommendations.push(format!(
            "✅ Your project is already using the latest edition ({})",
            status.latest
        ));
    }

    recommendations
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_edition_from_str() {
        assert_eq!(
            Edition::parse_edition("2015").unwrap(),
            Edition::Edition2015
        );
        assert_eq!(
            Edition::parse_edition("2018").unwrap(),
            Edition::Edition2018
        );
        assert_eq!(
            Edition::parse_edition("2021").unwrap(),
            Edition::Edition2021
        );
        assert_eq!(
            Edition::parse_edition("2024").unwrap(),
            Edition::Edition2024
        );
        assert!(Edition::parse_edition("2027").is_err());
    }

    #[test]
    fn test_edition_ordering() {
        assert!(Edition::Edition2015 < Edition::Edition2018);
        assert!(Edition::Edition2018 < Edition::Edition2021);
        assert!(Edition::Edition2021 < Edition::Edition2024);
    }

    #[test]
    fn test_edition_next() {
        assert_eq!(Edition::Edition2015.next(), Some(Edition::Edition2018));
        assert_eq!(Edition::Edition2018.next(), Some(Edition::Edition2021));
        assert_eq!(Edition::Edition2021.next(), Some(Edition::Edition2024));
        assert_eq!(Edition::Edition2024.next(), None);
    }

    #[test]
    fn test_migration_path() {
        let status = EditionStatus::new(Edition::Edition2015, PathBuf::from("Cargo.toml"));
        assert_eq!(status.migration_path.len(), 3);
        assert_eq!(status.migration_path[0], Edition::Edition2018);
        assert_eq!(status.migration_path[1], Edition::Edition2021);
        assert_eq!(status.migration_path[2], Edition::Edition2024);
    }

    #[tokio::test]
    async fn test_detect_edition() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("Cargo.toml");

        let manifest_content = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;

        fs::write(&manifest_path, manifest_content).await.unwrap();

        let edition = detect_edition(&manifest_path).await.unwrap();
        assert_eq!(edition, Edition::Edition2021);
    }
}

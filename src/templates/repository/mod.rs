//! Template Repository System - Community template sharing and management
//!
//! This module provides functionality for:
//! - Fetching templates from GitHub repositories
//! - Validating template structure before installation
//! - Caching templates locally in ~/.config/ferrous-forge/templates/
//! - Template versioning and updates
//!
//! @task T021
//! @epic T014

use crate::error::{Error, Result};
use crate::templates::manifest::TemplateManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// GitHub API client for fetching templates
pub mod github;

/// Template metadata stored in cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTemplate {
    /// Template name
    pub name: String,
    /// Source repository (e.g., "gh:user/repo")
    pub source: String,
    /// Template version
    pub version: String,
    /// When the template was fetched
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    /// When the template was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Path to cached template directory
    pub cache_path: PathBuf,
    /// Template manifest
    pub manifest: TemplateManifest,
}

/// Template repository manager
pub struct TemplateRepository {
    /// Cache directory for templates
    cache_dir: PathBuf,
    /// Index of cached templates
    index: TemplateIndex,
}

/// Index of cached templates
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateIndex {
    /// Map of template name to cached template info
    pub templates: HashMap<String, CachedTemplate>,
    /// Index version for migrations
    pub version: u32,
}

impl TemplateRepository {
    /// Create a new template repository manager
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be created or accessed.
    pub fn new() -> Result<Self> {
        let cache_dir = Self::cache_dir()?;
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            Error::template(format!("Failed to create template cache directory: {e}"))
        })?;

        let index = Self::load_index(&cache_dir)?;

        Ok(Self { cache_dir, index })
    }

    /// Get the cache directory path
    fn cache_dir() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| Error::config("Could not find config directory"))?;
        Ok(config_dir.join("ferrous-forge").join("templates"))
    }

    /// Load the template index from disk
    fn load_index(cache_dir: &Path) -> Result<TemplateIndex> {
        let index_path = cache_dir.join("index.json");
        if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)
                .map_err(|e| Error::template(format!("Failed to read template index: {e}")))?;
            let index: TemplateIndex = serde_json::from_str(&content)
                .map_err(|e| Error::template(format!("Failed to parse template index: {e}")))?;
            Ok(index)
        } else {
            Ok(TemplateIndex {
                version: 1,
                templates: HashMap::new(),
            })
        }
    }

    /// Save the template index to disk
    fn save_index(&self) -> Result<()> {
        let index_path = self.cache_dir.join("index.json");
        let content = serde_json::to_string_pretty(&self.index)
            .map_err(|e| Error::template(format!("Failed to serialize template index: {e}")))?;
        std::fs::write(&index_path, content)
            .map_err(|e| Error::template(format!("Failed to write template index: {e}")))?;
        Ok(())
    }

    /// List all cached templates
    pub fn list_cached(&self) -> Vec<&CachedTemplate> {
        self.index.templates.values().collect()
    }

    /// Get a cached template by name
    pub fn get_cached(&self, name: &str) -> Option<&CachedTemplate> {
        self.index.templates.get(name)
    }

    /// Check if a template is cached
    pub fn is_cached(&self, name: &str) -> bool {
        self.index.templates.contains_key(name)
    }

    /// Add a template to the cache
    pub fn add_to_cache(&mut self, template: CachedTemplate) -> Result<()> {
        self.index.templates.insert(template.name.clone(), template);
        self.save_index()
    }

    /// Remove a template from cache
    pub fn remove_from_cache(&mut self, name: &str) -> Result<()> {
        if let Some(template) = self.index.templates.remove(name) {
            // Remove the cached directory
            if template.cache_path.exists() {
                std::fs::remove_dir_all(&template.cache_path).map_err(|e| {
                    Error::template(format!("Failed to remove cached template: {e}"))
                })?;
            }
        }
        self.save_index()
    }

    /// Get cache directory path
    pub fn cache_directory(&self) -> &Path {
        &self.cache_dir
    }

    /// Get the path where a template should be cached
    pub fn template_cache_path(&self, name: &str) -> PathBuf {
        self.cache_dir.join(name)
    }
}

impl CachedTemplate {
    /// Check if the template needs an update (older than 24 hours)
    pub fn needs_update(&self) -> bool {
        let age = chrono::Utc::now() - self.updated_at;
        age.num_hours() >= 24
    }
}

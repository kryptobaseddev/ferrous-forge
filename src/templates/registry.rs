//! Template registry for managing available templates

use super::builtin::{create_cli_template, create_library_template, create_web_service_template};
use super::manifest::{TemplateKind, TemplateManifest};
use crate::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Registry of available templates
pub struct TemplateRegistry {
    /// Built-in templates
    builtin: HashMap<String, BuiltinTemplate>,

    /// Custom templates from filesystem
    custom: HashMap<String, PathBuf>,
}

/// Built-in template definition
pub struct BuiltinTemplate {
    /// Template manifest
    pub manifest: TemplateManifest,

    /// Template files as strings
    pub files: HashMap<String, String>,
}

impl TemplateRegistry {
    /// Create a new registry with built-in templates
    pub fn new() -> Self {
        let mut registry = Self {
            builtin: HashMap::new(),
            custom: HashMap::new(),
        };

        // Register built-in templates
        registry.builtin.insert("cli-app".to_string(), create_cli_template());
        registry.builtin.insert("library".to_string(), create_library_template());
        registry.builtin.insert("web-service".to_string(), create_web_service_template());

        registry
    }




    /// List all available templates
    pub fn list_templates(&self) -> Vec<(&str, TemplateKind, &str)> {
        let mut templates = Vec::new();

        for (name, template) in &self.builtin {
            templates.push((
                name.as_str(),
                template.manifest.kind,
                template.manifest.description.as_str(),
            ));
        }

        templates.sort_by_key(|t| t.0);
        templates
    }

    /// Get a built-in template
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinTemplate> {
        self.builtin.get(name)
    }

    /// Register a custom template
    pub fn register_custom(&mut self, name: String, path: PathBuf) -> Result<()> {
        if self.builtin.contains_key(&name) {
            return Err(Error::validation(format!(
                "Cannot override built-in template: {}",
                name
            )));
        }

        if !path.exists() {
            return Err(Error::validation(format!(
                "Template path does not exist: {}",
                path.display()
            )));
        }

        self.custom.insert(name, path);
        Ok(())
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

//! Template system for Ferrous Forge
//!
//! This module manages project templates and configuration files that get
//! injected into new Rust projects to enforce standards.

use crate::{Result, Error};
use handlebars::Handlebars;
use serde_json::json;
use std::path::Path;

/// Template manager for Ferrous Forge
pub struct TemplateManager {
    handlebars: Handlebars<'static>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        
        // Register built-in templates
        handlebars.register_template_string("cargo_toml", include_str!("../templates/Cargo.toml.hbs"))
            .map_err(|e| Error::template(format!("Failed to register Cargo.toml template: {}", e)))?;
            
        handlebars.register_template_string("main_rs", include_str!("../templates/main.rs.hbs"))
            .map_err(|e| Error::template(format!("Failed to register main.rs template: {}", e)))?;
            
        handlebars.register_template_string("lib_rs", include_str!("../templates/lib.rs.hbs"))
            .map_err(|e| Error::template(format!("Failed to register lib.rs template: {}", e)))?;
            
        handlebars.register_template_string("github_workflow", include_str!("../templates/ci.yml.hbs"))
            .map_err(|e| Error::template(format!("Failed to register GitHub workflow template: {}", e)))?;
            
        Ok(Self { handlebars })
    }

    /// Generate a Cargo.toml file with Ferrous Forge standards
    pub fn generate_cargo_toml(&self, project_name: &str, is_lib: bool) -> Result<String> {
        let data = json!({
            "project_name": project_name,
            "is_lib": is_lib,
            "version": "0.1.0",
            "edition": "2024",
            "rust_version": "1.85",
        });

        self.handlebars.render("cargo_toml", &data)
            .map_err(|e| Error::template(format!("Failed to render Cargo.toml: {}", e)))
    }

    /// Generate a main.rs file with Ferrous Forge standards
    pub fn generate_main_rs(&self, project_name: &str) -> Result<String> {
        let data = json!({
            "project_name": project_name,
        });

        self.handlebars.render("main_rs", &data)
            .map_err(|e| Error::template(format!("Failed to render main.rs: {}", e)))
    }

    /// Generate a lib.rs file with Ferrous Forge standards
    pub fn generate_lib_rs(&self, project_name: &str) -> Result<String> {
        let data = json!({
            "project_name": project_name,
        });

        self.handlebars.render("lib_rs", &data)
            .map_err(|e| Error::template(format!("Failed to render lib.rs: {}", e)))
    }

    /// Generate GitHub Actions CI workflow
    pub fn generate_github_workflow(&self, project_name: &str) -> Result<String> {
        let data = json!({
            "project_name": project_name,
        });

        self.handlebars.render("github_workflow", &data)
            .map_err(|e| Error::template(format!("Failed to render GitHub workflow: {}", e)))
    }

    /// Apply templates to an existing project
    pub async fn apply_to_project(&self, project_path: &Path) -> Result<()> {
        // This would be used to retrofit existing projects with Ferrous Forge standards
        // For now, just create a placeholder
        tracing::info!("Applying Ferrous Forge templates to project: {}", project_path.display());
        
        // TODO: Implement project retrofitting
        // - Backup existing files
        // - Update Cargo.toml with Edition 2024
        // - Add strict clippy configuration
        // - Add GitHub Actions if .git exists
        // - Update main.rs/lib.rs with documentation requirements
        
        Ok(())
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default template manager")
    }
}

/// Information about a project template
#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    /// Name of the template
    pub name: String,
    /// Description of what this template provides
    pub description: String,
    /// Files that this template will create/modify
    pub files: Vec<String>,
}

/// Get available project templates
pub fn available_templates() -> Vec<ProjectTemplate> {
    vec![
        ProjectTemplate {
            name: "basic".to_string(),
            description: "Basic Rust project with Ferrous Forge standards".to_string(),
            files: vec![
                "Cargo.toml".to_string(),
                "src/main.rs".to_string(),
                ".github/workflows/ci.yml".to_string(),
            ],
        },
        ProjectTemplate {
            name: "library".to_string(),
            description: "Rust library project with Ferrous Forge standards".to_string(),
            files: vec![
                "Cargo.toml".to_string(),
                "src/lib.rs".to_string(),
                ".github/workflows/ci.yml".to_string(),
            ],
        },
    ]
}
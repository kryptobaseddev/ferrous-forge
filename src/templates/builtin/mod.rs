//! Built-in template definitions

pub mod cli_app;
pub mod library;
pub mod web_service;

use super::manifest::{TemplateKind, TemplateManifest};
use crate::Result;
use std::collections::HashMap;

/// Built-in template definition
pub struct BuiltinTemplate {
    /// Template manifest
    pub manifest: TemplateManifest,

    /// Template files as strings
    pub files: HashMap<String, String>,
}

/// Get all built-in templates
pub fn get_builtin_templates() -> Result<HashMap<String, BuiltinTemplate>> {
    let mut templates = HashMap::new();

    templates.insert("cli-app".to_string(), cli_app::create_template()?);
    templates.insert("library".to_string(), library::create_template()?);
    templates.insert("web-service".to_string(), web_service::create_template()?);

    Ok(templates)
}

/// List all built-in template info
pub fn list_builtin_templates() -> Vec<(&'static str, TemplateKind, &'static str)> {
    vec![
        (
            "cli-app",
            TemplateKind::CliApp,
            "Command-line application with clap and tokio",
        ),
        (
            "library",
            TemplateKind::Library,
            "Library crate with comprehensive testing",
        ),
        (
            "web-service",
            TemplateKind::WebService,
            "Web service with axum and tokio",
        ),
    ]
}

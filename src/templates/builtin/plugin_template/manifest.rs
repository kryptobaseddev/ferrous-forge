//! Plugin template manifest creation

use crate::templates::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use std::path::PathBuf;

/// Create the manifest for plugin template
pub fn create_plugin_manifest() -> TemplateManifest {
    let mut manifest = TemplateManifest::new("plugin".to_string(), TemplateKind::Custom);

    manifest.description = "Plugin system with dynamic loading support".to_string();
    manifest.author = "Ferrous Forge Team".to_string();

    add_plugin_variables(&mut manifest);
    add_plugin_files(&mut manifest);
    add_plugin_commands(&mut manifest);

    manifest
}

/// Add variables to the plugin manifest
fn add_plugin_variables(manifest: &mut TemplateManifest) {
    manifest.add_variable(TemplateVariable::required(
        "project_name".to_string(),
        "Name of the plugin project".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "author".to_string(),
        "Author name".to_string(),
        "Unknown".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "plugin_api_version".to_string(),
        "Plugin API version".to_string(),
        "1.0".to_string(),
    ));
}

/// Add files to the plugin manifest
fn add_plugin_files(manifest: &mut TemplateManifest) {
    // Core files
    manifest.add_file(TemplateFile::new(
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/lib.rs"),
    ));

    // Plugin API
    manifest.add_file(TemplateFile::new(
        PathBuf::from("src/api.rs"),
        PathBuf::from("src/api.rs"),
    ));

    // Example host application
    manifest.add_file(TemplateFile::new(
        PathBuf::from("examples/host.rs"),
        PathBuf::from("examples/host.rs"),
    ));

    // Example plugin
    manifest.add_file(TemplateFile::new(
        PathBuf::from("examples/example_plugin.rs"),
        PathBuf::from("examples/example_plugin.rs"),
    ));

    // Configuration
    manifest.add_file(TemplateFile::new(
        PathBuf::from(".ferrous-forge/config.toml"),
        PathBuf::from(".ferrous-forge/config.toml"),
    ));
}

/// Add post-generate commands to the plugin manifest
fn add_plugin_commands(manifest: &mut TemplateManifest) {
    manifest.add_post_generate("cargo fmt".to_string());
    manifest.add_post_generate("cargo check".to_string());
}

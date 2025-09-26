//! Workspace template manifest creation

use crate::templates::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use std::path::PathBuf;

/// Create the manifest for workspace template
pub fn create_workspace_manifest() -> TemplateManifest {
    let mut manifest = TemplateManifest::new("workspace".to_string(), TemplateKind::Workspace);

    manifest.description = "Multi-crate workspace with shared dependencies".to_string();
    manifest.author = "Ferrous Forge Team".to_string();

    add_workspace_variables(&mut manifest);
    add_workspace_files(&mut manifest);
    add_workspace_commands(&mut manifest);

    manifest
}

/// Add variables to the workspace manifest
fn add_workspace_variables(manifest: &mut TemplateManifest) {
    manifest.add_variable(TemplateVariable::required(
        "workspace_name".to_string(),
        "Name of the workspace".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "author".to_string(),
        "Author name".to_string(),
        "Unknown".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "description".to_string(),
        "Workspace description".to_string(),
        "A Rust workspace project".to_string(),
    ));
}

/// Add files to the workspace manifest
fn add_workspace_files(manifest: &mut TemplateManifest) {
    // Workspace root files
    manifest.add_file(TemplateFile::new(
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("README.md"),
        PathBuf::from("README.md"),
    ));

    // Core library crate
    add_core_files(manifest);

    // CLI binary crate
    add_cli_files(manifest);

    // Utils library crate
    add_utils_files(manifest);

    // Configuration
    manifest.add_file(TemplateFile::new(
        PathBuf::from(".ferrous-forge/config.toml"),
        PathBuf::from(".ferrous-forge/config.toml"),
    ));
}

/// Add core crate files to manifest
fn add_core_files(manifest: &mut TemplateManifest) {
    manifest.add_file(TemplateFile::new(
        PathBuf::from("core/Cargo.toml"),
        PathBuf::from("core/Cargo.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("core/src/lib.rs"),
        PathBuf::from("core/src/lib.rs"),
    ));
}

/// Add CLI crate files to manifest
fn add_cli_files(manifest: &mut TemplateManifest) {
    manifest.add_file(TemplateFile::new(
        PathBuf::from("cli/Cargo.toml"),
        PathBuf::from("cli/Cargo.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("cli/src/main.rs"),
        PathBuf::from("cli/src/main.rs"),
    ));
}

/// Add utils crate files to manifest
fn add_utils_files(manifest: &mut TemplateManifest) {
    manifest.add_file(TemplateFile::new(
        PathBuf::from("utils/Cargo.toml"),
        PathBuf::from("utils/Cargo.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("utils/src/lib.rs"),
        PathBuf::from("utils/src/lib.rs"),
    ));
}

/// Add post-generate commands to the workspace manifest
fn add_workspace_commands(manifest: &mut TemplateManifest) {
    manifest.add_post_generate("cargo fmt --all".to_string());
    manifest.add_post_generate("cargo test --workspace".to_string());
}

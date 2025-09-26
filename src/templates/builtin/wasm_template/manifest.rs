//! WASM template manifest creation

use crate::templates::{TemplateFile, TemplateKind, TemplateManifest, TemplateVariable};
use std::path::PathBuf;

/// Create the manifest for WASM template
pub fn create_wasm_manifest() -> TemplateManifest {
    let mut manifest = TemplateManifest::new("wasm".to_string(), TemplateKind::Custom);

    manifest.description = "WebAssembly project with wasm-bindgen".to_string();
    manifest.author = "Ferrous Forge Team".to_string();

    add_wasm_variables(&mut manifest);
    add_wasm_files(&mut manifest);
    add_wasm_commands(&mut manifest);

    manifest
}

/// Add variables to the WASM manifest
fn add_wasm_variables(manifest: &mut TemplateManifest) {
    manifest.add_variable(TemplateVariable::required(
        "project_name".to_string(),
        "Name of the WASM project".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "author".to_string(),
        "Author name".to_string(),
        "Unknown".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "description".to_string(),
        "Project description".to_string(),
        "A WebAssembly project".to_string(),
    ));
}

/// Add files to the WASM manifest
fn add_wasm_files(manifest: &mut TemplateManifest) {
    // Core Rust files
    manifest.add_file(TemplateFile::new(
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/lib.rs"),
    ));

    // Web files
    manifest.add_file(TemplateFile::new(
        PathBuf::from("www/index.html"),
        PathBuf::from("www/index.html"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("www/index.js"),
        PathBuf::from("www/index.js"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("www/package.json"),
        PathBuf::from("www/package.json"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("www/webpack.config.js"),
        PathBuf::from("www/webpack.config.js"),
    ));

    // Configuration
    manifest.add_file(TemplateFile::new(
        PathBuf::from(".ferrous-forge/config.toml"),
        PathBuf::from(".ferrous-forge/config.toml"),
    ));
}

/// Add post-generate commands to the WASM manifest
fn add_wasm_commands(manifest: &mut TemplateManifest) {
    manifest.add_post_generate("cargo fmt".to_string());
    manifest.add_post_generate("wasm-pack build".to_string());
}

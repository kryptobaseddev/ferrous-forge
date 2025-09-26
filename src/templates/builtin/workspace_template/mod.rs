//! Workspace template for multi-crate Rust projects

mod content;
mod manifest;
mod templates;

use crate::templates::BuiltinTemplate;
use content::create_workspace_files;
use manifest::create_workspace_manifest;

/// Create the workspace template
pub fn create_workspace_template() -> BuiltinTemplate {
    let manifest = create_workspace_manifest();
    let files = create_workspace_files();

    BuiltinTemplate { manifest, files }
}

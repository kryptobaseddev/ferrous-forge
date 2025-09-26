//! Plugin template for extensible Rust applications

mod content;
mod manifest;
mod templates;

use crate::templates::BuiltinTemplate;
use content::create_plugin_files;
use manifest::create_plugin_manifest;

/// Create the plugin template
pub fn create_plugin_template() -> BuiltinTemplate {
    let manifest = create_plugin_manifest();
    let files = create_plugin_files();

    BuiltinTemplate { manifest, files }
}

//! WASM template for WebAssembly Rust projects

mod content;
mod manifest;
mod templates;

use crate::templates::BuiltinTemplate;
use content::create_wasm_files;
use manifest::create_wasm_manifest;

/// Create the WASM template
pub fn create_wasm_template() -> BuiltinTemplate {
    let manifest = create_wasm_manifest();
    let files = create_wasm_files();

    BuiltinTemplate { manifest, files }
}

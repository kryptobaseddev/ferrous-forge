//! Built-in template definitions

/// Command-line application project template.
pub mod cli_template;
/// Embedded systems (no_std) project template.
pub mod embedded_template;
/// Reusable library crate project template.
pub mod library_template;
/// Plugin system project template.
pub mod plugin_template;
/// WebAssembly project template.
pub mod wasm_template;
/// Web service (HTTP API) project template.
pub mod web_service_template;
/// Cargo workspace multi-crate project template.
pub mod workspace_template;

pub use cli_template::create_cli_template;
pub use embedded_template::create_embedded_template;
pub use library_template::create_library_template;
pub use plugin_template::create_plugin_template;
pub use wasm_template::create_wasm_template;
pub use web_service_template::create_web_service_template;
pub use workspace_template::create_workspace_template;

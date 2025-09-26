//! Built-in template definitions

pub mod cli_template;
pub mod embedded_template;
pub mod library_template;
pub mod plugin_template;
pub mod wasm_template;
pub mod web_service_template;
pub mod workspace_template;

pub use cli_template::create_cli_template;
pub use embedded_template::create_embedded_template;
pub use library_template::create_library_template;
pub use plugin_template::create_plugin_template;
pub use wasm_template::create_wasm_template;
pub use web_service_template::create_web_service_template;
pub use workspace_template::create_workspace_template;

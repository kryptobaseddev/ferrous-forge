//! Built-in template definitions

pub mod cli_template;
pub mod library_template;
pub mod web_service_template;

pub use cli_template::create_cli_template;
pub use library_template::create_library_template;
pub use web_service_template::create_web_service_template;

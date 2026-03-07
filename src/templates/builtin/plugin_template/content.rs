//! Plugin template file content generation

use super::templates::{api_rs_content, lib_rs_content, plugin_example_content};
use std::collections::HashMap;

/// Create the files for plugin template
pub fn create_plugin_files() -> HashMap<String, String> {
    let mut files = HashMap::new();

    files.insert("Cargo.toml".to_string(), cargo_toml_content());
    files.insert("src/lib.rs".to_string(), lib_rs_content());
    files.insert("src/api.rs".to_string(), api_rs_content());
    files.insert("examples/host.rs".to_string(), host_example_content());
    files.insert(
        "examples/example_plugin.rs".to_string(),
        plugin_example_content(),
    );
    files.insert(
        ".ferrous-forge/config.toml".to_string(),
        config_toml_content(),
    );

    files
}

/// Cargo.toml content for plugin project
fn cargo_toml_content() -> String {
    r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"
authors = ["{{author}}"]
description = "A plugin system with dynamic loading"
license = "MIT OR Apache-2.0"
repository = "https://github.com/{{author}}/{{project_name}}"

[dependencies]
anyhow = "1.0"
thiserror = "1.0"
libloading = "0.8"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
tempfile = "3.10"

[lib]
name = "{{project_name}}"
crate-type = ["cdylib", "rlib"]

[[example]]
name = "host"
path = "examples/host.rs"

[[example]]
name = "example_plugin"
path = "examples/example_plugin.rs"
crate-type = ["cdylib"]
"#
    .to_string()
}

/// examples/host.rs content for example host application
fn host_example_content() -> String {
    r#"//! Example host application that loads and uses plugins

use anyhow::Result;
use {{project_name}}::{DefaultPluginManager, PluginManager};

fn main() -> Result<()> {
    println!("{{project_name}} Host Application");
    println!("================================");

    let mut manager = DefaultPluginManager::new();

    // In a real application, you would load plugins from a directory
    println!("Plugin system initialized");
    println!("Available plugins: {}", manager.list_plugins().len());

    // Example of how to use plugins once loaded
    if let Some(plugin) = manager.get_plugin("example") {
        let result = plugin.execute("test input")?;
        println!("Plugin result: {}", result);
    } else {
        println!("No plugins loaded. Build and load plugins to see them in action.");
        println!("Example: cargo build --example example_plugin");
    }

    Ok(())
}
"#
    .to_string()
}

/// .ferrous-forge/config.toml content
fn config_toml_content() -> String {
    r#"# Ferrous Forge configuration for plugin projects

[validation]
max_line_length = 100
max_file_length = 300
max_function_length = 50
allow_unwrap = false
allow_expect = false

[plugin]
# Plugin-specific configuration
allow_cdylib = true
allow_unsafe_in_examples = true
"#
    .to_string()
}

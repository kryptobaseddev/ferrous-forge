//! Plugin template content strings

/// src/lib.rs content for plugin system
pub fn lib_rs_content() -> String {
    r#"//! {{project_name}} - Plugin system with dynamic loading
#![deny(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

pub mod api;

pub use api::{Plugin, PluginApi, PluginManager, PluginError, PluginInfo};

use anyhow::Result;
use libloading::Library;
use std::collections::HashMap;
use std::path::Path;

/// Plugin manager for loading and managing plugins
pub struct DefaultPluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    libraries: HashMap<String, Library>,
}

impl DefaultPluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            libraries: HashMap::new(),
        }
    }
}

impl PluginManager for DefaultPluginManager {
    fn load_plugin(&mut self, path: &Path) -> Result<PluginInfo, PluginError> {
        let lib_path = path.to_string_lossy();
        
        // Note: Dynamic loading requires unsafe code in production
        // For a safe alternative, consider compile-time plugin registration
        // This is a simplified example for demonstration
        Err(PluginError::LoadError(format!(
            "Dynamic loading not implemented in safe mode: {}",
            lib_path
        )))
    }

    fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(|p| p.info()).collect()
    }

    fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        self.plugins.remove(name);
        self.libraries.remove(name);
        Ok(())
    }
}

impl Default for DefaultPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = DefaultPluginManager::new();
        assert!(manager.list_plugins().is_empty());
    }
}
"#
    .to_string()
}

/// src/api.rs content for plugin API definitions
pub fn api_rs_content() -> String {
    r#"//! Plugin API definitions and traits

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Plugin API version
pub const PLUGIN_API_VERSION: &str = "{{plugin_api_version}}";

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> PluginInfo;

    /// Initialize the plugin
    fn initialize(&mut self) -> Result<()>;

    /// Execute plugin functionality
    fn execute(&self, input: &str) -> Result<String>;

    /// Cleanup plugin resources
    fn cleanup(&mut self) -> Result<()>;
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// API version compatibility
    pub api_version: String,
}

/// Plugin manager trait for managing loaded plugins
pub trait PluginManager {
    /// Load a plugin from a file path
    fn load_plugin(&mut self, path: &Path) -> Result<PluginInfo, PluginError>;

    /// Get a reference to a loaded plugin
    fn get_plugin(&self, name: &str) -> Option<&dyn Plugin>;

    /// List all loaded plugins
    fn list_plugins(&self) -> Vec<PluginInfo>;

    /// Unload a plugin
    fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError>;
}

/// Plugin API trait for plugin discovery
pub trait PluginApi {
    /// Get the API version
    fn api_version(&self) -> &'static str {
        PLUGIN_API_VERSION
    }

    /// Check if a plugin is compatible with this API version
    fn is_compatible(&self, plugin_info: &PluginInfo) -> bool {
        plugin_info.api_version == PLUGIN_API_VERSION
    }
}

/// Plugin-specific errors
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin loading error
    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    /// Symbol resolution error
    #[error("Failed to find plugin symbol: {0}")]
    SymbolError(String),

    /// Plugin initialization error
    #[error("Plugin initialization failed: {0}")]
    InitError(String),

    /// Plugin execution error
    #[error("Plugin execution failed: {0}")]
    ExecutionError(String),

    /// Plugin not found error
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// API version mismatch
    #[error("API version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
}
"#
    .to_string()
}

/// `examples/example_plugin.rs` content for example plugin
pub fn plugin_example_content() -> String {
    r#"//! Example plugin implementation

use anyhow::Result;
use {{project_name}}::{Plugin, PluginInfo, PLUGIN_API_VERSION};

/// Example plugin implementation
pub struct ExamplePlugin {
    initialized: bool,
}

impl ExamplePlugin {
    /// Create a new example plugin instance
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Plugin for ExamplePlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "example".to_string(),
            version: "1.0.0".to_string(),
            description: "Example plugin for demonstration".to_string(),
            api_version: PLUGIN_API_VERSION.to_string(),
        }
    }

    fn initialize(&mut self) -> Result<()> {
        println!("Example plugin initializing...");
        self.initialized = true;
        Ok(())
    }

    fn execute(&self, input: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Plugin not initialized"));
        }

        Ok(format!("Example plugin processed: {}", input))
    }

    fn cleanup(&mut self) -> Result<()> {
        println!("Example plugin cleaning up...");
        self.initialized = false;
        Ok(())
    }
}

/// Plugin entry point - this is the function the host calls to create the plugin
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(ExamplePlugin::new())
}
"#
    .to_string()
}

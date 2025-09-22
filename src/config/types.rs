//! Configuration type definitions

use serde::{Deserialize, Serialize};

/// Ferrous Forge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Whether Ferrous Forge has been initialized
    pub initialized: bool,
    /// Version of the configuration format
    pub version: String,
    /// Update channel (stable, beta, nightly)
    pub update_channel: String,
    /// Whether to automatically check for updates
    pub auto_update: bool,
    /// Custom clippy rules
    pub clippy_rules: Vec<String>,
    /// File size limit in lines
    pub max_file_lines: usize,
    /// Function size limit in lines
    pub max_function_lines: usize,
    /// Whether to enforce Edition 2024
    pub enforce_edition_2024: bool,
    /// Whether to ban underscore bandaid patterns
    pub ban_underscore_bandaid: bool,
    /// Whether to require documentation
    pub require_documentation: bool,
    /// Custom validation rules
    pub custom_rules: Vec<CustomRule>,
}

/// Custom validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Name of the rule
    pub name: String,
    /// Pattern to match (regex)
    pub pattern: String,
    /// Error message to display
    pub message: String,
    /// Whether this rule is enabled
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initialized: false,
            version: "1.3.0".to_string(),
            update_channel: "stable".to_string(),
            auto_update: true,
            clippy_rules: vec![
                "--warn=clippy::all".to_string(),
                "--warn=clippy::pedantic".to_string(),
                "--warn=clippy::nursery".to_string(),
                "--warn=clippy::unwrap_used".to_string(),
                "--warn=clippy::expect_used".to_string(),
            ],
            max_file_lines: 300,
            max_function_lines: 50,
            enforce_edition_2024: true,
            ban_underscore_bandaid: true,
            require_documentation: true,
            custom_rules: vec![],
        }
    }
}
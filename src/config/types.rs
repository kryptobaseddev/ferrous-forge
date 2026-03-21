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
    /// Required Rust edition (e.g. "2024") — locked; LLM agents must not change this
    pub required_edition: String,
    /// Required minimum rust-version (e.g. "1.85.0") — locked; LLM agents must not change this
    pub required_rust_version: String,
    /// Whether to ban underscore bandaid patterns
    pub ban_underscore_bandaid: bool,
    /// Whether to require documentation
    pub require_documentation: bool,
    /// Custom validation rules
    pub custom_rules: Vec<CustomRule>,
    /// Validation settings
    pub validation: ValidationConfig,
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

/// Validation configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether to check for version consistency (SSoT)
    pub check_version_consistency: Option<bool>,
    /// Paths to exclude from version checking
    pub version_check_exclusions: Option<Vec<String>>,
    /// Whether to enforce Keep a Changelog format
    pub enforce_keep_a_changelog: Option<bool>,
    /// Whether to require changelog entry for current version
    pub require_changelog_entry: Option<bool>,
    /// Whether to validate changelog when creating git tags
    pub check_changelog_on_tag: Option<bool>,
    /// Required sections in changelog (e.g., ["Added", "Changed", "Fixed"])
    pub changelog_required_sections: Option<Vec<String>>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_version_consistency: Some(true),
            version_check_exclusions: None,
            enforce_keep_a_changelog: Some(true),
            require_changelog_entry: Some(true),
            check_changelog_on_tag: Some(true),
            changelog_required_sections: Some(vec![
                "Added".to_string(),
                "Changed".to_string(),
                "Fixed".to_string(),
            ]),
        }
    }
}

/// # Examples
///
/// ```rust
/// # use ferrous_forge::Config;
/// let config = Config::default();
/// assert_eq!(config.required_edition, "2024");
/// ```
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
            required_edition: "2024".to_string(),
            required_rust_version: "1.85.0".to_string(),
            ban_underscore_bandaid: true,
            require_documentation: true,
            custom_rules: vec![],
            validation: ValidationConfig::default(),
        }
    }
}

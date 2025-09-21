//! Template manifest and metadata types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Template manifest describing a project template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    /// Name of the template
    pub name: String,

    /// Version of the template
    pub version: String,

    /// Description of what the template creates
    pub description: String,

    /// Author of the template
    pub author: String,

    /// Kind of project this template creates
    pub kind: TemplateKind,

    /// Variables that can be substituted in the template
    pub variables: Vec<TemplateVariable>,

    /// Files included in the template
    pub files: Vec<TemplateFile>,

    /// Post-generation commands to run
    pub post_generate: Vec<String>,

    /// Minimum Rust version required
    pub min_rust_version: Option<String>,

    /// Default Rust edition to use
    pub edition: String,
}

/// Kind of project template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateKind {
    /// Command-line application
    CliApp,
    /// Library crate
    Library,
    /// Web service
    WebService,
    /// Embedded application
    Embedded,
    /// Workspace with multiple crates
    Workspace,
    /// Custom template
    Custom,
}

impl TemplateKind {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::CliApp => "Command-line application with clap and tokio",
            Self::Library => "Library crate with comprehensive testing",
            Self::WebService => "Web service with async runtime",
            Self::Embedded => "Embedded application with no_std support",
            Self::Workspace => "Multi-crate workspace",
            Self::Custom => "Custom project template",
        }
    }
}

/// Variable that can be substituted in templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name (e.g., "project_name")
    pub name: String,

    /// Description shown to user
    pub description: String,

    /// Default value if not provided
    pub default: Option<String>,

    /// Whether this variable is required
    pub required: bool,

    /// Validation regex pattern
    pub pattern: Option<String>,
}

/// File to be generated from template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    /// Source path in template
    pub source: PathBuf,

    /// Destination path (can include variables)
    pub destination: PathBuf,

    /// Whether to process this file for variable substitution
    pub process: bool,

    /// File permissions (Unix-style)
    pub permissions: Option<u32>,
}

impl TemplateManifest {
    /// Create a new manifest with defaults
    pub fn new(name: String, kind: TemplateKind) -> Self {
        Self {
            name,
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            kind,
            variables: Vec::new(),
            files: Vec::new(),
            post_generate: Vec::new(),
            min_rust_version: None,
            edition: "2024".to_string(),
        }
    }

    /// Add a variable to the manifest
    pub fn add_variable(&mut self, variable: TemplateVariable) {
        self.variables.push(variable);
    }

    /// Add a file to the manifest
    pub fn add_file(&mut self, file: TemplateFile) {
        self.files.push(file);
    }

    /// Add a post-generation command
    pub fn add_post_generate(&mut self, command: String) {
        self.post_generate.push(command);
    }

    /// Validate the manifest
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.is_empty() {
            return Err(crate::Error::validation("Template name cannot be empty"));
        }

        if self.files.is_empty() {
            return Err(crate::Error::validation(
                "Template must include at least one file",
            ));
        }

        // Check for duplicate variable names
        let mut seen = std::collections::HashSet::new();
        for var in &self.variables {
            if !seen.insert(&var.name) {
                return Err(crate::Error::validation(format!(
                    "Duplicate variable name: {}",
                    var.name
                )));
            }
        }

        Ok(())
    }
}

impl TemplateVariable {
    /// Create a new required variable
    pub fn required(name: String, description: String) -> Self {
        Self {
            name,
            description,
            default: None,
            required: true,
            pattern: None,
        }
    }

    /// Create a new optional variable with default
    pub fn optional(name: String, description: String, default: String) -> Self {
        Self {
            name,
            description,
            default: Some(default),
            required: false,
            pattern: None,
        }
    }

    /// Set validation pattern
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }
}

impl TemplateFile {
    /// Create a new template file
    pub fn new(source: PathBuf, destination: PathBuf) -> Self {
        Self {
            source,
            destination,
            process: true,
            permissions: None,
        }
    }

    /// Create a static file (no processing)
    pub fn static_file(source: PathBuf, destination: PathBuf) -> Self {
        Self {
            source,
            destination,
            process: false,
            permissions: None,
        }
    }

    /// Set file permissions
    pub fn with_permissions(mut self, permissions: u32) -> Self {
        self.permissions = Some(permissions);
        self
    }
}

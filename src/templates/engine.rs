//! Template engine for processing and generating projects

use super::manifest::{TemplateFile, TemplateManifest};
use crate::{Error, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Template engine for processing templates
pub struct TemplateEngine {
    /// Variables for substitution
    variables: HashMap<String, String>,

    /// Template manifest
    manifest: TemplateManifest,

    /// Source directory for template files
    source_dir: PathBuf,
}

/// Variable for template substitution
pub use super::manifest::TemplateVariable;

impl TemplateEngine {
    /// Create a new template engine
    pub fn new(manifest: TemplateManifest, source_dir: PathBuf) -> Self {
        Self {
            variables: HashMap::new(),
            manifest,
            source_dir,
        }
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: String, value: String) -> Result<()> {
        // Validate against pattern if specified
        if let Some(var_def) = self.manifest.variables.iter().find(|v| v.name == name) {
            if let Some(pattern) = &var_def.pattern {
                let regex = Regex::new(pattern)
                    .map_err(|e| Error::validation(format!("Invalid regex pattern: {}", e)))?;
                if !regex.is_match(&value) {
                    return Err(Error::validation(format!(
                        "Value '{}' does not match pattern for {}",
                        value, name
                    )));
                }
            }
        }

        self.variables.insert(name.clone(), value.clone());

        // Auto-generate derived variables
        if name == "project_name" {
            // Convert hyphenated names to valid Rust identifiers
            let project_ident = value.replace('-', "_");
            self.variables
                .insert("project_ident".to_string(), project_ident);
        }

        Ok(())
    }

    /// Set multiple variables at once
    pub fn set_variables(&mut self, vars: HashMap<String, String>) -> Result<()> {
        for (name, value) in vars {
            self.set_variable(name, value)?;
        }
        Ok(())
    }

    /// Generate project from template
    pub fn generate(&self, target_dir: &Path) -> Result<()> {
        // Validate all required variables are set
        self.validate_variables()?;

        // Create target directory
        if target_dir.exists() {
            return Err(Error::validation(format!(
                "Target directory already exists: {}",
                target_dir.display()
            )));
        }
        fs::create_dir_all(target_dir)?;

        // Process each file
        for file in &self.manifest.files {
            self.process_file(file, target_dir)?;
        }

        // Run post-generation commands
        self.run_post_generate(target_dir)?;

        Ok(())
    }

    /// Validate all required variables are set
    fn validate_variables(&self) -> Result<()> {
        for var in &self.manifest.variables {
            if var.required && !self.variables.contains_key(&var.name) && var.default.is_none() {
                return Err(Error::validation(format!(
                    "Required variable '{}' is not set",
                    var.name
                )));
            }
        }
        Ok(())
    }

    /// Process a single template file
    fn process_file(&self, file: &TemplateFile, target_dir: &Path) -> Result<()> {
        let source_path = self.source_dir.join(&file.source);

        // Substitute variables in destination path
        let dest_str = self.substitute_variables(&file.destination.to_string_lossy())?;
        let dest_path = target_dir.join(dest_str);

        // Create parent directories
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if file.process {
            // Read and process content
            let content = fs::read_to_string(&source_path)?;
            let processed = self.substitute_variables(&content)?;
            fs::write(&dest_path, processed)?;
        } else {
            // Copy file as-is
            fs::copy(&source_path, &dest_path)?;
        }

        // Set permissions if specified (Unix only)
        #[cfg(unix)]
        if let Some(perms) = file.permissions {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(perms);
            fs::set_permissions(&dest_path, permissions)?;
        }

        Ok(())
    }

    /// Substitute variables in text
    fn substitute_variables(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();

        // Build a complete variable map with defaults
        let mut vars = self.variables.clone();
        for var_def in &self.manifest.variables {
            if !vars.contains_key(&var_def.name) {
                if let Some(default) = &var_def.default {
                    vars.insert(var_def.name.clone(), default.clone());
                }
            }
        }

        // Replace {{variable_name}} with values
        for (name, value) in vars {
            let pattern = format!("{{{{{}}}}}", name);
            result = result.replace(&pattern, &value);
        }

        // Check for unsubstituted variables
        let unsubstituted = Regex::new(r"\{\{[^}]+\}\}")
            .map_err(|e| Error::validation(format!("Regex error: {}", e)))?;
        if unsubstituted.is_match(&result) {
            if let Some(m) = unsubstituted.find(&result) {
                return Err(Error::validation(format!(
                    "Unsubstituted variable found: {}",
                    m.as_str()
                )));
            }
        }

        Ok(result)
    }

    /// Run post-generation commands
    fn run_post_generate(&self, target_dir: &Path) -> Result<()> {
        for command in &self.manifest.post_generate {
            let processed = self.substitute_variables(command)?;

            // Parse command
            let parts: Vec<&str> = processed.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            // Execute command
            let output = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .current_dir(target_dir)
                .output()
                .map_err(|e| Error::process(format!("Failed to run command: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::process(format!(
                    "Command failed: {}\n{}",
                    processed, stderr
                )));
            }
        }

        Ok(())
    }

    /// Get list of required variables
    pub fn required_variables(&self) -> Vec<&TemplateVariable> {
        self.manifest
            .variables
            .iter()
            .filter(|v| v.required && v.default.is_none())
            .collect()
    }

    /// Get list of optional variables
    pub fn optional_variables(&self) -> Vec<&TemplateVariable> {
        self.manifest
            .variables
            .iter()
            .filter(|v| !v.required || v.default.is_some())
            .collect()
    }
}

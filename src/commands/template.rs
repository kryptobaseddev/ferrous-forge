//! Template management commands
#![allow(clippy::too_many_lines)]

use crate::templates::{TemplateEngine, TemplateRegistry};
use crate::{Error, Result};
use clap::Subcommand;
use console::style;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Template subcommands
#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    /// List available templates
    List,

    /// Create a new project from template
    Create {
        /// Template name
        template: String,

        /// Target directory for the new project
        #[arg(default_value = ".")]
        target: PathBuf,

        /// Variable values (key=value)
        #[arg(long, value_parser = parse_var)]
        var: Vec<(String, String)>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Show template details
    Info {
        /// Template name
        template: String,
    },

    /// Validate a template
    Validate {
        /// Path to template directory
        path: PathBuf,
    },
}

/// Parse a key=value variable
fn parse_var(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(Error::validation(format!(
            "Invalid variable format: {}. Use key=value",
            s
        )));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

impl TemplateCommand {
    /// Execute the template command
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::List => list_templates().await,
            Self::Create {
                template,
                target,
                var,
                yes,
            } => create_project(&template, &target, var, yes).await,
            Self::Info { template } => show_template_info(&template).await,
            Self::Validate { path } => validate_template(&path).await,
        }
    }
}

/// List available templates
async fn list_templates() -> Result<()> {
    let registry = TemplateRegistry::new();
    let templates = registry.list_templates();

    println!("{}", style("📦 Available Templates").bold().cyan());
    println!();

    // Header
    println!(
        "{:15} {:15} {}",
        style("NAME").bold(),
        style("TYPE").bold(),
        style("DESCRIPTION").bold()
    );
    println!("{}", style("-".repeat(70)).dim());

    for (name, kind, description) in templates {
        println!(
            "{:15} {:15} {}",
            style(name).green(),
            style(format!("{:?}", kind)).yellow(),
            description
        );
    }

    println!();
    println!(
        "{}",
        style("💡 Use 'ferrous-forge template create <name>' to create a project").dim()
    );

    Ok(())
}

/// Create a project from template
async fn create_project(
    template_name: &str,
    target_dir: &Path,
    vars: Vec<(String, String)>,
    skip_prompts: bool,
) -> Result<()> {
    let registry = TemplateRegistry::new();

    // Get the template
    let template = registry
        .get_builtin(template_name)
        .ok_or_else(|| Error::validation(format!("Template not found: {}", template_name)))?;

    println!(
        "{}",
        style(format!(
            "🚀 Creating project from template: {}",
            template_name
        ))
        .bold()
        .cyan()
    );
    println!();

    // Create a temporary directory for template files
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("ferrous-forge-{}", timestamp));
    std::fs::create_dir_all(&temp_dir)?;

    // Write template files to temp directory
    for (path, content) in &template.files {
        let file_path = temp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file_path, content)?;
    }

    // Create engine
    let mut engine = TemplateEngine::new(template.manifest.clone(), temp_dir.clone());

    // Set provided variables
    let mut variables = HashMap::new();
    for (key, value) in vars {
        variables.insert(key, value);
    }

    // Prompt for required variables if not provided
    if !skip_prompts {
        for var in engine.required_variables() {
            if !variables.contains_key(&var.name) {
                println!("{}: {}", style(&var.name).yellow(), var.description);
                print!("  Enter {}: ", var.name);
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut value = String::new();
                io::stdin().read_line(&mut value)?;
                let value = value.trim().to_string();
                variables.insert(var.name.clone(), value);
            }
        }
    }

    // Set all variables
    engine.set_variables(variables)?;

    // Generate the project
    engine.generate(target_dir)?;

    println!();
    println!(
        "{}",
        style("✅ Project created successfully!").green().bold()
    );
    println!();
    println!("Next steps:");
    println!("  cd {}", target_dir.display());
    println!("  cargo build");
    println!("  ferrous-forge validate .");

    Ok(())
}

/// Show template information
async fn show_template_info(template_name: &str) -> Result<()> {
    let registry = TemplateRegistry::new();

    let template = registry
        .get_builtin(template_name)
        .ok_or_else(|| Error::validation(format!("Template not found: {}", template_name)))?;

    let manifest = &template.manifest;

    println!(
        "{}",
        style(format!("📋 Template: {}", manifest.name))
            .bold()
            .cyan()
    );
    println!();
    println!("  {} {}", style("Version:").bold(), manifest.version);
    println!("  {} {:?}", style("Type:").bold(), manifest.kind);
    println!(
        "  {} {}",
        style("Description:").bold(),
        manifest.description
    );
    println!("  {} {}", style("Author:").bold(), manifest.author);
    println!("  {} {}", style("Edition:").bold(), manifest.edition);

    if !manifest.variables.is_empty() {
        println!();
        println!("{}", style("Variables:").bold());
        for var in &manifest.variables {
            let required = if var.required { "*" } else { "" };
            println!(
                "  {} {}{} - {}",
                style("•").dim(),
                style(&var.name).yellow(),
                style(required).red(),
                var.description
            );
            if let Some(default) = &var.default {
                println!("    {} {}", style("Default:").dim(), default);
            }
            if let Some(pattern) = &var.pattern {
                println!("    {} {}", style("Pattern:").dim(), pattern);
            }
        }
    }

    if !manifest.files.is_empty() {
        println!();
        println!("{}", style("Files:").bold());
        for file in &manifest.files {
            let process_marker = if file.process { "📝" } else { "📄" };
            println!(
                "  {} {} → {}",
                process_marker,
                style(file.source.display()).dim(),
                style(file.destination.display()).green()
            );
        }
    }

    if !manifest.post_generate.is_empty() {
        println!();
        println!("{}", style("Post-generation commands:").bold());
        for cmd in &manifest.post_generate {
            println!("  {} {}", style("$").dim(), cmd);
        }
    }

    Ok(())
}

/// Validate a template
async fn validate_template(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(Error::validation(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }

    let manifest_path = path.join("template.toml");
    if !manifest_path.exists() {
        return Err(Error::validation(
            "No template.toml found in the specified directory",
        ));
    }

    // Read and parse manifest
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: crate::templates::TemplateManifest = toml::from_str(&content)
        .map_err(|e| Error::validation(format!("Invalid template manifest: {}", e)))?;

    // Validate manifest
    manifest.validate()?;

    // Check all referenced files exist
    for file in &manifest.files {
        let file_path = path.join(&file.source);
        if !file_path.exists() {
            return Err(Error::validation(format!(
                "Template file not found: {}",
                file.source.display()
            )));
        }
    }

    println!("{}", style("✅ Template is valid!").green().bold());
    println!();
    println!("Template: {}", style(&manifest.name).cyan());
    println!("Version: {}", manifest.version);
    println!("Files: {}", manifest.files.len());
    println!("Variables: {}", manifest.variables.len());
    Ok(())
}

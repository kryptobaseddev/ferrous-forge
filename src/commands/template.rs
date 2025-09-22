//! Template management commands

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
    let template = get_template(&registry, template_name)?;
    
    display_creation_header(template_name);
    let temp_dir = setup_template_files(&template)?;
    let mut engine = TemplateEngine::new(template.manifest.clone(), temp_dir);
    
    let variables = collect_template_variables(vars, &engine, skip_prompts)?;
    engine.set_variables(variables)?;
    engine.generate(target_dir)?;
    
    display_creation_success(target_dir);
    Ok(())
}

/// Get template from registry
fn get_template<'a>(
    registry: &'a TemplateRegistry,
    template_name: &str,
) -> Result<&'a crate::templates::registry::BuiltinTemplate> {
    registry
        .get_builtin(template_name)
        .ok_or_else(|| Error::validation(format!("Template not found: {}", template_name)))
}

/// Display project creation header
fn display_creation_header(template_name: &str) {
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
}

/// Setup template files in temporary directory
fn setup_template_files(template: &crate::templates::registry::BuiltinTemplate) -> Result<PathBuf> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("ferrous-forge-{}", timestamp));
    std::fs::create_dir_all(&temp_dir)?;

    for (path, content) in &template.files {
        let file_path = temp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file_path, content)?;
    }
    
    Ok(temp_dir)
}

/// Collect template variables from command line and prompts
fn collect_template_variables(
    vars: Vec<(String, String)>,
    engine: &TemplateEngine,
    skip_prompts: bool,
) -> Result<HashMap<String, String>> {
    let mut variables = HashMap::new();
    
    // Set provided variables
    for (key, value) in vars {
        variables.insert(key, value);
    }

    // Prompt for required variables if not provided
    if !skip_prompts {
        prompt_for_missing_variables(&mut variables, engine)?;
    }
    
    Ok(variables)
}

/// Prompt user for missing required variables
fn prompt_for_missing_variables(
    variables: &mut HashMap<String, String>,
    engine: &TemplateEngine,
) -> Result<()> {
    use std::io::{self, Write};
    
    for var in engine.required_variables() {
        if !variables.contains_key(&var.name) {
            println!("{}: {}", style(&var.name).yellow(), var.description);
            print!("  Enter {}: ", var.name);
            io::stdout().flush()?;
            let mut value = String::new();
            io::stdin().read_line(&mut value)?;
            let value = value.trim().to_string();
            variables.insert(var.name.clone(), value);
        }
    }
    
    Ok(())
}

/// Display project creation success message
fn display_creation_success(target_dir: &Path) {
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
}

/// Show template information
async fn show_template_info(template_name: &str) -> Result<()> {
    let registry = TemplateRegistry::new();
    let template = get_template(&registry, template_name)?;
    let manifest = &template.manifest;

    display_template_header(manifest);
    display_template_basic_info(manifest);
    display_template_variables(manifest);
    display_template_files(manifest);
    display_post_generation_commands(manifest);

    Ok(())
}

/// Display template header
fn display_template_header(manifest: &crate::templates::TemplateManifest) {
    println!(
        "{}",
        style(format!("📋 Template: {}", manifest.name))
            .bold()
            .cyan()
    );
    println!();
}

/// Display basic template information
fn display_template_basic_info(manifest: &crate::templates::TemplateManifest) {
    println!("  {} {}", style("Version:").bold(), manifest.version);
    println!("  {} {:?}", style("Type:").bold(), manifest.kind);
    println!(
        "  {} {}",
        style("Description:").bold(),
        manifest.description
    );
    println!("  {} {}", style("Author:").bold(), manifest.author);
    println!("  {} {}", style("Edition:").bold(), manifest.edition);
}

/// Display template variables information
fn display_template_variables(manifest: &crate::templates::TemplateManifest) {
    if !manifest.variables.is_empty() {
        println!();
        println!("{}", style("Variables:").bold());
        for var in &manifest.variables {
            display_variable_info(var);
        }
    }
}

/// Display information for a single variable
fn display_variable_info(var: &crate::templates::TemplateVariable) {
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

/// Display template files information
fn display_template_files(manifest: &crate::templates::TemplateManifest) {
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
}

/// Display post-generation commands
fn display_post_generation_commands(manifest: &crate::templates::TemplateManifest) {
    if !manifest.post_generate.is_empty() {
        println!();
        println!("{}", style("Post-generation commands:").bold());
        for cmd in &manifest.post_generate {
            println!("  {} {}", style("$").dim(), cmd);
        }
    }
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

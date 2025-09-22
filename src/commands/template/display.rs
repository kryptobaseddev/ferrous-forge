//! Template information display functionality

use crate::templates::TemplateRegistry;
use crate::{Error, Result};
use console::style;
use std::path::Path;

/// Show detailed information about a template
pub async fn show_template_info(template_name: &str) -> Result<()> {
    let registry = TemplateRegistry::new();
    let template = registry
        .get_builtin(template_name)
        .ok_or_else(|| Error::template(format!("Template '{}' not found", template_name)))?;

    display_template_header(&template.manifest);
    display_template_basic_info(&template.manifest);
    display_template_variables(&template.manifest);
    display_template_files(&template.manifest);
    display_post_generation_commands(&template.manifest);

    Ok(())
}

/// Validate a template manifest
pub async fn validate_template_manifest(manifest_path: &Path) -> Result<()> {
    let content = tokio::fs::read_to_string(manifest_path)
        .await
        .map_err(|e| Error::template(format!("Failed to read manifest: {}", e)))?;

    let manifest: crate::templates::TemplateManifest = toml::from_str(&content)
        .map_err(|e| Error::template(format!("Invalid template manifest: {}", e)))?;

    println!("{}", style("✅ Template manifest is valid").green().bold());
    println!("   📄 Template: {}", manifest.name);
    println!("   📝 Description: {}", manifest.description);
    println!("   📊 Variables: {}", manifest.variables.len());
    println!("   📁 Files: {}", manifest.files.len());

    Ok(())
}

fn display_template_header(manifest: &crate::templates::TemplateManifest) {
    println!(
        "{}",
        style(format!("📋 Template: {}", manifest.name))
            .cyan()
            .bold()
    );
    println!("{}", style("═".repeat(60)).dim());
    println!();
}

fn display_template_basic_info(manifest: &crate::templates::TemplateManifest) {
    println!("{}", style("📝 Description:").yellow().bold());
    println!("   {}", manifest.description);
    println!();

    println!("{}", style("🏷️  Version:").yellow().bold());
    println!("   {}", manifest.version);
    println!();
}

fn display_template_variables(manifest: &crate::templates::TemplateManifest) {
    if !manifest.variables.is_empty() {
        println!("{}", style("🔧 Variables:").yellow().bold());
        for var in &manifest.variables {
            display_variable_info(var);
        }
        println!();
    }
}

fn display_variable_info(var: &crate::templates::TemplateVariable) {
    let required_text = if var.required {
        style("(required)").red()
    } else {
        style("(optional)").dim()
    };

    println!(
        "   {} {} {}",
        style("•").cyan(),
        style(&var.name).white().bold(),
        required_text
    );

    println!("     {}", style(&var.description).dim());

    if let Some(default) = &var.default {
        println!("     Default: {}", style(default).cyan());
    }
}

fn display_template_files(manifest: &crate::templates::TemplateManifest) {
    if !manifest.files.is_empty() {
        println!("{}", style("📁 Files:").yellow().bold());
        for file in &manifest.files {
            println!("   {} {}", style("•").cyan(), file.destination.display());
        }
        println!();
    }
}

fn display_post_generation_commands(manifest: &crate::templates::TemplateManifest) {
    if !manifest.post_generate.is_empty() {
        let commands = &manifest.post_generate;
        if !commands.is_empty() {
            println!("{}", style("⚡ Post-generation commands:").yellow().bold());
            for (i, command) in commands.iter().enumerate() {
                println!("   {}. {}", i + 1, style(command).cyan());
            }
            println!();
        }
    }
}

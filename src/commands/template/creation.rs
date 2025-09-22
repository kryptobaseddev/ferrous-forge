//! Template creation functionality

use crate::templates::{TemplateEngine, TemplateRegistry};
use crate::{Error, Result};
use console::style;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Create a new project from template
pub async fn create_from_template(
    template_name: &str,
    output_dir: &Path,
    variables: &[String],
) -> Result<()> {
    display_creation_header(template_name);
    
    let registry = TemplateRegistry::new();
    let template = get_template(&registry, template_name)?;
    
    // Setup template files in temporary directory
    let template_dir = setup_template_files(&template)?;
    
    // Parse and collect variables
    let mut template_vars = collect_template_variables(variables)?;
    
    // Prompt for any missing required variables
    prompt_for_missing_variables(&template, &mut template_vars)?;
    
    // Create the project using the engine
    let mut engine = TemplateEngine::new(template.manifest.clone(), template_dir.clone());
    
    // Set variables
    for (key, value) in template_vars {
        engine.set_variable(key, value)?;
    }
    
    // Generate the project
    engine.generate(output_dir)?;
    
    display_creation_success(output_dir);
    Ok(())
}

fn get_template<'a>(
    registry: &'a TemplateRegistry,
    template_name: &str,
) -> Result<&'a crate::templates::registry::BuiltinTemplate> {
    registry.get_builtin(template_name)
        .ok_or_else(|| Error::template(format!("Template '{}' not found", template_name)))
}

fn display_creation_header(template_name: &str) {
    println!("{}", style(format!("🏗️  Creating project from template '{}'", template_name))
        .cyan().bold());
    println!();
}

fn setup_template_files(template: &crate::templates::registry::BuiltinTemplate) -> Result<PathBuf> {
    use std::io::Write;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new()
        .map_err(|e| Error::template(format!("Failed to create temporary directory: {}", e)))?;
    
    let template_path = temp_dir.path().join("template");
    std::fs::create_dir_all(&template_path)
        .map_err(|e| Error::template(format!("Failed to create template directory: {}", e)))?;
    
    // Write manifest
    let manifest_path = template_path.join("template.toml");
    let manifest_toml = toml::to_string(&template.manifest)
        .map_err(|e| Error::template(format!("Failed to serialize manifest: {}", e)))?;
    let mut manifest_file = std::fs::File::create(&manifest_path)
        .map_err(|e| Error::template(format!("Failed to create manifest file: {}", e)))?;
    manifest_file.write_all(manifest_toml.as_bytes())
        .map_err(|e| Error::template(format!("Failed to write manifest: {}", e)))?;
    
    // Write template files
    for (file_path, content) in &template.files {
        let full_path = template_path.join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::template(format!("Failed to create directory: {}", e)))?;
        }
        
        let mut file = std::fs::File::create(&full_path)
            .map_err(|e| Error::template(format!("Failed to create file: {}", e)))?;
        file.write_all(content.as_bytes())
            .map_err(|e| Error::template(format!("Failed to write file: {}", e)))?;
    }
    
    // Prevent cleanup by returning owned path
    Ok(template_path)
}

fn collect_template_variables(variables: &[String]) -> Result<HashMap<String, String>> {
    let mut template_vars = HashMap::new();
    
    for var in variables {
        let (key, value) = parse_var(var)?;
        template_vars.insert(key, value);
    }
    
    Ok(template_vars)
}

fn parse_var(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(Error::template(format!("Invalid variable format: '{}'. Use key=value", s)));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn prompt_for_missing_variables(
    template: &crate::templates::registry::BuiltinTemplate,
    template_vars: &mut HashMap<String, String>,
) -> Result<()> {
    use dialoguer::Input;
    
    for var in &template.manifest.variables {
        if var.required && !template_vars.contains_key(&var.name) {
            let prompt = format!("{} ({})", var.name, var.description);
            
            let value: String = Input::new()
                .with_prompt(&prompt)
                .interact()
                .map_err(|e| Error::template(format!("Failed to get input: {}", e)))?;
            
            template_vars.insert(var.name.clone(), value);
        }
    }
    
    Ok(())
}

fn display_creation_success(target_dir: &Path) {
    println!();
    println!("{}", style("✅ Project created successfully!").green().bold());
    println!("   📁 Location: {}", style(target_dir.display()).cyan());
    println!();
    println!("   Next steps:");
    println!("   1. {} {}", style("cd").cyan(), target_dir.display());
    println!("   2. {} (if needed)", style("cargo build").cyan());
    println!("   3. {} to validate code quality", style("ferrous-forge validate .").cyan());
    println!();
}
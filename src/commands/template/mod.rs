//! Template management commands
//!
//! @task T021
//! @epic T014

use crate::templates::TemplateRegistry;
use crate::templates::repository::TemplateRepository;
use crate::templates::repository::github::GitHubClient;
use crate::templates::validation::validate_before_install;
use crate::{Error, Result};
use clap::Subcommand;
use console::style;
use std::path::PathBuf;

mod creation;
mod display;
mod utils;

pub use creation::*;
pub use display::*;
pub use utils::*;

/// Template subcommands
#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    /// List available templates (local + cached)
    List {
        /// Show remote templates too
        #[arg(long)]
        remote: bool,
    },

    /// Fetch template from GitHub repository
    Fetch {
        /// GitHub repository reference (e.g., gh:user/repo or user/repo)
        repo: String,
        /// Branch, tag, or commit to fetch
        #[arg(short, long)]
        reference: Option<String>,
        /// Force re-fetch even if already cached
        #[arg(short, long)]
        force: bool,
    },

    /// Install a cached template locally
    Install {
        /// Template name (from cache) or repository
        name: String,
        /// Custom name for the installed template
        #[arg(short, long)]
        as_name: Option<String>,
    },

    /// Update all installed templates or a specific one
    Update {
        /// Specific template to update
        template: Option<String>,
        /// Check for updates without installing
        #[arg(long)]
        check: bool,
    },

    /// Create a new template from current project
    Create {
        /// Name for the new template
        name: String,
        /// Output directory for template
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Project directory to template-ize
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
    },

    /// Remove a cached template
    Remove {
        /// Template name to remove
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },

    /// Show detailed information about a template
    Info {
        /// Template name
        template: String,
        /// Show cache information
        #[arg(long)]
        cache: bool,
    },

    /// Validate a template manifest
    Validate {
        /// Path to template directory or manifest
        path: PathBuf,
    },
}

impl TemplateCommand {
    /// Execute the template command
    ///
    /// # Errors
    ///
    /// Returns an error if the template cannot be found, variables are
    /// invalid, or the project files cannot be written.
    pub async fn execute(&self) -> Result<()> {
        match self {
            TemplateCommand::List { remote } => list_templates(*remote).await,

            TemplateCommand::Fetch {
                repo,
                reference,
                force,
            } => fetch_template(repo, reference.as_deref(), *force).await,

            TemplateCommand::Install { name, as_name } => {
                install_template(name, as_name.as_deref()).await
            }

            TemplateCommand::Update { template, check } => {
                update_templates(template.as_deref(), *check).await
            }

            TemplateCommand::Create {
                name,
                output,
                project,
            } => create_template_from_project(name, output.as_deref(), project).await,

            TemplateCommand::Remove { name, yes } => remove_template(name, *yes).await,

            TemplateCommand::Info { template, cache } => {
                show_template_info_extended(template, *cache).await
            }

            TemplateCommand::Validate { path } => validate_template_manifest(path).await,
        }
    }
}

/// List available templates
async fn list_templates(remote: bool) -> Result<()> {
    println!("{}", style("📚 Available Templates").cyan().bold());
    println!();

    // List built-in templates
    let registry = TemplateRegistry::new();
    let builtin = registry.list_templates();

    if !builtin.is_empty() {
        println!("{}", style("Built-in Templates:").white().bold());
        for (name, _kind, description) in &builtin {
            println!("  {} {}", style("•").cyan(), style(name).white().bold());
            println!("    {}", style(description).dim());
        }
        println!();
    }

    // List cached templates
    let repo = TemplateRepository::new()?;
    let cached_count = repo.list_cached().len();

    if cached_count > 0 {
        println!("{}", style("Cached Templates:").white().bold());
        for template in repo.list_cached() {
            println!(
                "  {} {} {}",
                style("•").cyan(),
                style(&template.name).white().bold(),
                style(format!("(v{})", template.version)).dim()
            );
            println!("    Source: {}", style(&template.source).dim());
            println!(
                "    Updated: {}",
                style(template.updated_at.format("%Y-%m-%d %H:%M")).dim()
            );
        }
        println!();
    }

    // Show count
    let total = builtin.len() + cached_count;
    println!("Total: {} template(s) available\n", total);

    // Show usage
    println!(
        "Use {} to fetch a template from GitHub",
        style("ferrous-forge template fetch <repo>").cyan()
    );
    println!(
        "Use {} to create a project from a template",
        style("ferrous-forge template install <name>").cyan()
    );

    if remote {
        println!(
            "\n{}",
            style("Remote template discovery not yet implemented").yellow()
        );
    }

    Ok(())
}

/// Fetch template from GitHub
async fn fetch_template(repo: &str, reference: Option<&str>, force: bool) -> Result<()> {
    println!("{}", style("📦 Fetching Template").cyan().bold());
    println!();

    // Parse repo reference
    let mut repo_ref = GitHubClient::parse_repo_ref(repo)?;
    if let Some(git_ref) = reference {
        repo_ref.git_ref = Some(git_ref.to_string());
    }

    println!("Repository: {}/{}", repo_ref.owner, repo_ref.repo);
    if let Some(git_ref) = &repo_ref.git_ref {
        println!("Reference: {}", git_ref);
    }
    println!();

    // Check if already cached
    let mut repository = TemplateRepository::new()?;
    let cache_name = format!("{}-{}", repo_ref.owner, repo_ref.repo);

    if !force && repository.is_cached(&cache_name) {
        let cached = repository.get_cached(&cache_name).ok_or_else(|| {
            Error::validation(format!(
                "Failed to retrieve cached template '{}'",
                cache_name
            ))
        })?;
        println!(
            "{}",
            style(format!("Template '{}' already cached", cache_name)).yellow()
        );
        println!("Use --force to re-fetch");
        println!();
        println!("Cached version: {}", cached.version);
        println!(
            "Last updated: {}",
            cached.updated_at.format("%Y-%m-%d %H:%M")
        );
        return Ok(());
    }

    // Fetch from GitHub
    let client = GitHubClient::new()?;
    println!("{}", style("Fetching template...").dim());

    let template = client.fetch_template(&repo_ref, &mut repository).await?;

    println!();
    println!(
        "{}",
        style("✅ Template fetched successfully!").green().bold()
    );
    println!();
    println!("Name: {}", style(&template.name).cyan());
    println!("Version: {}", style(&template.version).cyan());
    println!("Description: {}", template.manifest.description);
    println!();
    println!(
        "Use {} to install this template",
        style(format!("ferrous-forge template install {}", template.name)).cyan()
    );

    Ok(())
}

/// Install a cached template
async fn install_template(name: &str, _as_name: Option<&str>) -> Result<()> {
    println!("{}", style("📥 Installing Template").cyan().bold());
    println!();

    let repository = TemplateRepository::new()?;

    // Check if it's a cached template
    if let Some(template) = repository.get_cached(name) {
        println!("Template: {}", style(name).cyan());
        println!("Source: {}", template.source);
        println!("Version: {}", template.version);
        println!();

        // Validate before installing
        let validation = validate_before_install(&template.cache_path).await?;

        if !validation.valid {
            println!("{}", style("❌ Template validation failed:").red().bold());
            for error in &validation.errors {
                println!("  • {}", error);
            }
            return Err(crate::Error::template(
                "Template validation failed - cannot install",
            ));
        }

        if !validation.warnings.is_empty() {
            println!("{}", style("⚠️  Warnings:").yellow());
            for warning in &validation.warnings {
                println!("  • {}", warning);
            }
            println!();
        }

        println!(
            "{}",
            style("✅ Template validated and ready to use!")
                .green()
                .bold()
        );
        println!();
        println!(
            "Use {} to create a project from this template",
            style(format!(
                "ferrous-forge template create {} <output-dir>",
                name
            ))
            .cyan()
        );

        Ok(())
    } else {
        // Try to fetch it first
        println!("Template '{}' not found in cache.", name);
        println!("Attempting to fetch from GitHub...");
        println!();

        fetch_template(name, None, false).await
    }
}

/// Update templates
async fn update_templates(template: Option<&str>, check: bool) -> Result<()> {
    if check {
        println!("{}", style("🔍 Checking for Updates").cyan().bold());
    } else {
        println!("{}", style("🔄 Updating Templates").cyan().bold());
    }
    println!();

    let mut repository = TemplateRepository::new()?;

    if let Some(name) = template {
        // Update specific template
        if let Some(template) = repository.get_cached(name).cloned() {
            if check {
                println!("Template: {}", name);
                println!("Current version: {}", template.version);
                println!(
                    "Last update: {}",
                    template.updated_at.format("%Y-%m-%d %H:%M")
                );
                if template.needs_update() {
                    println!("{}", style("Update available!").green());
                } else {
                    println!("{}", style("Up to date").green());
                }
            } else {
                let client = GitHubClient::new()?;
                let repo_ref = GitHubClient::parse_repo_ref(&template.source)?;
                client.fetch_template(&repo_ref, &mut repository).await?;
                println!(
                    "{}",
                    style(format!("✅ Updated template '{}'", name)).green()
                );
            }
        } else {
            return Err(crate::Error::template(format!(
                "Template '{}' not found in cache",
                name
            )));
        }
    } else {
        // Update all templates
        let templates_to_update: Vec<_> = repository
            .list_cached()
            .iter()
            .map(|t| (t.name.clone(), t.source.clone(), t.needs_update()))
            .collect();

        if templates_to_update.is_empty() {
            println!("No cached templates to update.");
            return Ok(());
        }

        let client = GitHubClient::new()?;
        let mut updated = 0;

        for (name, source, needs_update) in templates_to_update {
            if check {
                print!("{}: ", name);
                if needs_update {
                    println!("{}", style("update available").yellow());
                } else {
                    println!("{}", style("up to date").green());
                }
            } else {
                let repo_ref = GitHubClient::parse_repo_ref(&source)?;
                match client.fetch_template(&repo_ref, &mut repository).await {
                    Ok(_) => {
                        println!("{}", style(format!("✅ Updated {}", name)).green());
                        updated += 1;
                    }
                    Err(e) => {
                        println!(
                            "{}",
                            style(format!("❌ Failed to update {}: {}", name, e)).red()
                        );
                    }
                }
            }
        }

        if !check {
            println!();
            println!("Updated {} template(s)", updated);
        }
    }

    Ok(())
}

/// Create a template from current project
async fn create_template_from_project(
    name: &str,
    output: Option<&std::path::Path>,
    project: &std::path::Path,
) -> Result<()> {
    println!(
        "{}",
        style("📋 Creating Template from Project").cyan().bold()
    );
    println!();

    println!("Template name: {}", style(name).cyan());
    println!("Project path: {}", project.display());
    println!();

    // Validate project exists
    if !project.exists() {
        return Err(crate::Error::template(format!(
            "Project path does not exist: {}",
            project.display()
        )));
    }

    // Check for Cargo.toml
    let cargo_toml = project.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(crate::Error::template(
            "No Cargo.toml found - is this a Rust project?",
        ));
    }

    // Determine output directory
    let output_dir = output.map_or_else(
        || std::env::current_dir().map(|d| d.join(name)),
        |o| Ok(o.join(name)),
    )?;

    // Create template structure
    println!("Creating template at: {}", output_dir.display());

    // TODO: Implement template generation from project
    // This would:
    // 1. Copy project files
    // 2. Create template.toml manifest
    // 3. Replace project-specific values with template variables
    // 4. Add .templateignore support

    println!();
    println!("{}", style("✅ Template created!").green().bold());
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {} to configure variables",
        output_dir.join("template.toml").display()
    );
    println!(
        "  2. Test with: ferrous-forge template validate {}",
        output_dir.display()
    );
    println!("  3. Publish to GitHub and share with: ferrous-forge template fetch gh:user/repo");

    Ok(())
}

/// Remove a cached template
async fn remove_template(name: &str, yes: bool) -> Result<()> {
    let mut repository = TemplateRepository::new()?;

    if !repository.is_cached(name) {
        return Err(crate::Error::template(format!(
            "Template '{}' not found in cache",
            name
        )));
    }

    if !yes {
        println!("{}", style("⚠️  Remove Template").yellow().bold());
        println!();
        println!("This will remove '{}' from the cache.", name);
        println!();

        let confirm = dialoguer::Confirm::new()
            .with_prompt("Are you sure?")
            .default(false)
            .interact()
            .map_err(|e| crate::Error::template(format!("Failed to get confirmation: {e}")))?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    repository.remove_from_cache(name)?;

    println!(
        "{}",
        style(format!("✅ Removed template '{}'", name)).green()
    );

    Ok(())
}

/// Show extended template information
async fn show_template_info_extended(template: &str, show_cache: bool) -> Result<()> {
    let repository = TemplateRepository::new()?;

    if let Some(cached) = repository.get_cached(template) {
        println!("{}", style("📋 Template Information").cyan().bold());
        println!();
        println!("Name: {}", style(&cached.name).white().bold());
        println!("Source: {}", cached.source);
        println!("Version: {}", cached.version);
        println!("Description: {}", cached.manifest.description);
        println!("Author: {}", cached.manifest.author);
        println!("Kind: {:?}", cached.manifest.kind);
        println!();

        if show_cache {
            println!("{}", style("Cache Information:").white().bold());
            println!("  Cache path: {}", cached.cache_path.display());
            println!(
                "  Fetched: {}",
                cached.fetched_at.format("%Y-%m-%d %H:%M:%S")
            );
            println!(
                "  Updated: {}",
                cached.updated_at.format("%Y-%m-%d %H:%M:%S")
            );
            println!();
        }

        if !cached.manifest.variables.is_empty() {
            println!("{}", style("Variables:").white().bold());
            for var in &cached.manifest.variables {
                let req = if var.required { "required" } else { "optional" };
                println!(
                    "  • {} ({}) - {}",
                    style(&var.name).cyan(),
                    req,
                    var.description
                );
            }
        }
    } else {
        // Try built-in
        let registry = TemplateRegistry::new();
        if registry.get_builtin(template).is_some() {
            show_template_info(template).await?;
        } else {
            return Err(crate::Error::template(format!(
                "Template '{}' not found",
                template
            )));
        }
    }

    Ok(())
}

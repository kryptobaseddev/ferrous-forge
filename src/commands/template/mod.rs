//! Template management commands

use crate::templates::{TemplateEngine, TemplateRegistry};
use crate::{Error, Result};
use clap::Subcommand;
use console::style;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod creation;
mod display;
mod utils;

pub use creation::*;
pub use display::*;
pub use utils::*;

/// Template subcommands
#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    /// List available templates
    List,

    /// Create a new project from template
    Create {
        /// Template name
        template: String,

        /// Output directory
        output: PathBuf,

        /// Template variables in key=value format
        #[arg(long = "var", short = 'v')]
        variables: Vec<String>,
    },

    /// Show detailed information about a template
    Info {
        /// Template name
        template: String,
    },

    /// Validate a template manifest
    Validate {
        /// Path to template manifest
        manifest: PathBuf,
    },
}

impl TemplateCommand {
    /// Execute the template command
    pub async fn execute(&self) -> Result<()> {
        match self {
            TemplateCommand::List => {
                println!("{}", style("📚 Available Ferrous Forge Templates").cyan().bold());
                println!();

                let registry = TemplateRegistry::new();
                let templates = registry.list_templates();
                
                if templates.is_empty() {
                    println!("   No templates found.");
                    return Ok(());
                }

                for (name, _kind, description) in &templates {
                    println!("  {} {}", 
                        style("•").cyan(), 
                        style(name).white().bold()
                    );
                    println!("    {}", 
                        style(description).dim()
                    );
                    println!();
                }

                println!("Use {} to create a project from a template.", 
                    style("ferrous-forge template create <template-name> <output-dir>").cyan()
                );
                println!("Use {} to see detailed information about a template.",
                    style("ferrous-forge template info <template-name>").cyan()
                );
                Ok(())
            }

            TemplateCommand::Create { template, output, variables } => {
                create_from_template(template, output, variables).await
            }

            TemplateCommand::Info { template } => {
                show_template_info(template).await
            }

            TemplateCommand::Validate { manifest } => {
                validate_template_manifest(manifest).await
            }
        }
    }
}
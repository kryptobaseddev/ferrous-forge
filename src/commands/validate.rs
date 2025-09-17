//! Validate command implementation

use crate::{Result, validation::RustValidator};
use console::style;
use std::path::PathBuf;

/// Execute the validate command
pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    
    println!("{}", style("🦀 Running Ferrous Forge validation...").bold().cyan());
    println!("📁 Project: {}", project_path.display());
    println!();

    // Create validator
    let validator = RustValidator::new(project_path.clone())?;
    
    // Run validation
    let violations = validator.validate_project().await?;
    
    // Generate and display report
    let report = validator.generate_report(&violations);
    println!("{}", report);
    
    // Run clippy with our strict configuration
    println!("{}", style("🔧 Running Clippy with strict configuration...").bold().yellow());
    let clippy_result = validator.run_clippy().await?;
    
    if !clippy_result.success {
        println!("{}", style("❌ Clippy found issues:").red());
        println!("{}", clippy_result.output);
    } else {
        println!("{}", style("✅ Clippy validation passed!").green());
    }
    
    // Exit with error code if violations found
    if !violations.is_empty() || !clippy_result.success {
        std::process::exit(1);
    } else {
        println!("{}", style("🎉 All validations passed! Code meets Ferrous Forge standards.").bold().green());
    }

    Ok(())
}
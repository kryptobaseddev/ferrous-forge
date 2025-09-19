//! Validate command implementation

use crate::{doc_coverage, formatting, security, validation::RustValidator, Result};
use console::style;
use std::path::PathBuf;

/// Execute the validate command
pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    println!(
        "{}",
        style("🦀 Running Ferrous Forge validation...")
            .bold()
            .cyan()
    );
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
    println!(
        "{}",
        style("🔧 Running Clippy with strict configuration...")
            .bold()
            .yellow()
    );
    let clippy_result = validator.run_clippy().await?;

    if !clippy_result.success {
        println!("{}", style("❌ Clippy found issues:").red());
        println!("{}", clippy_result.output);
    } else {
        println!("{}", style("✅ Clippy validation passed!").green());
    }

    // Check documentation coverage
    println!();
    println!(
        "{}",
        style("📚 Checking documentation coverage...")
            .bold()
            .yellow()
    );
    match doc_coverage::check_documentation_coverage(&project_path).await {
        Ok(coverage) => {
            println!("{}", coverage.report());
            if coverage.coverage_percent < 80.0 {
                println!("{}", style("⚠️  Documentation coverage below 80%").yellow());
            }
        }
        Err(e) => {
            println!(
                "{}",
                style(format!("⚠️  Could not check documentation: {}", e)).yellow()
            );
        }
    }

    // Check formatting
    println!();
    println!(
        "{}",
        style("📝 Checking code formatting...").bold().yellow()
    );
    match formatting::check_formatting(&project_path).await {
        Ok(format_result) => {
            println!("{}", format_result.report());
        }
        Err(e) => {
            println!(
                "{}",
                style(format!("⚠️  Could not check formatting: {}", e)).yellow()
            );
        }
    }

    // Run security audit
    println!();
    println!("{}", style("🔒 Running security audit...").bold().yellow());
    match security::run_security_audit(&project_path).await {
        Ok(audit_report) => {
            println!("{}", audit_report.report());
        }
        Err(e) => {
            println!(
                "{}",
                style(format!("⚠️  Could not run security audit: {}", e)).yellow()
            );
        }
    }

    // Exit with error code if violations found
    if !violations.is_empty() || !clippy_result.success {
        std::process::exit(1);
    } else {
        println!();
        println!(
            "{}",
            style("🎉 All validations passed! Code meets Ferrous Forge standards.")
                .bold()
                .green()
        );
    }

    Ok(())
}

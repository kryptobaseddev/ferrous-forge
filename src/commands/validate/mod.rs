//! Validate command implementation

pub mod ai_report;
pub mod checks;
pub mod markdown;
pub mod utils;

use ai_report::generate_ai_report;
use checks::{run_additional_checks, run_clippy_validation};

use crate::{
    validation::{RustValidator, Violation},
    Result,
};
use console::style;
use std::path::PathBuf;

/// Execute the validate command
pub async fn execute(path: Option<PathBuf>, ai_report: bool) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    print_header(&project_path);

    let validator = RustValidator::new(project_path.clone())?;
    let violations = validator.validate_project().await?;

    display_validation_results(&validator, &violations)?;

    if ai_report {
        generate_ai_report_with_message(&project_path, &violations).await?;
    }

    let clippy_result = run_clippy_validation(&validator).await?;
    run_additional_checks(&project_path).await;

    handle_final_result(&violations, &clippy_result);

    Ok(())
}

fn print_header(project_path: &PathBuf) {
    println!();
    println!("{}", style("🦀 Running Ferrous Forge validation...").bold());
    println!();

    println!("📁 Project: {}", project_path.display());
    println!();
}

fn display_validation_results(validator: &RustValidator, violations: &[Violation]) -> Result<()> {
    let report = validator.generate_report(violations);

    println!("{}", report);

    Ok(())
}

async fn generate_ai_report_with_message(
    project_path: &PathBuf,
    violations: &[Violation],
) -> Result<()> {
    generate_ai_report(project_path, violations).await
}

fn handle_final_result(violations: &[Violation], clippy_result: &crate::validation::ClippyResult) {
    if !violations.is_empty() || !clippy_result.success {
        println!(
            "{}",
            style("❌ Validation completed with issues").red().bold()
        );
        std::process::exit(1);
    } else {
        println!(
            "{}",
            style("✅ All validation checks passed!").green().bold()
        );
    }
}

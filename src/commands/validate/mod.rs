//! Validate command implementation

pub mod ai_report;
pub mod checks;
pub mod markdown;
pub mod utils;

use ai_report::generate_ai_report;
use checks::{run_additional_checks, run_clippy_validation};

use crate::{
    Result,
    config::Config,
    validation::{RustValidator, Violation, ViolationType},
};
use console::style;
use std::path::{Path, PathBuf};

/// Execute the validate command
pub async fn execute(path: Option<PathBuf>, ai_report: bool, locked_only: bool) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    print_header(&project_path);

    // Load config so validators use project-configured limits and locked settings
    let config = Config::load_or_default().await?;
    let validator = RustValidator::with_config(project_path.clone(), config)?;
    let violations = validator.validate_project().await?;

    if locked_only {
        return handle_locked_only_check(&violations);
    }

    display_validation_results(&validator, &violations)?;

    if ai_report {
        generate_ai_report_with_message(&project_path, &violations).await?;
    }

    let clippy_result = run_clippy_validation(&validator).await?;
    run_additional_checks(&project_path).await;

    handle_final_result(&violations, &clippy_result);

    Ok(())
}

fn print_header(project_path: &Path) {
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

/// When --locked-only is set, only report and fail on locked setting violations
fn handle_locked_only_check(violations: &[Violation]) -> Result<()> {
    let locked: Vec<&Violation> = violations
        .iter()
        .filter(|v| {
            matches!(
                v.violation_type,
                ViolationType::WrongEdition
                    | ViolationType::OldRustVersion
                    | ViolationType::LockedSetting
            )
        })
        .collect();

    if locked.is_empty() {
        println!("{}", style("✅ No locked setting violations.").green());
        return Ok(());
    }

    eprintln!("\n❌ FERROUS FORGE — Locked Setting Violations\n");
    for v in &locked {
        eprintln!("{}\n", v.message);
    }
    std::process::exit(1);
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

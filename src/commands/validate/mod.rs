//! Validate command implementation

/// AI-friendly compliance report generation.
pub mod ai_report;
/// Validation check execution and result collection.
pub mod checks;
/// Markdown report formatting for validation results.
pub mod markdown;
/// Shared utilities for the validate command.
pub mod utils;

use ai_report::generate_ai_report;
use checks::{run_additional_checks, run_clippy_validation};

use crate::{
    Result,
    config::Config,
    validation::{RustValidator, Violation, ViolationType},
};
use console::style;
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Execute the validate command
///
/// # Errors
///
/// Returns an error if the configuration cannot be loaded, the validator
/// fails to initialize, or the validation process encounters an I/O error.
pub async fn execute(path: Option<PathBuf>, ai_report: bool, locked_only: bool) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Acquire process lock to prevent concurrent validation deadlocks.
    // Two simultaneous `ferrous-forge validate` invocations (e.g. from parallel
    // git commits in worktrees) would otherwise contend on cargo build locks
    // and stall indefinitely.
    let _lock_guard = match try_acquire_lock(&project_path) {
        Ok(Some(file)) => Some(file),
        Ok(None) => {
            println!(
                "{}",
                style("Another ferrous-forge validation is running for this project, skipping.")
                    .yellow()
            );
            return Ok(());
        }
        Err(e) => {
            // Lock acquisition failed unexpectedly; proceed without lock
            // rather than blocking the user's workflow.
            tracing::warn!("Failed to acquire process lock: {}", e);
            None
        }
    };

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

/// Try to acquire an exclusive process lock for validation.
///
/// Uses an advisory file lock keyed by a hash of the project path. The lock is
/// automatically released when the returned `File` handle is dropped (including
/// on abnormal process exit, since the OS releases `flock` locks).
///
/// Returns `Ok(Some(file))` if the lock was acquired, `Ok(None)` if another
/// instance is already validating this project, or `Err` on unexpected I/O
/// failures.
fn try_acquire_lock(project_path: &Path) -> std::io::Result<Option<std::fs::File>> {
    // Canonicalize so symlinks, trailing slashes, and relative paths all
    // resolve to the same lock file.
    let canonical = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let lock_path = std::env::temp_dir().join(format!("ferrous-forge-{}.lock", &hash[..16]));

    let file = std::fs::File::create(lock_path)?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(file)),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e),
    }
}

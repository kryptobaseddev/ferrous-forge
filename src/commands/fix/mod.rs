//! Auto-fix command for Ferrous Forge violations
#![allow(clippy::too_many_lines)]
//!
//! This module implements intelligent auto-fixing for common Rust anti-patterns.
//! It analyzes code context to ensure fixes are safe and won't break compilation.

mod context;
mod strategies;
mod types;
mod utils;

#[cfg(test)]
mod tests;

use context::analyze_file_context;
use strategies::fix_violation_in_line;
pub use types::{FileContext, FixConfig, FixResult, FunctionSignature};
use utils::{filter_violations, group_violations_by_file};

use crate::ai_analyzer;
use crate::validation::{RustValidator, Violation};
use crate::Result;
use anyhow::Context as AnyhowContext;
use console::style;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Execute the fix command
pub async fn execute(
    path: Option<PathBuf>,
    only: Option<String>,
    skip: Option<String>,
    dry_run: bool,
    limit: Option<usize>,
) -> Result<()> {
    execute_with_ai(path, only, skip, dry_run, limit, false).await
}

/// Execute the fix command with optional AI analysis
pub async fn execute_with_ai(
    path: Option<PathBuf>,
    only: Option<String>,
    skip: Option<String>,
    dry_run: bool,
    limit: Option<usize>,
    ai_analysis: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    
    print_startup_banner(&project_path, dry_run);
    
    let filter_options = parse_filter_options(only, skip);
    let filtered_violations = 
        validate_and_filter_violations(&project_path, &filter_options, limit).await?;
    
    if let Some(violations) = filtered_violations {
        execute_fix_process(&project_path, violations, ai_analysis, dry_run).await;
    }
    
    Ok(())
}

/// Configuration for filter options
struct FilterOptions {
    only_types: Option<HashSet<String>>,
    skip_types: Option<HashSet<String>>,
}

/// Statistics for fix operations
struct FixStats {
    total_fixed: usize,
    total_skipped: usize,
    files_modified: usize,
}

/// Validation results with original and filtered violations
struct ValidationResult {
    all: Vec<Violation>,
    filtered: Vec<Violation>,
}

/// Print startup banner with project information
fn print_startup_banner(project_path: &Path, dry_run: bool) {
    println!(
        "{}",
        style("🔧 Running Ferrous Forge auto-fix...").bold().cyan()
    );
    println!("📁 Project: {}", project_path.display());
    
    if dry_run {
        println!(
            "{}",
            style("ℹ️ Dry-run mode - no changes will be made").yellow()
        );
    }
}

/// Parse filter options from command line arguments
fn parse_filter_options(only: Option<String>, skip: Option<String>) -> FilterOptions {
    let only_types: Option<HashSet<String>> = only
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect());
    
    let skip_types: Option<HashSet<String>> = skip
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect());
    
    FilterOptions {
        only_types,
        skip_types,
    }
}

/// Validate the project and return violations
async fn validate_project(project_path: &Path) -> Result<Vec<Violation>> {
    let validator = RustValidator::new(project_path.to_path_buf())?;
    let violations = validator
        .validate_project()
        .await
        .with_context(|| format!("Failed to validate project at {}", project_path.display()))?;
    Ok(violations)
}

/// Validate project and filter violations, returning None if no violations to fix
async fn validate_and_filter_violations(
    project_path: &Path,
    filter_options: &FilterOptions,
    limit: Option<usize>,
) -> Result<Option<ValidationResult>> {
    let violations = validate_project(project_path).await?;
    
    if violations.is_empty() {
        println!(
            "{}",
            style("✨ No violations found - nothing to fix!").green()
        );
        return Ok(None);
    }
    
    let filtered_violations = filter_violations(
        &violations, 
        &filter_options.only_types, 
        &filter_options.skip_types, 
        limit
    );
    
    if filtered_violations.is_empty() {
        println!("{}", style("📝 No matching violations to fix").yellow());
        return Ok(None);
    }
    
    Ok(Some(ValidationResult {
        all: violations,
        filtered: filtered_violations,
    }))
}

/// Print summary of found violations
fn print_violations_summary(violations: &[Violation]) {
    println!(
        "{}",
        style(format!(
            "📊 Found {} potentially fixable violations",
            violations.len()
        ))
        .cyan()
    );
}

/// Run AI analysis if requested
async fn run_ai_analysis(project_path: &Path, violations: &[Violation]) {
    println!(
        "{}",
        style("🤖 Running AI-powered analysis...").bold().magenta()
    );
    
    if let Err(e) = ai_analyzer::analyze_and_generate_report(project_path, violations).await {
        eprintln!(
            "{}",
            style(format!("⚠️  AI analysis failed: {}", e)).yellow()
        );
    } else {
        println!("{}", style("✅ AI analysis complete").green());
        println!(
            "{}",
            style("   📊 Reports saved to .ferrous-forge/ai-analysis/").dim()
        );
    }
}

/// Print warning banner about experimental features
fn print_warning_banner() {
    println!();
    println!(
        "{}",
        style("⚠️  WARNING: Auto-fix is experimental!")
            .yellow()
            .bold()
    );
    println!(
        "{}",
        style("    Please review all changes and ensure your tests still pass.").yellow()
    );
}

/// Process all files and return fix statistics
fn process_all_files(
    violations_by_file: std::collections::HashMap<PathBuf, Vec<Violation>>,
    dry_run: bool,
) -> FixStats {
    let mut total_fixed = 0;
    let mut total_skipped = 0;
    let mut files_modified = 0;
    
    for (file_path, file_violations) in violations_by_file {
        match fix_file_violations(&file_path, &file_violations, dry_run) {
            Ok((fixed, skipped)) => {
                if fixed > 0 {
                    files_modified += 1;
                    if !dry_run {
                        println!(
                            "  {} Fixed {} violations in {}",
                            style("✅").green(),
                            fixed,
                            file_path.display()
                        );
                    }
                }
                if skipped > 0 && !dry_run {
                    println!(
                        "  {} Skipped {} unsafe fixes in {}",
                        style("⚠").yellow(),
                        skipped,
                        file_path.display()
                    );
                }
                total_fixed += fixed;
                total_skipped += skipped;
            }
            Err(e) => {
                eprintln!(
                    "  {} Failed to fix {}: {}",
                    style("❌").red(),
                    file_path.display(),
                    e
                );
            }
        }
    }
    
    FixStats {
        total_fixed,
        total_skipped,
        files_modified,
    }
}

/// Execute the main fix process for validated violations
async fn execute_fix_process(
    project_path: &Path,
    violations: ValidationResult,
    ai_analysis: bool,
    dry_run: bool,
) {
    print_violations_summary(&violations.filtered);
    
    if ai_analysis {
        run_ai_analysis(project_path, &violations.all).await;
    }
    
    print_warning_banner();
    
    let violations_by_file = group_violations_by_file(&violations.filtered);
    let fix_stats = process_all_files(violations_by_file, dry_run);
    
    print_final_summary(fix_stats, dry_run);
}

/// Print final summary of fix operations
fn print_final_summary(stats: FixStats, dry_run: bool) {
    println!();
    println!("{}", style("─".repeat(50)).dim());
    
    if dry_run {
        println!(
            "{}",
            style(format!("📝 Would fix {} violations safely", stats.total_fixed)).green()
        );
        if stats.total_skipped > 0 {
            println!(
                "{}",
                style(format!("⚠️ Would skip {} unsafe fixes", stats.total_skipped)).yellow()
            );
        }
    } else {
        if stats.total_fixed > 0 {
            println!(
                "{}",
                style(format!(
                    "✅ Fixed {} violations in {} files",
                    stats.total_fixed, stats.files_modified
                ))
                .green()
                .bold()
            );
        } else {
            println!("{}", style("No violations were auto-fixed").yellow());
        }
        
        if stats.total_skipped > 0 {
            println!(
                "{}",
                style(format!("⚠️ Skipped {} unsafe fixes", stats.total_skipped)).yellow()
            );
            println!();
            println!(
                "{}",
                style("💡 Tip: Review skipped fixes manually or use AI analysis").dim()
            );
        }
    }
}

/// Fix violations in a single file
fn fix_file_violations(
    file_path: &Path,
    violations: &[Violation],
    dry_run: bool,
) -> Result<(usize, usize)> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let context = analyze_file_context(&content);
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut fixed_count = 0;
    let mut skipped_count = 0;

    // Process violations (already sorted in descending order by line number)
    for violation in violations {
        if violation.line == 0 || violation.line > lines.len() {
            continue;
        }

        let line_idx = violation.line - 1; // Convert to 0-indexed
        let original_line = &lines[line_idx];

        match fix_violation_in_line(original_line, violation, &context) {
            FixResult::Fixed(new_line) => {
                if !dry_run {
                    lines[line_idx] = new_line;
                }
                fixed_count += 1;
            }
            FixResult::Skipped(_reason) => {
                skipped_count += 1;
            }
            FixResult::NotApplicable => {}
        }
    }

    // Write the file back if we made changes and not in dry-run mode
    if fixed_count > 0 && !dry_run {
        let new_content = lines.join("\n");
        // Preserve trailing newline if original had one
        let final_content = if content.ends_with('\n') {
            format!("{}\n", new_content)
        } else {
            new_content
        };

        fs::write(file_path, final_content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
    }

    Ok((fixed_count, skipped_count))
}

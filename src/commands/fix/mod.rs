//! Auto-fix command for Ferrous Forge violations
//!
//! This module implements intelligent auto-fixing for common Rust anti-patterns.
//! It analyzes code context to ensure fixes are safe and won't break compilation.

mod types;
mod context;
mod strategies;
mod utils;

#[cfg(test)]
mod tests;

pub use types::{FileContext, FunctionSignature, FixResult, FixConfig};
use context::analyze_file_context;
use strategies::fix_violation_in_line;
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

    // Parse filter options
    let only_types: Option<HashSet<String>> = only
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect());

    let skip_types: Option<HashSet<String>> = skip
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect());

    // Create validator and run validation
    let validator = RustValidator::new(project_path.clone())?;
    let violations = validator
        .validate_project()
        .await
        .with_context(|| format!("Failed to validate project at {}", project_path.display()))?;

    if violations.is_empty() {
        println!(
            "{}",
            style("✨ No violations found - nothing to fix!").green()
        );
        return Ok(());
    }

    // Filter violations based on options
    let filtered_violations = filter_violations(&violations, &only_types, &skip_types, limit);

    if filtered_violations.is_empty() {
        println!(
            "{}",
            style("📝 No matching violations to fix").yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        style(format!(
            "📊 Found {} potentially fixable violations",
            filtered_violations.len()
        ))
        .cyan()
    );

    if ai_analysis {
        println!(
            "{}",
            style("🤖 Running AI-powered analysis...").bold().magenta()
        );
        
        // Run AI analysis
        if let Err(e) = 
            ai_analyzer::analyze_and_generate_report(&project_path, &violations).await
        {
            eprintln!("{}", style(format!("⚠️  AI analysis failed: {}", e)).yellow());
        } else {
            println!("{}", style("✅ AI analysis complete").green());
            println!("{}", style("   📊 Reports saved to .ferrous-forge/ai-analysis/").dim());
        }
    }

    println!();
    println!("{}", style("⚠️  WARNING: Auto-fix is experimental!")
        .yellow().bold());
    println!("{}", style("    Please review all changes and ensure your tests still pass.")
        .yellow());

    // Group violations by file
    let violations_by_file = group_violations_by_file(&filtered_violations);

    let mut total_fixed = 0;
    let mut total_skipped = 0;
    let mut files_modified = 0;

    // Process each file
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

    // Print summary
    println!();
    println!("{}", style("──────────────────────────────────────────────────").dim());
    
    if dry_run {
        println!(
            "{}",
            style(format!("📝 Would fix {} violations safely", total_fixed)).green()
        );
        if total_skipped > 0 {
            println!(
                "{}",
                style(format!("⚠️ Would skip {} unsafe fixes", total_skipped)).yellow()
            );
        }
    } else {
        if total_fixed > 0 {
            println!(
                "{}",
                style(format!(
                    "✅ Fixed {} violations in {} files",
                    total_fixed, files_modified
                ))
                .green()
                .bold()
            );
        } else {
            println!(
                "{}",
                style("No violations were auto-fixed").yellow()
            );
        }
        
        if total_skipped > 0 {
            println!(
                "{}",
                style(format!("⚠️ Skipped {} unsafe fixes", total_skipped)).yellow()
            );
            println!();
            println!(
                "{}",
                style("💡 Tip: Review skipped fixes manually or use AI analysis for guidance").dim()
            );
        }
    }

    Ok(())
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
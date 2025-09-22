//! Fix command execution logic

use super::file_processing::process_all_files;
use super::types::{FilterOptions, FixStats};
use super::utils::{filter_violations, group_violations_by_file};
use crate::ai_analyzer;
use crate::validation::{RustValidator, Violation};
use crate::Result;
use console::style;
use std::collections::HashSet;
use std::path::Path;

/// Execute the main fix process
pub async fn execute_fix_process(
    project_path: &Path,
    dry_run: bool,
    filter_options: FilterOptions,
    ai_mode: bool,
) -> Result<()> {
    // Validate project and get violations
    let violations = validate_project(project_path).await?;
    
    // Filter violations based on user preferences
    let filtered_violations = validate_and_filter_violations(
        violations,
        &filter_options,
        project_path
    ).await?;

    if filtered_violations.is_empty() {
        println!("✅ No violations found that can be auto-fixed!");
        return Ok(());
    }

    print_violations_summary(&filtered_violations);

    // Run AI analysis if requested
    if ai_mode {
        run_ai_analysis(project_path, &filtered_violations).await;
    }

    if !dry_run {
        print_warning_banner();
    }

    // Group violations by file for processing
    let violations_by_file = group_violations_by_file(&filtered_violations);
    
    // Process all files
    let stats = process_all_files(violations_by_file, dry_run);
    
    print_final_summary(stats, dry_run);
    
    Ok(())
}

/// Validate project and return violations
async fn validate_project(project_path: &Path) -> Result<Vec<Violation>> {
    let validator = RustValidator::new(project_path.to_path_buf())?;
    validator.validate_project().await
}

/// Validate and filter violations based on fix capabilities
async fn validate_and_filter_violations(
    violations: Vec<Violation>,
    filter_options: &FilterOptions,
    project_path: &Path,
) -> Result<Vec<Violation>> {
    // Filter violations that we can potentially fix
    let fixable_violations = filter_violations(
        &violations,
        &filter_options.only_types,
        &filter_options.skip_types,
        None // limit
    );
    
    if fixable_violations.is_empty() {
        println!("ℹ️  No fixable violations found with current filters.");
        return Ok(vec![]);
    }

    // For now, we only support specific violation types
    let supported_types = HashSet::from([
        "LINETOOLONG",
        "UNDERSCORE_BANDAID", 
        "UNWRAP_IN_PRODUCTION",
    ]);

    let filtered: Vec<_> = fixable_violations
        .into_iter()
        .filter(|v| {
            let violation_str = format!("{:?}", v.violation_type).to_uppercase();
            supported_types.contains(violation_str.as_str())
        })
        .collect();

    if filtered.is_empty() {
        println!("ℹ️  No violations of supported types found for auto-fixing.");
        println!("    Supported types: LINETOOLONG, UNDERSCORE_BANDAID, UNWRAP_IN_PRODUCTION");
    }

    Ok(filtered)
}

/// Print summary of violations to be fixed
fn print_violations_summary(violations: &[Violation]) {
    println!();
    println!("📋 Found {} violations that can be auto-fixed:", violations.len());
    
    let mut counts = std::collections::HashMap::new();
    for violation in violations {
        *counts.entry(&violation.violation_type).or_insert(0) += 1;
    }
    
    for (vtype, count) in counts {
        println!("   • {:?}: {} violations", vtype, count);
    }
    println!();
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

/// Print final summary of fix results
fn print_final_summary(stats: FixStats, dry_run: bool) {
    println!();
    println!("📊 Fix Summary:");
    
    if dry_run {
        println!("   {} violations would be fixed", stats.total_fixed);
        println!("   {} violations would be skipped", stats.total_skipped);
        println!("   {} files would be modified", stats.files_modified);
        println!();
        println!("💡 Run without --dry-run to apply these fixes");
    } else {
        println!("   {} violations fixed", stats.total_fixed);
        println!("   {} violations skipped", stats.total_skipped);
        println!("   {} files modified", stats.files_modified);
        
        if stats.total_fixed > 0 {
            println!();
            println!("🔄 Remember to run your tests to ensure everything still works!");
        }
    }
}
//! Auto-fix command for Ferrous Forge violations
#![allow(clippy::too_many_lines)]
//!
//! This module implements intelligent auto-fixing for common Rust anti-patterns.
//! It analyzes code context to ensure fixes are safe and won't break compilation.

mod context;
mod execution;
mod file_processing;
mod strategies;
mod types;
mod utils;


use execution::execute_fix_process;
pub use types::{FileContext, FilterOptions, FixConfig, FixResult, FunctionSignature};

use crate::Result;
use console::style;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Execute the fix command
pub async fn execute(
    path: Option<PathBuf>,
    only: Option<String>,
    skip: Option<String>,
    dry_run: bool,
    _limit: Option<usize>,
) -> Result<()> {
    execute_with_ai(path, only, skip, dry_run, _limit, false).await
}

/// Execute the fix command with optional AI analysis
pub async fn execute_with_ai(
    path: Option<PathBuf>,
    only: Option<String>,
    skip: Option<String>,
    dry_run: bool,
    _limit: Option<usize>,
    ai_analysis: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    print_startup_banner(&project_path, dry_run);

    let filter_options = parse_filter_options(only, skip);
    execute_fix_process(&project_path, dry_run, filter_options, ai_analysis).await
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

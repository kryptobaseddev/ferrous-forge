//! File processing functionality for fix command

use super::types::FixStats;
use crate::validation::Violation;
use crate::{Error, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Process all files and return fix statistics
pub fn process_all_files(
    violations_by_file: HashMap<PathBuf, Vec<Violation>>,
    dry_run: bool,
) -> FixStats {
    let mut stats = FixStats {
        total_fixed: 0,
        total_skipped: 0,
        files_modified: 0,
    };
    
    for (file_path, file_violations) in violations_by_file {
        process_single_file(&file_path, &file_violations, dry_run, &mut stats);
    }
    
    stats
}

/// Process a single file's violations
fn process_single_file(
    file_path: &PathBuf,
    file_violations: &[Violation],
    dry_run: bool,
    stats: &mut FixStats,
) {
    println!("🔧 Processing: {}", file_path.display());
    
    match fix_file_violations(file_path, file_violations, dry_run) {
        Ok(fixed_count) if fixed_count > 0 => {
            handle_successful_fix(file_path, fixed_count, dry_run, stats);
        }
        Ok(_) => {
            println!("   ⚠️  No fixes applied");
            stats.total_skipped += file_violations.len();
        }
        Err(e) => {
            eprintln!("   ❌ Failed to fix file: {}", e);
            stats.total_skipped += file_violations.len();
        }
    }
}

/// Handle successful fix results
fn handle_successful_fix(
    _file_path: &PathBuf,
    fixed_count: usize,
    dry_run: bool,
    stats: &mut FixStats,
) {
    if dry_run {
        println!("   ✅ Would fix {} violations", fixed_count);
    } else {
        println!("   ✅ Fixed {} violations", fixed_count);
        stats.files_modified += 1;
    }
    stats.total_fixed += fixed_count;
}

/// Fix violations in a specific file
fn fix_file_violations(
    file_path: &PathBuf,
    violations: &[Violation],
    dry_run: bool,
) -> Result<usize> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| Error::validation(
            format!("Failed to read file {}: {}", file_path.display(), e)
        ))?;

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let preserve_trailing_newline = content.ends_with('\n');

    let fixed_count = apply_fixes_to_lines(&mut lines, violations);

    if fixed_count > 0 && !dry_run {
        write_fixed_content(file_path, &lines, preserve_trailing_newline)?;
    }

    Ok(fixed_count)
}

/// Apply fixes to the lines of code
fn apply_fixes_to_lines(lines: &mut [String], violations: &[Violation]) -> usize {
    let mut fixed_count = 0;

    // Sort violations by line number in descending order to avoid offset issues
    let mut sorted_violations = violations.to_vec();
    sorted_violations.sort_by(|a, b| b.line.cmp(&a.line));

    for violation in sorted_violations {
        if violation.line > 0 && violation.line <= lines.len() {
            let line_index = violation.line - 1;
            let original_line = &lines[line_index];

            // For now, use a simple fix approach
            if let Some(fixed_line) = try_simple_fix(original_line, &violation) {
                if fixed_line != *original_line {
                    lines[line_index] = fixed_line;
                    fixed_count += 1;
                }
            }
        }
    }

    fixed_count
}

/// Simple fix attempt for basic violations
fn try_simple_fix(_line: &str, violation: &Violation) -> Option<String> {
    match violation.violation_type {
        crate::validation::ViolationType::LineTooLong => {
            // For now, don't auto-fix line too long
            None
        }
        _ => None,
    }
}

/// Write the fixed content back to the file
fn write_fixed_content(
    file_path: &Path,
    lines: &[String],
    preserve_trailing_newline: bool,
) -> Result<()> {
    let new_content = lines.join("\n");
    let final_content = if preserve_trailing_newline {
        format!("{}\n", new_content)
    } else {
        new_content
    };

    fs::write(file_path, final_content)
        .map_err(|e| Error::validation(
            format!("Failed to write file {}: {}", file_path.display(), e)
        ))?;
    Ok(())
}
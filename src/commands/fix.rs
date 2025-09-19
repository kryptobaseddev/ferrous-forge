//! Auto-fix command for Ferrous Forge violations

use crate::validation::{RustValidator, Violation, ViolationType};
use crate::Result;
use anyhow::Context;
use console::style;
use std::collections::{HashMap, HashSet};
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
    let only_types: Option<HashSet<String>> =
        only.map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect());

    let skip_types: Option<HashSet<String>> =
        skip.map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect());

    // Create validator and run validation
    let validator = RustValidator::new(project_path.clone())?;
    let violations = validator.validate_project().await?;

    // Filter violations based on options
    let violations_to_fix = filter_violations(&violations, &only_types, &skip_types, limit);

    if violations_to_fix.is_empty() {
        println!("{}", style("✅ No violations to fix!").green());
        return Ok(());
    }

    println!("📊 Found {} violations to fix", violations_to_fix.len());

    // Group violations by file for efficient fixing
    let violations_by_file = group_violations_by_file(&violations_to_fix);

    let mut fixed_count = 0;
    let mut failed_count = 0;

    for (file_path, file_violations) in violations_by_file {
        match fix_file_violations(&file_path, &file_violations, dry_run) {
            Ok(count) => {
                fixed_count += count;
                if count > 0 && !dry_run {
                    println!(
                        "  {} Fixed {} violations in {}",
                        style("✓").green(),
                        count,
                        file_path.display()
                    );
                }
            }
            Err(e) => {
                failed_count += file_violations.len();
                eprintln!(
                    "  {} Failed to fix {}: {}",
                    style("✗").red(),
                    file_path.display(),
                    e
                );
            }
        }
    }

    // Print summary
    println!("\n{}", "─".repeat(50));
    if dry_run {
        println!(
            "{} Would fix {} violations",
            style("📝").blue(),
            fixed_count
        );
    } else {
        println!("{} Fixed {} violations", style("✅").green(), fixed_count);
    }

    if failed_count > 0 {
        println!(
            "{} Failed to fix {} violations",
            style("❌").red(),
            failed_count
        );
    }

    Ok(())
}

fn filter_violations(
    violations: &[Violation],
    only_types: &Option<HashSet<String>>,
    skip_types: &Option<HashSet<String>>,
    limit: Option<usize>,
) -> Vec<Violation> {
    let mut filtered: Vec<Violation> = violations
        .iter()
        .filter(|v| {
            let violation_type = format!("{:?}", v.violation_type).to_uppercase();

            // Check only filter
            if let Some(only) = only_types {
                if !only.contains(&violation_type) {
                    return false;
                }
            }

            // Check skip filter
            if let Some(skip) = skip_types {
                if skip.contains(&violation_type) {
                    return false;
                }
            }

            // Only include violations we can auto-fix
            can_auto_fix(v)
        })
        .cloned()
        .collect();

    // Apply limit if specified
    if let Some(limit) = limit {
        filtered.truncate(limit);
    }

    filtered
}

fn can_auto_fix(violation: &Violation) -> bool {
    match violation.violation_type {
        ViolationType::UnwrapInProduction => true,
        ViolationType::UnderscoreBandaid => true,
        ViolationType::LineTooLong => false, // Manual intervention needed
        ViolationType::FunctionTooLarge => false, // Requires refactoring
        ViolationType::FileTooLarge => false, // Requires splitting
        _ => false,
    }
}

fn group_violations_by_file(violations: &[Violation]) -> HashMap<PathBuf, Vec<Violation>> {
    let mut grouped: HashMap<PathBuf, Vec<Violation>> = HashMap::new();

    for violation in violations {
        grouped
            .entry(violation.file.clone())
            .or_insert_with(Vec::new)
            .push(violation.clone());
    }

    // Sort violations within each file by line number (reverse order for safe fixing)
    for file_violations in grouped.values_mut() {
        file_violations.sort_by(|a, b| b.line.cmp(&a.line));
    }

    grouped
}

fn fix_file_violations(
    file_path: &Path,
    violations: &[Violation],
    dry_run: bool,
) -> anyhow::Result<usize> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut fixed_count = 0;

    for violation in violations {
        if violation.line == 0 || violation.line > lines.len() {
            continue;
        }

        let line_idx = violation.line - 1;
        let original_line = lines[line_idx].clone();

        if let Some(fixed_line) = fix_violation_in_line(&original_line, violation) {
            if !dry_run {
                lines[line_idx] = fixed_line;
            }
            fixed_count += 1;
        }
    }

    if fixed_count > 0 && !dry_run {
        let fixed_content = lines.join("\n");
        // Preserve final newline if it existed
        let fixed_content = if content.ends_with('\n') {
            format!("{}\n", fixed_content)
        } else {
            fixed_content
        };
        fs::write(file_path, fixed_content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
    }

    Ok(fixed_count)
}

fn fix_violation_in_line(line: &str, violation: &Violation) -> Option<String> {
    match violation.violation_type {
        ViolationType::UnwrapInProduction => fix_unwrap_in_line(line),
        ViolationType::UnderscoreBandaid => fix_underscore_in_line(line),
        _ => None,
    }
}

fn fix_unwrap_in_line(line: &str) -> Option<String> {
    // Simple unwrap fixes - replace with ? operator where possible
    if line.contains(".unwrap()") {
        // Check if we're in a context that can return Result
        if !line.trim_start().starts_with("fn main")
            && !line.contains("panic!")
            && !line.contains("#[test]")
        {
            return Some(line.replace(".unwrap()", "?"));
        }
    }

    if line.contains(".expect(") {
        // For expect, we need to be more careful
        // Simple case: .context("message")? -> .context("message")?
        if let Some(start) = line.find(".expect(\"") {
            if let Some(end) = line[start + 9..].find("\")") {
                let message = &line[start + 9..start + 9 + end];
                let before = &line[..start];
                let after = &line[start + 9 + end + 2..];

                // Only fix if we can add anyhow::Context
                if !line.contains("panic!") && !line.trim_start().starts_with("fn main") {
                    return Some(format!("{}.context(\"{}\")?{}", before, message, after));
                }
            }
        }
    }

    None
}

fn fix_underscore_in_line(line: &str) -> Option<String> {
    // Fix underscore parameters in function signatures
    if line.contains("fn ") && line.contains('_') {
        // Find parameter patterns like: _config, _args, etc.
        // Use a simple approach to avoid regex dependency in this module
        let mut fixed = String::new();
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '_' {
                // Check if this is the start of an identifier
                if let Some(&next) = chars.peek() {
                    if next.is_ascii_lowercase() || next == '_' {
                        // Skip the underscore, this looks like _param
                        continue;
                    }
                }
            }
            fixed.push(ch);
        }

        if fixed != line {
            return Some(fixed);
        }
    }

    // Fix let _ = assignments
    if line.trim_start().starts_with("let _ =") {
        // Convert to proper error handling
        if let Some(rest) = line.trim_start().strip_prefix("let _ =") {
            let rest = rest.trim();
            if rest.ends_with(';') {
                let expr = &rest[..rest.len() - 1];
                let indent = &line[..line.find("let").unwrap_or(0)];
                // Add proper error handling
                return Some(format!("{}{}?;", indent, expr));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{Severity, ViolationType};

    #[test]
    fn test_fix_unwrap() {
        let line = "    let value = some_func().unwrap();";
        let violation = Violation {
            violation_type: ViolationType::UnwrapInProduction,
            file: PathBuf::from("test.rs"),
            line: 1,
            message: String::new(),
            severity: Severity::Error,
        };
        let fixed = fix_violation_in_line(line, &violation);
        assert_eq!(fixed, Some("    let value = some_func()?;".to_string()));
    }

    #[test]
    fn test_fix_expect() {
        let line = "    let value = some_func().expect(\"failed to get value\");";
        let violation = Violation {
            violation_type: ViolationType::UnwrapInProduction,
            file: PathBuf::from("test.rs"),
            line: 1,
            message: String::new(),
            severity: Severity::Error,
        };
        let fixed = fix_violation_in_line(line, &violation);
        assert_eq!(
            fixed,
            Some("    let value = some_func().context(\"failed to get value\")?;".to_string())
        );
    }

    #[test]
    fn test_fix_let_underscore() {
        let line = "    let _ = some_operation();";
        let violation = Violation {
            violation_type: ViolationType::UnderscoreBandaid,
            file: PathBuf::from("test.rs"),
            line: 1,
            message: String::new(),
            severity: Severity::Error,
        };
        let fixed = fix_violation_in_line(line, &violation);
        assert_eq!(fixed, Some("    some_operation()?;".to_string()));
    }
}

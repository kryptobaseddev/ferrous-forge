//! Code pattern validation
//!
//! Validates patterns that have no equivalent in native clippy/rustfmt:
//! - Underscore bandaid parameters (_unused, _param naming)
//! - Function size (config-driven, tracked via brace depth)
//!
//! NOTE: Line length is owned by rustfmt (`max_width`). Unwrap/expect are owned by
//! clippy lints injected via `ferrous-forge init --project`. This module does NOT
//! duplicate those checks.

use super::super::patterns::{ValidationPatterns, is_in_string_literal};
use crate::Result;
use crate::validation::{Severity, Violation, ViolationType};
use std::path::Path;

/// Validate code patterns that have no native clippy/rustfmt equivalent
pub fn validate_patterns(
    rust_file: &Path,
    lines: &[&str],
    patterns: &ValidationPatterns,
    violations: &mut Vec<Violation>,
    max_function_lines: usize,
) -> Result<()> {
    // Stack of (start_line, entry_brace_depth) for tracked functions
    let mut function_stack: Vec<(usize, usize)> = Vec::new();
    let mut brace_depth = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let open_braces = line.chars().filter(|&c| c == '{').count();
        let close_braces = line.chars().filter(|&c| c == '}').count();
        let depth_before = brace_depth;

        brace_depth = brace_depth.saturating_add(open_braces);
        brace_depth = brace_depth.saturating_sub(close_braces);

        // Check if any tracked functions ended on this line
        if close_braces > 0 {
            check_ended_functions(
                &mut function_stack,
                brace_depth,
                i,
                max_function_lines,
                rust_file,
                violations,
            );
        }

        // Track new function definitions
        if patterns.function_def.is_match(line) {
            function_stack.push((i, depth_before));
        }

        // Check for underscore bandaid coding
        check_underscore_bandaid(line, i, patterns, rust_file, violations);
    }

    // Handle functions still open at end of file
    check_unclosed_functions(
        &function_stack,
        lines.len(),
        max_function_lines,
        rust_file,
        violations,
    );

    Ok(())
}

/// Check if any tracked functions ended and emit violations for oversized ones
fn check_ended_functions(
    function_stack: &mut Vec<(usize, usize)>,
    brace_depth: usize,
    current_line: usize,
    max_function_lines: usize,
    rust_file: &Path,
    violations: &mut Vec<Violation>,
) {
    let mut ended = Vec::new();
    let mut remaining = Vec::new();
    for (start, entry_depth) in function_stack.drain(..) {
        if brace_depth <= entry_depth {
            ended.push((start, entry_depth));
        } else {
            remaining.push((start, entry_depth));
        }
    }
    *function_stack = remaining;

    for (start, _) in ended {
        let func_lines = current_line - start + 1;
        if func_lines > max_function_lines {
            violations.push(Violation {
                violation_type: ViolationType::FunctionTooLarge,
                file: rust_file.to_path_buf(),
                line: start + 1,
                message: format!(
                    "Function has {} lines, maximum allowed is {}",
                    func_lines, max_function_lines
                ),
                severity: Severity::Error,
            });
        }
    }
}

/// Check a line for underscore bandaid parameter patterns
fn check_underscore_bandaid(
    line: &str,
    line_index: usize,
    patterns: &ValidationPatterns,
    rust_file: &Path,
    violations: &mut Vec<Violation>,
) {
    if !patterns.underscore_param.is_match(line) {
        return;
    }

    let line_stripped = line.trim();
    let has_underscore = line.contains("_unused") || line.contains("_param");

    let not_in_unused_literal =
        !is_in_string_literal(line, "_unused") && !line.contains("r\"") && !line.contains("r#\"");
    let not_in_param_literal = !is_in_string_literal(line, "_param");

    let is_test_content =
        line.contains("test_function") || line.contains("test_module") || line.contains("assert!");

    let is_comment = line_stripped.starts_with("//");

    if has_underscore
        && not_in_unused_literal
        && not_in_param_literal
        && !is_test_content
        && !is_comment
    {
        violations.push(Violation {
            violation_type: ViolationType::UnderscoreBandaid,
            file: rust_file.to_path_buf(),
            line: line_index + 1,
            message: "BANNED: Underscore parameter (_param) - \
                     fix the design instead of hiding warnings"
                .to_string(),
            severity: Severity::Error,
        });
    }
}

/// Emit violations for functions still open at end of file
fn check_unclosed_functions(
    function_stack: &[(usize, usize)],
    total_lines: usize,
    max_function_lines: usize,
    rust_file: &Path,
    violations: &mut Vec<Violation>,
) {
    for (start, _) in function_stack {
        let func_lines = total_lines - start;
        if func_lines > max_function_lines {
            violations.push(Violation {
                violation_type: ViolationType::FunctionTooLarge,
                file: rust_file.to_path_buf(),
                line: start + 1,
                message: format!(
                    "Function has {} lines, maximum allowed is {}",
                    func_lines, max_function_lines
                ),
                severity: Severity::Error,
            });
        }
    }
}

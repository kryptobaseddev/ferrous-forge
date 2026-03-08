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

        // Check if any tracked functions ended on this line (brace depth returned to entry level)
        if close_braces > 0 {
            let mut ended = Vec::new();
            let mut remaining = Vec::new();
            for (start, entry_depth) in function_stack.drain(..) {
                if brace_depth <= entry_depth {
                    ended.push((start, entry_depth));
                } else {
                    remaining.push((start, entry_depth));
                }
            }
            function_stack = remaining;

            for (start, _) in ended {
                let func_lines = i - start + 1;
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

        // Track new function definitions (record entry depth = depth before this line's braces)
        if patterns.function_def.is_match(line) {
            function_stack.push((i, depth_before));
        }

        // Check for underscore bandaid coding (no clippy equivalent for _param/_unused naming)
        let line_stripped = line.trim();
        if patterns.underscore_param.is_match(line) {
            let has_underscore = line.contains("_unused") || line.contains("_param");

            let not_in_unused_literal = !is_in_string_literal(line, "_unused")
                && !line.contains("r\"")
                && !line.contains("r#\"");
            let not_in_param_literal = !is_in_string_literal(line, "_param");

            let is_test_content = line.contains("test_function")
                || line.contains("test_module")
                || line.contains("assert!");

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
                    line: i + 1,
                    message: "BANNED: Underscore parameter (_param) - \
                             fix the design instead of hiding warnings"
                        .to_string(),
                    severity: Severity::Error,
                });
            }
        }
    }

    // Handle functions still open at end of file (e.g., no closing brace — rare but handle it)
    for (start, _) in &function_stack {
        let func_lines = lines.len() - start;
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

    Ok(())
}

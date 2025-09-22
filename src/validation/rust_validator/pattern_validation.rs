//! Code pattern validation

use super::patterns::{is_in_string_literal, ValidationPatterns};
use crate::validation::{Severity, Violation, ViolationType};
use crate::Result;
use std::path::Path;

/// Legacy large function that contains the main validation logic
pub fn validate_patterns(
    rust_file: &Path,
    lines: &[&str],
    patterns: &ValidationPatterns,
    violations: &mut Vec<Violation>,
    is_test_file: bool,
    allow_unwrap: bool,
    allow_expect: bool,
) -> Result<()> {
    let mut in_test_block = false;
    let mut current_function_start: Option<usize> = None;
    let mut next_function_is_test = false;
    let mut in_test_module = false;
    let mut next_module_is_test = false;
    let mut brace_depth = 0;
    let mut test_module_depth = None;

    for (i, line) in lines.iter().enumerate() {
        let line_stripped = line.trim();

        // Track brace depth to properly scope test modules
        brace_depth += line.chars().filter(|&c| c == '{').count();
        brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());

        // Reset test module when we exit its scope
        if let Some(depth) = test_module_depth {
            if brace_depth <= depth {
                in_test_module = false;
                test_module_depth = None;
            }
        }

        // Check for test module markers
        if line_stripped.contains("#[cfg(test)]") {
            next_module_is_test = true;
        }

        // Check if we're entering a test module
        if (line_stripped.starts_with("mod tests") || 
            (line_stripped.starts_with("mod ") && line_stripped.contains("test"))) && 
           next_module_is_test {
            in_test_module = true;
            test_module_depth = Some(brace_depth);
            next_module_is_test = false;
        }
        
        // Also handle case where #[cfg(test)] and mod are on same line
        if line_stripped.contains("#[cfg(test)]") && line_stripped.contains("mod") {
            in_test_module = true;
            test_module_depth = Some(brace_depth);
        }

        // Check for test attributes that apply to the next function
        if line_stripped.contains("#[test]")
            || line_stripped.contains("#[tokio::test]")
            || line_stripped.contains("#[bench]")
        {
            next_function_is_test = true;
        }

        // Track function boundaries and check function size
        if patterns.function_def.is_match(line) {
            // Check previous function size
            if let Some(start) = current_function_start {
                let func_lines = i - start;
                if func_lines > 50 {
                    violations.push(Violation {
                        violation_type: ViolationType::FunctionTooLarge,
                        file: rust_file.to_path_buf(),
                        line: start + 1,
                        message: format!(
                            "Function has {} lines, maximum allowed is 50",
                            func_lines
                        ),
                        severity: Severity::Error,
                    });
                }
            }

            // Set test block state based on whether this function is a test
            in_test_block = next_function_is_test;
            next_function_is_test = false; // Reset for next function
            current_function_start = Some(i);
        }

        // Check for underscore bandaid coding
        if patterns.underscore_param.is_match(line) {
            let has_underscore = line.contains("_unused") || line.contains("_param");

            // Enhanced string detection
            let not_in_unused_literal = !is_in_string_literal(line, "_unused")
                && !line.contains("r\"") 
                && !line.contains("r#\"");
            let not_in_param_literal = !is_in_string_literal(line, "_param");

            // Check if it's actually test content (ignore false positives)
            let is_test_content = line.contains("test_function")
                || line.contains("test_module")
                || line.contains("assert!");

            if has_underscore && not_in_unused_literal && not_in_param_literal && !is_test_content {
                violations.push(Violation {
                    violation_type: ViolationType::UnderscoreBandaid,
                    file: rust_file.to_path_buf(),
                    line: i + 1,
                    message: "BANNED: Underscore parameter (_param) - fix the design instead of hiding warnings"
                        .to_string(),
                    severity: Severity::Error,
                });
            }
        }

        // Check for unwrap in production code (not in tests or if allowed)
        if !in_test_block
            && !in_test_module
            && !is_test_file
            && !allow_unwrap
            && !line_stripped.starts_with("//")
            && patterns.unwrap_call.is_match(line)
            && !is_in_string_literal(line, ".unwrap()")
        {
            violations.push(Violation {
                violation_type: ViolationType::UnwrapInProduction,
                file: rust_file.to_path_buf(),
                line: i + 1,
                message: "BANNED: .unwrap() in production code - use proper error handling with ?"
                    .to_string(),
                severity: Severity::Error,
            });
        }

        // Check for expect in production code (not in tests or if allowed)
        if !in_test_block
            && !in_test_module
            && !is_test_file
            && !allow_expect
            && !line_stripped.starts_with("//")
            && patterns.expect_call.is_match(line)
            && !is_in_string_literal(line, ".expect(")
        {
            violations.push(Violation {
                violation_type: ViolationType::UnwrapInProduction,
                file: rust_file.to_path_buf(),
                line: i + 1,
                message: "BANNED: .expect() in production code - use proper error handling with ?"
                    .to_string(),
                severity: Severity::Error,
            });
        }

        // Check line lengths (100 character limit)
        if line.len() > 100 {
            violations.push(Violation {
                violation_type: ViolationType::LineTooLong,
                file: rust_file.to_path_buf(),
                line: i + 1,
                message: format!("Line has {} characters, maximum allowed is 100", line.len()),
                severity: Severity::Warning,
            });
        }
    }

    Ok(())
}
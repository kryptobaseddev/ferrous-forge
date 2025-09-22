//! File-level validation checks

use super::patterns::{is_in_string_literal, ValidationPatterns};
use crate::validation::{Severity, Violation, ViolationType};
use crate::Result;
use std::path::Path;
use tokio::fs;

/// Validates a Cargo.toml file for standards compliance
pub async fn validate_cargo_toml(cargo_file: &Path, violations: &mut Vec<Violation>) -> Result<()> {
    let content = fs::read_to_string(cargo_file).await?;
    let lines: Vec<&str> = content.lines().collect();

    // Check for Edition 2021 or 2024
    let mut edition_found = false;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("edition") {
            if !line.contains("2021") && !line.contains("2024") {
                violations.push(Violation {
                    violation_type: ViolationType::WrongEdition,
                    file: cargo_file.to_path_buf(),
                    line: i + 1,
                    message: "Must use Edition 2021 or 2024".to_string(),
                    severity: Severity::Error,
                });
            }
            edition_found = true;
            break;
        }
    }

    if !edition_found {
        violations.push(Violation {
            violation_type: ViolationType::WrongEdition,
            file: cargo_file.to_path_buf(),
            line: 0,
            message: "Missing edition specification - must be '2021' or '2024'".to_string(),
            severity: Severity::Error,
        });
    }

    Ok(())
}

/// Validates a Rust source file for standards compliance
pub async fn validate_rust_file(
    rust_file: &Path,
    violations: &mut Vec<Violation>,
    patterns: &ValidationPatterns,
) -> Result<()> {
    let content = fs::read_to_string(rust_file).await?;
    let lines: Vec<&str> = content.lines().collect();

    // Check if this is a test or benchmark file
    let path_str = rust_file.to_string_lossy();
    let is_test_file = path_str.contains("/tests/")
        || path_str.contains("/benches/")
        || path_str.contains("/test_")
        || path_str.ends_with("_test.rs")
        || path_str.ends_with("_bench.rs")
        || path_str.ends_with("/tests.rs");

    // Check for allow attributes at the top of the file
    let mut allow_unwrap = false;
    let mut allow_expect = false;

    for line in &lines {
        let line_stripped = line.trim();
        if line_stripped.starts_with("#![allow(") {
            if line_stripped.contains("clippy::unwrap_used") {
                allow_unwrap = true;
            }
            if line_stripped.contains("clippy::expect_used") {
                allow_expect = true;
            }
        }
        // Also check for allow attributes that might be split across lines
        if line_stripped.contains("clippy::unwrap_used") {
            allow_unwrap = true;
        }
        if line_stripped.contains("clippy::expect_used") {
            allow_expect = true;
        }
        // Stop checking after we hit the first non-attribute line (but allow doc comments)
        if !line_stripped.starts_with("#")
            && !line_stripped.starts_with("//!")
            && !line_stripped.is_empty()
        {
            break;
        }
    }

    // Check file size limit (400 lines)
    if lines.len() > 400 {
        violations.push(Violation {
            violation_type: ViolationType::FileTooLarge,
            file: rust_file.to_path_buf(),
            line: lines.len() - 1,
            message: format!("File has {} lines, maximum allowed is 400", lines.len()),
            severity: Severity::Error,
        });
    }

    // Check line lengths (100 character limit)
    for (i, line) in lines.iter().enumerate() {
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

    let mut in_test_block = false;
    let mut current_function_start: Option<usize> = None;
    let mut next_function_is_test = false;
    let mut in_test_module = false;
    let mut next_module_is_test = false;

    for (i, line) in lines.iter().enumerate() {
        let line_stripped = line.trim();

        // Check for test module markers
        if line_stripped.contains("#[cfg(test)]") {
            next_module_is_test = true;
            next_function_is_test = true;
        }

        // Check if we're entering a test module
        if line_stripped.starts_with("mod tests") && next_module_is_test {
            in_test_module = true;
            next_module_is_test = false;
        }

        // Check for test attributes that apply to the next function
        if line_stripped.contains("#[test]")
            || line_stripped.contains("#[tokio::test]")
            || line_stripped.contains("#[bench]")
        {
            next_function_is_test = true;
        }

        // Track function boundaries
        if patterns.function_def.is_match(line) {
            // Check previous function size
            if let Some(start) = current_function_start {
                let func_lines = i - start;
                if func_lines > 230 {
                    violations.push(Violation {
                        violation_type: ViolationType::FunctionTooLarge,
                        file: rust_file.to_path_buf(),
                        line: start + 1,
                        message: format!(
                            "Function has {} lines, maximum allowed is 70",
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
        // Only check if we found an underscore parameter pattern
        if patterns.underscore_param.is_match(line) {
            // Check if the entire match is within a string literal
            // We need to check for common underscore patterns
            let has_underscore = line.contains("_unused") || line.contains("_param");
            
            // Only flag if it's not in a string literal or comment
            let not_in_unused_literal = !is_in_string_literal(line, "_unused");
            let not_in_param_literal = !is_in_string_literal(line, "_param");
            if has_underscore && not_in_unused_literal && not_in_param_literal {
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

        // Check let _ = patterns more carefully
        if patterns.underscore_let.is_match(line) {
            // Only flag if not in string literal and not a valid drop pattern
            if !is_in_string_literal(line, "let _") {
                // Some valid uses of let _ = :
                // - Dropping guards/locks explicitly
                // - Pattern matching where we don't care about some values
                // For now, flag all of them and require explicit handling
                violations.push(Violation {
                    violation_type: ViolationType::UnderscoreBandaid,
                    file: rust_file.to_path_buf(),
                    line: i + 1,
                    message: "BANNED: Underscore assignment (let _ =) - handle errors properly"
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

        // Reset test block tracking
        if line_stripped.starts_with('}') && in_test_block {
            in_test_block = false;
        }
    }

    // Check the last function if any
    if let Some(start) = current_function_start {
        let func_lines = lines.len() - start;
        if func_lines > 230 {
            violations.push(Violation {
                violation_type: ViolationType::FunctionTooLarge,
                file: rust_file.to_path_buf(),
                line: start,
                message: format!("Function has {} lines, maximum allowed is 70", func_lines),
                severity: Severity::Error,
            });
        }
    }

    Ok(())
}

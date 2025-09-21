//! Fix strategies for different violation types
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::context::check_can_use_question_mark;
use super::types::{FileContext, FixResult};
use crate::validation::{Violation, ViolationType};

/// Fix a violation in a line of code
pub fn fix_violation_in_line(
    line: &str,
    violation: &Violation,
    context: &FileContext,
) -> FixResult {
    match violation.violation_type {
        ViolationType::UnwrapInProduction => fix_unwrap_in_line(line, violation, context),
        ViolationType::UnderscoreBandaid => fix_underscore_in_line(line, violation, context),
        _ => FixResult::NotApplicable,
    }
}

/// Fix unwrap violations in a line
fn fix_unwrap_in_line(line: &str, violation: &Violation, context: &FileContext) -> FixResult {
    // Skip test files
    if context.is_test_file {
        return FixResult::Skipped(format!(
            "Test file - manual review needed at {}:{}",
            violation.file.display(),
            violation.line
        ));
    }

    if line.contains(".unwrap()") {
        // Don't fix if it's in a string literal
        if line.contains(r#"".unwrap()""#) || line.contains(r#"'.unwrap()'"#) {
            return FixResult::Skipped("String literal, not actual code".into());
        }

        // Check if we're in a function that can use ?
        let can_use_question_mark = check_can_use_question_mark(context);

        if can_use_question_mark {
            // Safe to replace with ?
            return FixResult::Fixed(line.replace(".unwrap()", "?"));
        } else {
            // For main functions or examples, use expect
            if context.is_bin_file || context.is_example_file {
                let fixed = line.replace(".unwrap()", r#".expect("Failed to complete operation")"#);
                return FixResult::Fixed(fixed);
            }
            return FixResult::Skipped(
                "Cannot use ? operator - function doesn't return Result/Option".to_string(),
            );
        }
    } else if line.contains(".expect(") {
        // For expect, we can potentially replace with ? if the context allows
        if check_can_use_question_mark(context) {
            // Find the expect call and replace it
            if let Some(start) = line.find(".expect(") {
                let before = &line[..start];

                // Find the matching closing parenthesis
                let after_expect = &line[start + 8..];
                let mut paren_count = 1;
                let mut end_idx = 0;
                let mut in_string = false;
                let mut escape_next = false;

                for (i, ch) in after_expect.chars().enumerate() {
                    if escape_next {
                        escape_next = false;
                        continue;
                    }

                    if ch == '\\' {
                        escape_next = true;
                        continue;
                    }

                    if ch == '"' {
                        in_string = !in_string;
                    }

                    if !in_string {
                        if ch == '(' {
                            paren_count += 1;
                        } else if ch == ')' {
                            paren_count -= 1;
                            if paren_count == 0 {
                                end_idx = i;
                                break;
                            }
                        }
                    }
                }

                if paren_count == 0 {
                    let after = &after_expect[end_idx + 1..];
                    let fixed = format!("{}?{}", before, after);
                    return FixResult::Fixed(fixed);
                }
            }
            return FixResult::Skipped("Complex expect pattern - manual review needed".to_string());
        } else {
            return FixResult::Skipped(
                "Cannot use ? operator - function doesn't return Result/Option".to_string(),
            );
        }
    }

    FixResult::NotApplicable
}

/// Fix underscore parameter violations
fn fix_underscore_in_line(line: &str, violation: &Violation, context: &FileContext) -> FixResult {
    // Skip test files
    if context.is_test_file {
        return FixResult::Skipped(format!(
            "Test file - manual review needed at {}:{}",
            violation.file.display(),
            violation.line
        ));
    }

    // Check if this is a simple underscore assignment that can be removed
    if line.trim().starts_with("let _") && line.contains("=") {
        // Extract the assignment to see if it's side-effect free
        if let Some(equals_pos) = line.find("=") {
            let value_part = line[equals_pos + 1..].trim();
            // Only remove if it looks like a simple value assignment
            if !value_part.contains("(") && !value_part.contains(".") {
                // This is likely a simple unused assignment, can be removed
                return FixResult::Fixed(String::new());
            }
        }
    }

    // For more complex cases, provide context-aware skip message
    FixResult::Skipped(format!(
        "Underscore at {}:{} requires manual review - may have side effects",
        violation.file.display(),
        violation.line
    ))
}

/// Check if a violation can potentially be auto-fixed
pub fn can_potentially_auto_fix(violation: &Violation) -> bool {
    matches!(
        violation.violation_type,
        ViolationType::UnwrapInProduction | ViolationType::UnderscoreBandaid
    )
}

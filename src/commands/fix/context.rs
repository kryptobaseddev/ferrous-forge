//! File context analysis for intelligent fixes

use super::types::{FileContext, FunctionSignature};

/// Analyze a file's content to understand its context
#[allow(dead_code)]
pub fn analyze_file_context(content: &str) -> FileContext {
    let is_test_file = content.contains("#[test]")
        || content.contains("#[cfg(test)]")
        || content.contains("mod tests");

    let is_bin_file = content.contains("fn main()");
    let is_example_file = content.contains("//! Example") || content.contains("// Example");

    // Parse function signatures
    let mut function_signatures = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ")
        {
            if let Some(sig) = parse_function_signature_multiline(&lines, i) {
                function_signatures.push(sig);
            }
        }
    }

    FileContext {
        is_test_file,
        is_bin_file,
        is_example_file,
        function_signatures,
    }
}

/// Parse a function signature that may span multiple lines
#[allow(dead_code)]
pub fn parse_function_signature_multiline(
    lines: &[&str],
    start_idx: usize,
) -> Option<FunctionSignature> {
    let (full_signature, brace_line) = collect_signature_lines(lines, start_idx)?;
    let name = extract_function_name(&full_signature)?;
    let (returns_result, returns_option) = check_return_types(&full_signature);
    let line_end = find_function_end(lines, brace_line, start_idx);

    Some(FunctionSignature {
        name,
        line_start: start_idx + 1, // Convert to 1-indexed
        line_end: line_end + 1,    // Convert to 1-indexed
        returns_result,
        returns_option,
    })
}

/// Collect signature lines until the opening brace is found
#[allow(dead_code)]
fn collect_signature_lines(lines: &[&str], start_idx: usize) -> Option<(String, usize)> {
    let mut brace_line = start_idx;
    let mut signature_lines = vec![lines[start_idx].to_string()];

    // Collect lines until we find the opening brace
    while brace_line < lines.len() && !lines[brace_line].contains('{') {
        if brace_line > start_idx {
            signature_lines.push(lines[brace_line].to_string());
        }
        brace_line += 1;
    }

    // Join all signature lines
    let full_signature = signature_lines.join(" ");
    Some((full_signature, brace_line))
}

/// Extract the function name from the full signature
#[allow(dead_code)]
fn extract_function_name(full_signature: &str) -> Option<String> {
    let name_start = full_signature.find("fn ")?;
    let name_part = &full_signature[name_start + 3..];
    let end = name_part.find(['(', '<'])?;
    Some(name_part[..end].trim().to_string())
}

/// Check what types the function returns
#[allow(dead_code)]
fn check_return_types(full_signature: &str) -> (bool, bool) {
    let returns_result = full_signature.contains("-> Result")
        || full_signature.contains("-> anyhow::Result")
        || full_signature.contains("-> std::result::Result")
        || full_signature.contains("-> io::Result");

    let returns_option = full_signature.contains("-> Option");

    (returns_result, returns_option)
}

/// Find the end of the function (closing brace at the same indentation level)
fn find_function_end(lines: &[&str], brace_line: usize, start_idx: usize) -> usize {
    let indent = lines[start_idx].len() - lines[start_idx].trim_start().len();
    let mut line_end = brace_line + 1;
    let mut brace_count = 1;

    while line_end < lines.len() && brace_count > 0 {
        let line = lines[line_end];
        let line_indent = line.len() - line.trim_start().len();

        for ch in line.chars() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                // Check if this is the closing brace at the same indentation
                if brace_count == 0 && line_indent == indent {
                    break;
                }
            }
        }
        if brace_count == 0 {
            break;
        }
        line_end += 1;
    }

    line_end
}

/// Check if the ? operator can be used in this context
pub fn check_can_use_question_mark(context: &FileContext) -> bool {
    // Don't use ? in test functions
    if context.is_test_file {
        return false;
    }

    // Don't use ? in main functions (unless they return Result)
    if context.is_bin_file {
        // Check if main returns Result
        for sig in &context.function_signatures {
            if sig.name == "main" && sig.returns_result {
                return true;
            }
        }
        return false;
    }

    // Don't use ? in example files (they often have simple main functions)
    if context.is_example_file {
        return false;
    }

    // For other cases, check if we're in a function that returns Result or Option
    // This is a simplified check - in reality we'd need to know which function we're in
    !context.function_signatures.is_empty()
        && context
            .function_signatures
            .iter()
            .any(|sig| sig.returns_result || sig.returns_option)
}

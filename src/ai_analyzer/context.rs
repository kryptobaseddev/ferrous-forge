use super::types::{CodeContext, ErrorHandlingStyle};

/// Extract code context around a violation line
pub fn extract_code_context(line_number: usize, content: &str) -> CodeContext {
    let lines: Vec<&str> = content.lines().collect();
    let context_start = line_number.saturating_sub(10);
    let context_end = (line_number + 10).min(lines.len());

    let surrounding_code = lines[context_start..context_end]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let imports = extract_imports(content);
    let (function_name, function_signature, return_type) =
        extract_function_info(&lines, line_number);
    let is_async = function_signature
        .as_ref()
        .map(|s| s.contains("async"))
        .unwrap_or(false);
    let is_generic = function_signature
        .as_ref()
        .map(|s| s.contains('<'))
        .unwrap_or(false);
    let trait_impl = detect_trait_impl(content, line_number);
    let error_handling_style = detect_error_handling_style(&imports, content);

    CodeContext {
        function_name,
        function_signature,
        return_type,
        is_async,
        is_generic,
        trait_impl,
        surrounding_code,
        imports,
        error_handling_style,
    }
}

fn extract_imports(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.trim().starts_with("use "))
        .map(|s| s.to_string())
        .collect()
}

fn extract_function_info(
    lines: &[&str],
    line_number: usize,
) -> (Option<String>, Option<String>, Option<String>) {
    for i in (0..line_number).rev() {
        if let Some(line) = lines.get(i) {
            if line.contains("fn ") && !line.trim().starts_with("//") {
                let signature = line.trim().to_string();
                let name = signature
                    .split("fn ")
                    .nth(1)
                    .and_then(|s| s.split('(').next())
                    .map(|s| s.trim().to_string());
                let return_type = if signature.contains("->") {
                    signature.split("->").nth(1).map(|s| s.trim().to_string())
                } else {
                    None
                };
                return (name, Some(signature), return_type);
            }
        }
    }
    (None, None, None)
}

/// Detect the error handling style used in the code
pub fn detect_error_handling_style(imports: &[String], content: &str) -> ErrorHandlingStyle {
    if imports.iter().any(|i| i.contains("anyhow")) || content.contains("anyhow::Result") {
        ErrorHandlingStyle::AnyhowResult
    } else if content.contains("Result<") && !content.contains("std::result::Result") {
        ErrorHandlingStyle::CustomResult
    } else if content.contains("Result<") {
        ErrorHandlingStyle::StdResult
    } else if content.contains("Option<") {
        ErrorHandlingStyle::OptionBased
    } else if content.contains("panic!") || content.contains(".unwrap()") {
        ErrorHandlingStyle::Panic
    } else {
        ErrorHandlingStyle::Unknown
    }
}

fn detect_trait_impl(content: &str, line: usize) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    for i in (0..line.min(lines.len())).rev() {
        if lines[i].contains("impl") && lines[i].contains("for") {
            return Some(lines[i].trim().to_string());
        }
    }
    None
}

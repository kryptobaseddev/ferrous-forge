use std::collections::HashMap;

use super::types::{CodeContext, FixComplexity, SemanticAnalysis};
use crate::validation::{Violation, ViolationType};

/// Perform semantic analysis on a violation
pub fn perform_semantic_analysis(
    violation: &Violation,
    context: &CodeContext,
    content: &str,
) -> SemanticAnalysis {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = violation.line.saturating_sub(1);

    let actual_type = infer_actual_type(&lines, line_idx);
    let expected_type = infer_expected_type(&violation.violation_type);
    let data_flow = trace_data_flow(&lines, line_idx);
    let control_flow = trace_control_flow(&lines, line_idx);
    let dependencies = extract_dependencies(context);
    let error_propagation = trace_error_propagation(&lines, line_idx);

    SemanticAnalysis {
        actual_type,
        expected_type,
        data_flow,
        control_flow,
        dependencies,
        error_propagation,
    }
}

fn infer_actual_type(lines: &[&str], line_idx: usize) -> Option<String> {
    if line_idx < lines.len() {
        let line = lines[line_idx];
        if line.contains("Result<") {
            Some("Result".to_string())
        } else if line.contains("Option<") {
            Some("Option".to_string())
        } else if line.contains("Vec<") {
            Some("Vec".to_string())
        } else {
            None
        }
    } else {
        None
    }
}

fn infer_expected_type(violation_type: &ViolationType) -> Option<String> {
    match violation_type {
        ViolationType::UnwrapInProduction => Some("Result or Option".to_string()),
        ViolationType::UnderscoreBandaid
        | ViolationType::WrongEdition
        | ViolationType::FileTooLarge
        | ViolationType::FunctionTooLarge
        | ViolationType::LineTooLong
        | ViolationType::MissingDocs
        | ViolationType::MissingDependencies
        | ViolationType::OldRustVersion => None,
    }
}

fn trace_data_flow(lines: &[&str], line_idx: usize) -> Vec<String> {
    let mut flow = Vec::new();
    if line_idx < lines.len() {
        if let Some(var_name) = extract_variable_name(lines[line_idx]) {
            let usage = analyze_variable_usage(lines);
            if let Some(uses) = usage.get(&var_name) {
                for use_line in uses {
                    flow.push(format!(
                        "Line {}: Variable '{}' used",
                        use_line + 1,
                        var_name
                    ));
                }
            }
        }
    }
    flow
}

fn trace_control_flow(lines: &[&str], line_idx: usize) -> Vec<String> {
    let mut flow = Vec::new();
    // Include context around the violation line
    let start = line_idx.saturating_sub(5);
    let end = (line_idx + 5).min(lines.len());

    for (i, line) in lines.iter().enumerate().skip(start).take(end - start) {
        if line.contains("if ") || line.contains("match ") || line.contains("while ") {
            flow.push(format!("Line {}: Control flow statement", i + 1));
        }
    }
    flow
}

fn analyze_variable_usage(lines: &[&str]) -> HashMap<String, Vec<usize>> {
    let mut usage = HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(var) = extract_variable_name(line) {
            usage.entry(var).or_insert_with(Vec::new).push(i);
        }
    }
    usage
}

fn extract_variable_name(line: &str) -> Option<String> {
    if line.contains("let ") {
        line.split("let ")
            .nth(1)
            .and_then(|s| s.split('=').next())
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn trace_error_propagation(lines: &[&str], line_idx: usize) -> Vec<String> {
    let mut path = Vec::new();
    // Focus on error handling near the violation
    let start = line_idx.saturating_sub(10);
    let end = (line_idx + 10).min(lines.len());

    for (i, line) in lines.iter().enumerate().skip(start).take(end - start) {
        if line.contains('?') || line.contains(".unwrap()") || line.contains(".expect(") {
            path.push(format!("Line {}: Error handling point", i + 1));
        }
    }
    path
}

/// Assess the complexity of fixing a violation
pub fn assess_fix_complexity(
    violation: &Violation,
    context: &CodeContext,
    semantic: &SemanticAnalysis,
) -> FixComplexity {
    match violation.violation_type {
        ViolationType::UnwrapInProduction => {
            if context
                .return_type
                .as_ref()
                .is_some_and(|r| r.contains("Result"))
            {
                if semantic.dependencies.is_empty() {
                    FixComplexity::Trivial
                } else {
                    FixComplexity::Simple
                }
            } else {
                FixComplexity::Moderate
            }
        }
        ViolationType::UnderscoreBandaid => FixComplexity::Simple,
        ViolationType::LineTooLong => FixComplexity::Trivial,
        ViolationType::FunctionTooLarge => FixComplexity::Complex,
        ViolationType::FileTooLarge => FixComplexity::Architectural,
        _ => FixComplexity::Moderate,
    }
}

fn extract_dependencies(context: &CodeContext) -> Vec<String> {
    let mut deps = Vec::new();

    for import in &context.imports {
        if !import.contains("std::") {
            deps.push(import.clone());
        }
    }

    if context.trait_impl.is_some() {
        deps.push("Trait implementation".to_string());
    }

    if context.is_async {
        deps.push("Async context".to_string());
    }

    if context.is_generic {
        deps.push("Generic constraints".to_string());
    }

    deps
}

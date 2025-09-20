//! AI-Powered Code Analysis for Intelligent Fixing
//!
//! This module performs deep semantic analysis of code violations to generate
//! comprehensive context and instructions for AI-powered fixing through the
//! Claude Orchestrator or other LLM agents.

use crate::validation::{Violation, ViolationType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{parse_file, Item, ReturnType};

/// Comprehensive AI analysis report for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisReport {
    /// Metadata about the analysis
    pub metadata: AnalysisMetadata,
    /// Detailed violation analyses
    pub violation_analyses: Vec<ViolationAnalysis>,
    /// Code patterns and architectural insights
    pub code_patterns: CodePatterns,
    /// Suggested fix strategies with confidence scores
    pub fix_strategies: Vec<FixStrategy>,
    /// Instructions for AI/LLM agents
    pub ai_instructions: AIInstructions,
}

/// Metadata about the AI analysis run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Timestamp when the analysis was performed
    pub timestamp: String,
    /// Total number of violations analyzed
    pub total_violations: usize,
    /// Number of violations that could be analyzed by AI
    pub analyzable_violations: usize,
    /// Path to the project being analyzed
    pub project_path: String,
    /// Depth of analysis performed
    pub analysis_depth: AnalysisDepth,
}

/// Depth level of AI analysis performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    /// Basic text-level analysis
    Surface,
    /// AST-based semantic analysis
    Semantic,
    /// Full project context analysis
    Contextual,
    /// Runtime behavior inference
    Behavioral,
}

/// Detailed analysis of a single violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationAnalysis {
    /// The violation being analyzed
    pub violation: Violation,
    /// Context of the code where violation occurred
    pub code_context: CodeContext,
    /// Semantic analysis of the violation
    pub semantic_analysis: SemanticAnalysis,
    /// Estimated complexity of fixing the violation
    pub fix_complexity: FixComplexity,
    /// Dependencies that might be affected
    pub dependencies: Vec<String>,
    /// Potential side effects of fixing
    pub side_effects: Vec<String>,
    /// Confidence score for automated fix (0.0-1.0)
    pub confidence_score: f32,
}

/// Context information about code where violation occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    /// Name of the function containing violation
    pub function_name: Option<String>,
    /// Full function signature
    pub function_signature: Option<String>,
    /// Return type of the function
    pub return_type: Option<String>,
    /// Whether the function is async
    pub is_async: bool,
    /// Whether the function has generic parameters
    pub is_generic: bool,
    /// Trait being implemented if applicable
    pub trait_impl: Option<String>,
    /// Lines of code surrounding the violation
    pub surrounding_code: Vec<String>,
    /// Import statements in the file
    pub imports: Vec<String>,
    /// Error handling style used in the code
    pub error_handling_style: ErrorHandlingStyle,
}

/// Style of error handling used in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStyle {
    /// Uses anyhow crate for error handling
    Anyhow,
    /// Uses thiserror with custom error types
    ThiserrorCustom,
    /// Uses standard library Result
    StdResult,
    /// Primarily uses Option for error handling
    OptionBased,
    /// Uses panic for error conditions
    Panic,
    /// Unknown or mixed error handling style
    Unknown,
}

/// Semantic analysis results for code understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    /// Actual type being used in the violation
    pub actual_type: Option<String>,
    /// Expected type for proper handling
    pub expected_type: Option<String>,
    /// Data flow analysis results
    pub data_flow: Vec<String>,
    /// Control flow analysis results
    pub control_flow: Vec<String>,
    /// Variable usage tracking across lines
    pub variable_usage: HashMap<String, Vec<usize>>,
    /// Function calls made in the context
    pub function_calls: Vec<String>,
    /// Path of error propagation in the code
    pub error_propagation_path: Vec<String>,
}

/// Complexity level of fixing a violation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FixComplexity {
    /// Simple text replacement
    Trivial,
    /// Single-line change with type checking
    Simple,
    /// Multi-line changes, needs context
    Moderate,
    /// Requires refactoring
    Complex,
    /// Needs design changes
    Architectural,
}

/// Patterns detected in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePatterns {
    /// Commonly used patterns in the code
    pub common_patterns: Vec<Pattern>,
    /// Anti-patterns detected
    pub anti_patterns: Vec<Pattern>,
    /// Overall architectural style
    pub architectural_style: ArchitecturalStyle,
    /// Error handling pattern used
    pub error_handling_pattern: ErrorPattern,
}

/// A code pattern detected in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Name of the pattern
    pub name: String,
    /// Number of occurrences
    pub occurrences: usize,
    /// Locations where pattern is found
    pub locations: Vec<String>,
    /// Description of the pattern
    pub description: String,
}

/// Architectural style of the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturalStyle {
    /// Service-oriented architecture
    ServiceOriented,
    /// Monolithic architecture
    Monolithic,
    /// Modular architecture
    Modular,
    /// Functional programming style
    Functional,
    /// Object-oriented programming style
    ObjectOriented,
    /// Mixed architectural style
    Mixed,
}

/// Error handling pattern in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorPattern {
    /// Uses Result for all error handling
    ResultEverywhere,
    /// Mixed error handling approaches
    MixedErrorHandling,
    /// Heavy use of panic for errors
    PanicHeavy,
    /// Heavy use of Option for errors
    OptionHeavy,
    /// Uses custom error types
    CustomErrors,
}

/// Strategy for fixing a type of violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStrategy {
    /// Type of violation this strategy addresses
    pub violation_type: String,
    /// Name of the strategy
    pub strategy_name: String,
    /// Description of the strategy
    pub description: String,
    /// Steps to implement the fix
    pub steps: Vec<String>,
    /// Prerequisites for applying the strategy
    pub prerequisites: Vec<String>,
    /// Potential risks of applying the strategy
    pub risks: Vec<String>,
    /// Confidence score for the strategy
    pub confidence: f32,
    /// Estimated time to implement in minutes
    pub estimated_time_minutes: u32,
}

/// Instructions for AI/LLM agents to fix violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInstructions {
    /// System prompt for the AI agent
    pub system_prompt: String,
    /// Violation-specific prompts
    pub violation_specific_prompts: Vec<ViolationPrompt>,
    /// Required context for fixing
    pub context_requirements: Vec<String>,
    /// Criteria for validating fixes
    pub validation_criteria: Vec<String>,
    /// Instructions for rolling back changes
    pub rollback_instructions: String,
}

/// Specific prompt for handling a violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationPrompt {
    /// ID of the violation
    pub violation_id: String,
    /// Prompt text for the AI
    pub prompt: String,
    /// Knowledge required to fix the violation
    pub required_knowledge: Vec<String>,
    /// Expected format of the fix
    pub expected_output_format: String,
}

/// Performs deep AI-powered analysis of violations
pub async fn analyze_violations_for_ai(
    violations: &[Violation],
    project_path: &Path,
) -> anyhow::Result<AIAnalysisReport> {
    println!("🤖 Performing AI-powered code analysis...");

    let mut violation_analyses = Vec::new();
    let mut analyzed_count = 0;

    for violation in violations.iter().take(100) {
        // Limit for performance
        if let Ok(analysis) = analyze_single_violation(violation, project_path).await {
            violation_analyses.push(analysis);
            analyzed_count += 1;
        }
    }

    let code_patterns = detect_code_patterns(project_path, violations)?;
    let fix_strategies = generate_fix_strategies(violations, &code_patterns);
    let ai_instructions = generate_ai_instructions(&violation_analyses, &fix_strategies);

    Ok(AIAnalysisReport {
        metadata: AnalysisMetadata {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_violations: violations.len(),
            analyzable_violations: analyzed_count,
            project_path: project_path.display().to_string(),
            analysis_depth: AnalysisDepth::Semantic,
        },
        violation_analyses,
        code_patterns,
        fix_strategies,
        ai_instructions,
    })
}

async fn analyze_single_violation(
    violation: &Violation,
    _project_path: &Path,
) -> anyhow::Result<ViolationAnalysis> {
    let file_content = fs::read_to_string(&violation.file)?;

    // Parse the file using syn for AST analysis
    let ast = parse_file(&file_content).ok();

    let code_context = extract_code_context(&file_content, violation, ast.as_ref())?;
    let semantic_analysis = perform_semantic_analysis(&file_content, violation, ast.as_ref())?;
    let fix_complexity = assess_fix_complexity(violation, &code_context, &semantic_analysis);

    let dependencies = extract_dependencies(&code_context, &semantic_analysis);
    let side_effects = identify_side_effects(&semantic_analysis);
    let confidence_score = calculate_confidence(&code_context, &semantic_analysis);

    Ok(ViolationAnalysis {
        violation: violation.clone(),
        code_context,
        semantic_analysis,
        fix_complexity,
        dependencies,
        side_effects,
        confidence_score,
    })
}

fn extract_code_context(
    content: &str,
    violation: &Violation,
    ast: Option<&syn::File>,
) -> anyhow::Result<CodeContext> {
    let lines: Vec<&str> = content.lines().collect();
    let violation_line = violation.line.saturating_sub(1);

    // Extract surrounding code (5 lines before and after)
    let start = violation_line.saturating_sub(5);
    let end = (violation_line + 5).min(lines.len() - 1);
    let surrounding_code = lines[start..=end].iter().map(|s| s.to_string()).collect();

    // Extract function context if available
    let (function_name, function_signature, return_type, is_async, is_generic) =
        if let Some(ast) = ast {
            extract_function_info(ast, violation_line)
        } else {
            (None, None, None, false, false)
        };

    // Detect imports
    let imports: Vec<String> = lines
        .iter()
        .filter(|l| l.starts_with("use "))
        .map(|s| s.to_string())
        .collect();

    // Detect error handling style
    let error_handling_style = detect_error_handling_style(&imports, content);

    Ok(CodeContext {
        function_name,
        function_signature,
        return_type,
        is_async,
        is_generic,
        trait_impl: detect_trait_impl(content, violation_line),
        surrounding_code,
        imports,
        error_handling_style,
    })
}

fn extract_function_info(
    ast: &syn::File,
    _line: usize,
) -> (Option<String>, Option<String>, Option<String>, bool, bool) {
    for item in &ast.items {
        if let Item::Fn(func) = item {
            // This is simplified - real implementation would check line numbers
            let name = Some(func.sig.ident.to_string());
            let signature = Some(format!("{}", quote::quote!(#func.sig)));
            let return_type = match &func.sig.output {
                ReturnType::Default => None,
                ReturnType::Type(_, ty) => Some(format!("{}", quote::quote!(#ty))),
            };
            let is_async = func.sig.asyncness.is_some();
            let is_generic = !func.sig.generics.params.is_empty();

            return (name, signature, return_type, is_async, is_generic);
        }
    }

    (None, None, None, false, false)
}

fn detect_error_handling_style(imports: &[String], content: &str) -> ErrorHandlingStyle {
    if imports.iter().any(|i| i.contains("anyhow")) {
        ErrorHandlingStyle::Anyhow
    } else if imports.iter().any(|i| i.contains("thiserror")) {
        ErrorHandlingStyle::ThiserrorCustom
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
    // Simplified trait detection
    let lines: Vec<&str> = content.lines().collect();
    for i in (0..line.min(lines.len())).rev() {
        if lines[i].contains("impl") && lines[i].contains("for") {
            return Some(lines[i].to_string());
        }
    }
    None
}

fn perform_semantic_analysis(
    content: &str,
    violation: &Violation,
    _ast: Option<&syn::File>,
) -> anyhow::Result<SemanticAnalysis> {
    // This is a simplified version - real implementation would use full AST analysis
    let lines: Vec<&str> = content.lines().collect();
    let violation_line = violation.line.saturating_sub(1);

    let actual_type = infer_actual_type(&lines, violation_line);
    let expected_type = infer_expected_type(&violation.violation_type);

    // Simple data flow analysis
    let data_flow = trace_data_flow(&lines, violation_line);
    let control_flow = trace_control_flow(&lines, violation_line);

    // Variable usage analysis
    let variable_usage = analyze_variable_usage(&lines);

    // Function call analysis
    let function_calls = extract_function_calls(&lines[violation_line]);

    // Error propagation path
    let error_propagation_path = trace_error_propagation(&lines, violation_line);

    Ok(SemanticAnalysis {
        actual_type,
        expected_type,
        data_flow,
        control_flow,
        variable_usage,
        function_calls,
        error_propagation_path,
    })
}

fn infer_actual_type(lines: &[&str], line_idx: usize) -> Option<String> {
    // Simplified type inference
    if line_idx < lines.len() {
        let line = lines[line_idx];
        if line.contains("Result<") {
            Some("Result".to_string())
        } else if line.contains("Option<") {
            Some("Option".to_string())
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
        _ => None,
    }
}

fn trace_data_flow(lines: &[&str], line_idx: usize) -> Vec<String> {
    // Simplified data flow tracing
    let mut flow = Vec::new();
    if line_idx < lines.len() {
        flow.push(format!("Line {}: {}", line_idx + 1, lines[line_idx].trim()));
    }
    flow
}

fn trace_control_flow(lines: &[&str], _line_idx: usize) -> Vec<String> {
    // Simplified control flow tracing
    let mut flow = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.contains("if ") || line.contains("match ") || line.contains("for ") {
            flow.push(format!("Line {}: Control structure", i + 1));
        }
    }
    flow
}

fn analyze_variable_usage(lines: &[&str]) -> HashMap<String, Vec<usize>> {
    // Simplified variable usage analysis
    let mut usage = HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        if line.contains("let ") {
            if let Some(var_name) = extract_variable_name(line) {
                usage.entry(var_name).or_insert_with(Vec::new).push(i);
            }
        }
    }
    usage
}

fn extract_variable_name(line: &str) -> Option<String> {
    // Very simplified variable extraction
    if let Some(start) = line.find("let ") {
        let rest = &line[start + 4..];
        if let Some(end) = rest.find([' ', ':', '=']) {
            return Some(rest[..end].trim().to_string());
        }
    }
    None
}

fn extract_function_calls(line: &str) -> Vec<String> {
    // Simplified function call extraction
    let mut calls = Vec::new();
    for part in line.split('.') {
        if part.contains('(') {
            if let Some(end) = part.find('(') {
                calls.push(part[..end].trim().to_string());
            }
        }
    }
    calls
}

fn trace_error_propagation(lines: &[&str], _line_idx: usize) -> Vec<String> {
    // Simplified error propagation tracing
    let mut path = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.contains('?') || line.contains(".unwrap()") || line.contains(".expect(") {
            path.push(format!("Line {}: Error handling point", i + 1));
        }
    }
    path
}

fn assess_fix_complexity(
    violation: &Violation,
    context: &CodeContext,
    _semantic: &SemanticAnalysis,
) -> FixComplexity {
    match violation.violation_type {
        ViolationType::UnwrapInProduction => {
            if context
                .return_type
                .as_ref()
                .is_some_and(|t| t.contains("Result"))
            {
                FixComplexity::Simple
            } else if context.trait_impl.is_some() {
                FixComplexity::Complex
            } else {
                FixComplexity::Moderate
            }
        }
        ViolationType::UnderscoreBandaid => {
            if context.trait_impl.is_some() {
                FixComplexity::Architectural
            } else {
                FixComplexity::Moderate
            }
        }
        ViolationType::FunctionTooLarge => FixComplexity::Complex,
        ViolationType::FileTooLarge => FixComplexity::Architectural,
        _ => FixComplexity::Moderate,
    }
}

fn extract_dependencies(context: &CodeContext, semantic: &SemanticAnalysis) -> Vec<String> {
    let mut deps = Vec::new();

    // Add imports as dependencies
    for import in &context.imports {
        if let Some(dep) = extract_crate_from_import(import) {
            deps.push(dep);
        }
    }

    // Add function calls as potential dependencies
    deps.extend(semantic.function_calls.clone());

    deps
}

fn extract_crate_from_import(import: &str) -> Option<String> {
    // Extract crate name from use statement
    if let Some(start) = import.find("use ") {
        let rest = &import[start + 4..];
        if let Some(end) = rest.find("::") {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn identify_side_effects(semantic: &SemanticAnalysis) -> Vec<String> {
    let mut effects = Vec::new();

    for call in &semantic.function_calls {
        if is_side_effect_function(call) {
            effects.push(format!("Function '{}' may have side effects", call));
        }
    }

    effects
}

fn is_side_effect_function(name: &str) -> bool {
    // Common side-effect functions
    matches!(
        name,
        "print" | "println" | "eprintln" | "write" | "flush" | "spawn" | "send"
    )
}

fn calculate_confidence(context: &CodeContext, semantic: &SemanticAnalysis) -> f32 {
    let mut score = 0.5_f32; // Base confidence

    // Increase confidence if we have good context
    if context.function_name.is_some() {
        score += 0.1;
    }
    if context.return_type.is_some() {
        score += 0.15;
    }
    if !semantic.function_calls.is_empty() {
        score += 0.1;
    }
    if matches!(
        context.error_handling_style,
        ErrorHandlingStyle::Anyhow | ErrorHandlingStyle::StdResult
    ) {
        score += 0.15;
    }

    score.min(1.0)
}

fn detect_code_patterns(
    _project_path: &Path,
    violations: &[Violation],
) -> anyhow::Result<CodePatterns> {
    // Analyze patterns across the codebase
    let common_patterns = Vec::new();
    let mut anti_patterns = Vec::new();

    // Count unwrap patterns
    let unwrap_count = violations
        .iter()
        .filter(|v| matches!(v.violation_type, ViolationType::UnwrapInProduction))
        .count();

    if unwrap_count > 10 {
        anti_patterns.push(Pattern {
            name: "Excessive Unwrapping".to_string(),
            occurrences: unwrap_count,
            locations: violations
                .iter()
                .filter(|v| matches!(v.violation_type, ViolationType::UnwrapInProduction))
                .map(|v| format!("{}:{}", v.file.display(), v.line))
                .take(5)
                .collect(),
            description: "Heavy use of .unwrap() indicates poor error handling".to_string(),
        });
    }

    // Detect architectural style (simplified)
    let architectural_style = ArchitecturalStyle::Modular;

    // Detect error handling pattern
    let error_handling_pattern = if unwrap_count > 20 {
        ErrorPattern::PanicHeavy
    } else {
        ErrorPattern::MixedErrorHandling
    };

    Ok(CodePatterns {
        common_patterns,
        anti_patterns,
        architectural_style,
        error_handling_pattern,
    })
}

fn generate_fix_strategies(
    _violations: &[Violation], 
    _patterns: &CodePatterns
) -> Vec<FixStrategy> {
    vec![
        // Strategy for UnwrapInProduction
        FixStrategy {
        violation_type: "UnwrapInProduction".to_string(),
        strategy_name: "Progressive Error Handling Migration".to_string(),
        description: "Gradually replace unwrap() with proper error handling".to_string(),
        steps: vec![
            "1. Identify function return types".to_string(),
            "2. Add Result return types where missing".to_string(),
            "3. Replace .unwrap() with ? operator".to_string(),
            "4. Add context with anyhow::Context trait".to_string(),
            "5. Implement custom error types for domain errors".to_string(),
        ],
        prerequisites: vec![
            "anyhow or thiserror dependency".to_string(),
            "Understanding of error propagation paths".to_string(),
        ],
        risks: vec![
            "Breaking API changes if public functions modified".to_string(),
            "Need to handle errors at call sites".to_string(),
        ],
        confidence: 0.8,
        estimated_time_minutes: 15,
    },
    // Strategy for UnderscoreBandaid
    FixStrategy {
        violation_type: "UnderscoreBandaid".to_string(),
        strategy_name: "Implement Missing Functionality".to_string(),
        description: "Either use parameters properly or remove them".to_string(),
        steps: vec![
            "1. Analyze if parameter is actually needed".to_string(),
            "2. If needed, implement the missing logic".to_string(),
            "3. If not needed, remove from signature".to_string(),
            "4. Update all call sites".to_string(),
            "5. Add tests for new functionality".to_string(),
        ],
        prerequisites: vec![
            "Understanding of function's purpose".to_string(),
            "Access to all call sites".to_string(),
        ],
        risks: vec![
            "May break existing code if signature changes".to_string(),
            "Could introduce bugs if logic is complex".to_string(),
        ],
        confidence: 0.6,
        estimated_time_minutes: 30,
    }]
}

fn generate_ai_instructions(
    analyses: &[ViolationAnalysis],
    _strategies: &[FixStrategy],
) -> AIInstructions {
    let system_prompt = 
        r#"You are an expert Rust developer tasked with fixing code violations 
identified by Ferrous Forge.

Your goals:
1. Fix violations while maintaining code functionality
2. Improve error handling without breaking existing behavior
3. Follow Rust best practices and idioms
4. Ensure all changes compile and pass tests
5. Add appropriate documentation for complex changes

Key principles:
- Preserve existing behavior unless explicitly broken
- Prefer explicit error handling over panics
- Use type system to prevent errors at compile time
- Write self-documenting code
- Consider performance implications"#.to_string();

    let mut violation_specific_prompts = Vec::new();

    for (i, analysis) in analyses.iter().enumerate() {
        let prompt = generate_specific_prompt(analysis);
        violation_specific_prompts.push(ViolationPrompt {
            violation_id: format!("violation_{}", i),
            prompt,
            required_knowledge: vec![
                "Rust error handling patterns".to_string(),
                "Function signatures and return types".to_string(),
                "Trait implementations".to_string(),
            ],
            expected_output_format: "Rust code with explanatory comments".to_string(),
        });
    }

    AIInstructions {
        system_prompt,
        violation_specific_prompts,
        context_requirements: vec![
            "Full file content".to_string(),
            "Function signatures".to_string(),
            "Import statements".to_string(),
            "Test coverage".to_string(),
        ],
        validation_criteria: vec![
            "Code must compile".to_string(),
            "Tests must pass".to_string(),
            "No new violations introduced".to_string(),
            "Performance not degraded".to_string(),
        ],
        rollback_instructions:
            "If changes break compilation or tests, revert and mark for manual review".to_string(),
    }
}

fn generate_specific_prompt(analysis: &ViolationAnalysis) -> String {
    format!(
        r#"Fix the following {} violation:
        
File: {}
Line: {}
Current code: {:?}

Context:
- Function: {:?}
- Return type: {:?}
- Error handling style: {:?}
- Complexity: {:?}

The fix should:
1. Resolve the violation
2. Maintain existing behavior
3. Use appropriate error handling for this context
4. Be idiomatic Rust code

Confidence in automated fix: {:.1}%"#,
        format_args!("{:?}", analysis.violation.violation_type),
        analysis.violation.file.display(),
        analysis.violation.line,
        analysis
            .code_context
            .surrounding_code
            .get(5)
            .unwrap_or(&String::new()),
        analysis.code_context.function_name,
        analysis.code_context.return_type,
        analysis.code_context.error_handling_style,
        analysis.fix_complexity,
        analysis.confidence_score * 100.0
    )
}

/// Generates a comprehensive report for Claude Orchestrator
pub async fn generate_orchestrator_instructions(
    analysis_report: &AIAnalysisReport,
) -> anyhow::Result<String> {
    let mut instructions = String::new();

    instructions.push_str(&format!(
        r#"# Claude Orchestrator Fix Instructions

## Overview
Total violations to fix: {}
Analyzable violations: {}
Confidence level: High for {}, Medium for {}, Low for {}

## Fix Priority
"#,
        analysis_report.metadata.total_violations,
        analysis_report.metadata.analyzable_violations,
        analysis_report
            .violation_analyses
            .iter()
            .filter(|a| a.confidence_score > 0.8)
            .count(),
        analysis_report
            .violation_analyses
            .iter()
            .filter(|a| a.confidence_score > 0.5 && a.confidence_score <= 0.8)
            .count(),
        analysis_report
            .violation_analyses
            .iter()
            .filter(|a| a.confidence_score <= 0.5)
            .count(),
    ));

    // Group violations by complexity
    for complexity in &[
        FixComplexity::Trivial,
        FixComplexity::Simple,
        FixComplexity::Moderate,
        FixComplexity::Complex,
        FixComplexity::Architectural,
    ] {
        let violations_of_complexity: Vec<_> = analysis_report
            .violation_analyses
            .iter()
            .filter(|a| matches!(&a.fix_complexity, c if c == complexity))
            .collect();

        if !violations_of_complexity.is_empty() {
            instructions.push_str(&format!(
                "\n### {:?} Fixes ({} violations)\n",
                complexity,
                violations_of_complexity.len()
            ));

            for (i, analysis) in violations_of_complexity.iter().take(3).enumerate() {
                instructions.push_str(&format!(
                    "{}. {}:{} - {}\n",
                    i + 1,
                    analysis.violation.file.display(),
                    analysis.violation.line,
                    analysis.violation.message
                ));
            }
        }
    }

    // Add specific strategies
    instructions.push_str("\n## Recommended Strategies\n\n");
    for strategy in &analysis_report.fix_strategies {
        instructions.push_str(&format!(
            "### {}\n{}\nEstimated time: {} minutes\nConfidence: {:.0}%\n\n",
            strategy.strategy_name,
            strategy.description,
            strategy.estimated_time_minutes,
            strategy.confidence * 100.0
        ));
    }

    // Add AI prompts
    instructions.push_str("\n## AI Agent Instructions\n\n");
    instructions.push_str(&analysis_report.ai_instructions.system_prompt);
    instructions.push_str("\n\n### Validation Criteria\n");
    for criterion in &analysis_report.ai_instructions.validation_criteria {
        instructions.push_str(&format!("- {}\n", criterion));
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handling_detection() {
        let imports = vec!["use anyhow::Result;".to_string()];
        let content = "fn main() -> Result<()> { Ok(()) }";
        let style = detect_error_handling_style(&imports, content);
        assert!(matches!(style, ErrorHandlingStyle::Anyhow));
    }

    #[test]
    fn test_confidence_calculation() {
        let context = CodeContext {
            function_name: Some("process".to_string()),
            function_signature: Some("fn process() -> Result<()>".to_string()),
            return_type: Some("Result<()>".to_string()),
            is_async: false,
            is_generic: false,
            trait_impl: None,
            surrounding_code: vec![],
            imports: vec![],
            error_handling_style: ErrorHandlingStyle::Anyhow,
        };

        let semantic = SemanticAnalysis {
            actual_type: Some("Result".to_string()),
            expected_type: Some("Result".to_string()),
            data_flow: vec![],
            control_flow: vec![],
            variable_usage: HashMap::new(),
            function_calls: vec!["process_data".to_string()],
            error_propagation_path: vec![],
        };

        let confidence = calculate_confidence(&context, &semantic);
        assert!(confidence > 0.7);
    }
}

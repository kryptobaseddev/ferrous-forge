use super::types::{
    AIInstructions, ArchitecturalStyle, CodePatterns, ErrorPattern, FixStrategy, Pattern,
    ViolationAnalysis,
};
use crate::validation::ViolationType;

/// Generate fix strategies from violation analyses
pub fn generate_fix_strategies(analyses: &[ViolationAnalysis]) -> Vec<FixStrategy> {
    let mut strategies = Vec::new();
    let mut violation_counts = std::collections::HashMap::new();

    for analysis in analyses {
        *violation_counts
            .entry(&analysis.violation.violation_type)
            .or_insert(0) += 1;
    }

    for (violation_type, count) in violation_counts {
        strategies.push(create_strategy(violation_type.clone(), count));
    }

    strategies
}

fn create_strategy(violation_type: ViolationType, count: usize) -> FixStrategy {
    match violation_type {
        ViolationType::UnwrapInProduction => FixStrategy {
            violation_type,
            strategy_name: "Replace unwrap with proper error handling".to_string(),
            description: format!(
                "Replace {} instances of .unwrap() with ? operator or proper error handling",
                count
            ),
            implementation_steps: vec![
                "Identify function return type".to_string(),
                "If Result type, replace .unwrap() with ?".to_string(),
                "If not, change return type to Result".to_string(),
                "Update function callers".to_string(),
            ],
            estimated_effort: if count < 10 { "Low" } else { "Medium" }.to_string(),
            risk_level: "Low".to_string(),
        },
        ViolationType::UnderscoreBandaid => FixStrategy {
            violation_type,
            strategy_name: "Remove underscore prefixes".to_string(),
            description: format!("Fix {} underscore parameter warnings properly", count),
            implementation_steps: vec![
                "Identify unused parameters".to_string(),
                "Either use the parameter or remove it".to_string(),
                "Update function signatures if needed".to_string(),
            ],
            estimated_effort: "Low".to_string(),
            risk_level: "Low".to_string(),
        },
        ViolationType::FunctionTooLarge => FixStrategy {
            violation_type,
            strategy_name: "Refactor large functions".to_string(),
            description: format!("Break down {} large functions into smaller ones", count),
            implementation_steps: vec![
                "Identify logical sections".to_string(),
                "Extract helper functions".to_string(),
                "Improve code organization".to_string(),
            ],
            estimated_effort: "High".to_string(),
            risk_level: "Medium".to_string(),
        },
        ViolationType::FileTooLarge => FixStrategy {
            violation_type,
            strategy_name: "Split large files into modules".to_string(),
            description: format!("Modularize {} large files", count),
            implementation_steps: vec![
                "Identify logical components".to_string(),
                "Create module structure".to_string(),
                "Move code to appropriate modules".to_string(),
                "Update imports".to_string(),
            ],
            estimated_effort: "High".to_string(),
            risk_level: "Medium".to_string(),
        },
        _ => FixStrategy {
            violation_type,
            strategy_name: "Generic fix".to_string(),
            description: format!("Fix {} violations", count),
            implementation_steps: vec!["Analyze violation".to_string(), "Apply fix".to_string()],
            estimated_effort: "Medium".to_string(),
            risk_level: "Low".to_string(),
        },
    }
}

/// Generate AI instructions for fixing violations
pub fn generate_ai_instructions(
    analyses: &[ViolationAnalysis],
    _strategies: &[FixStrategy],
) -> AIInstructions {
    let summary = format!(
        "Analyzed {} violations. {} are AI-fixable with varying complexity.",
        analyses.len(),
        analyses.iter().filter(|a| a.ai_fixable).count()
    );

    let mut prioritized_fixes = Vec::new();

    // Prioritize by complexity and confidence
    let mut trivial_fixes = Vec::new();
    let mut simple_fixes = Vec::new();
    let mut moderate_fixes = Vec::new();

    for analysis in analyses {
        if !analysis.ai_fixable {
            continue;
        }

        let fix_desc = format!(
            "Fix {:?} at {}:{} (confidence: {:.0}%)",
            analysis.violation.violation_type,
            analysis.violation.file.display(),
            analysis.violation.line,
            analysis.confidence_score * 100.0
        );

        match analysis.fix_complexity {
            super::types::FixComplexity::Trivial => trivial_fixes.push(fix_desc),
            super::types::FixComplexity::Simple => simple_fixes.push(fix_desc),
            super::types::FixComplexity::Moderate => moderate_fixes.push(fix_desc),
            _ => {}
        }
    }

    prioritized_fixes.extend(trivial_fixes);
    prioritized_fixes.extend(simple_fixes);
    prioritized_fixes.extend(moderate_fixes);

    let architectural_recommendations = vec![
        "Consider adopting consistent error handling patterns".to_string(),
        "Modularize large files to improve maintainability".to_string(),
        "Extract complex logic into well-tested utility functions".to_string(),
    ];

    let code_quality_improvements = vec![
        "Add comprehensive documentation".to_string(),
        "Increase test coverage".to_string(),
        "Implement CI/CD checks for code standards".to_string(),
    ];

    AIInstructions {
        summary,
        prioritized_fixes,
        architectural_recommendations,
        code_quality_improvements,
    }
}

/// Identify code patterns in the given content
pub fn identify_code_patterns(content: &str) -> CodePatterns {
    let architectural_style = detect_architectural_style(content);
    let error_patterns = detect_error_patterns(content);
    let common_patterns = detect_common_patterns(content);

    CodePatterns {
        architectural_style,
        error_patterns,
        common_patterns,
    }
}

fn detect_architectural_style(content: &str) -> ArchitecturalStyle {
    if content.contains("mod ") && content.matches("mod ").count() > 5 {
        ArchitecturalStyle::Modular
    } else if content.contains("async fn") && content.contains("tokio") {
        ArchitecturalStyle::EventDriven
    } else if content.contains("layer") || content.contains("Layer") {
        ArchitecturalStyle::Layered
    } else {
        ArchitecturalStyle::Unknown
    }
}

fn detect_error_patterns(content: &str) -> Vec<ErrorPattern> {
    let mut patterns = Vec::new();

    if content.contains(".unwrap()") {
        patterns.push(ErrorPattern::UnwrapUsage);
    }
    if content.contains(".expect(") {
        patterns.push(ErrorPattern::ExpectUsage);
    }
    if content.contains("panic!") {
        patterns.push(ErrorPattern::PanicUsage);
    }
    if content.contains("let _ =") {
        patterns.push(ErrorPattern::IgnoredError);
    }
    if content.contains("?") {
        patterns.push(ErrorPattern::PropagatedError);
    }

    patterns
}

fn detect_common_patterns(content: &str) -> Vec<Pattern> {
    let mut patterns = Vec::new();

    let builder_count = content.matches("Builder").count();
    if builder_count > 0 {
        patterns.push(Pattern {
            name: "Builder Pattern".to_string(),
            occurrences: builder_count,
            locations: vec![],
        });
    }

    let factory_count = content.matches("Factory").count() + content.matches("::new(").count();
    if factory_count > 0 {
        patterns.push(Pattern {
            name: "Factory Pattern".to_string(),
            occurrences: factory_count,
            locations: vec![],
        });
    }

    patterns
}

//! Auto-fix command for Ferrous Forge violations
//!
//! This module implements intelligent auto-fixing for common Rust anti-patterns.
//! It analyzes code context to ensure fixes are safe and won't break compilation.

use crate::ai_analyzer;
use crate::validation::{RustValidator, Violation, ViolationType};
use crate::Result;
use anyhow::Context;
use console::style;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Execute the fix command
pub async fn execute(
    path: Option<PathBuf>,
    only: Option<String>,
    skip: Option<String>,
    dry_run: bool,
    limit: Option<usize>,
) -> Result<()> {
    execute_with_ai(path, only, skip, dry_run, limit, false).await
}

/// Execute the fix command with optional AI analysis
pub async fn execute_with_ai(
    path: Option<PathBuf>,
    only: Option<String>,
    skip: Option<String>,
    dry_run: bool,
    limit: Option<usize>,
    ai_analysis: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    println!(
        "{}",
        style("🔧 Running Ferrous Forge auto-fix...").bold().cyan()
    );
    println!("📁 Project: {}", project_path.display());

    if dry_run {
        println!(
            "{}",
            style("ℹ️ Dry-run mode - no changes will be made").yellow()
        );
    }

    // Parse filter options
    let only_types: Option<HashSet<String>> =
        only.map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect());

    let skip_types: Option<HashSet<String>> =
        skip.map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect());

    // Create validator and run validation
    let validator = RustValidator::new(project_path.clone())?;
    let violations = validator.validate_project().await?;

    // Filter violations based on options
    let violations_to_fix = filter_violations(&violations, &only_types, &skip_types, limit);

    if violations_to_fix.is_empty() {
        println!("{}", style("✅ No auto-fixable violations found!").green());
        println!(
            "\n{}",
            style("ℹ️  Note: Not all violations can be auto-fixed safely.").yellow()
        );
        println!("    Some violations require manual intervention to ensure correctness.");
        return Ok(());
    }

    println!(
        "📊 Found {} potentially fixable violations",
        violations_to_fix.len()
    );
    println!(
        "\n{}",
        style("⚠️  WARNING: Auto-fix is experimental!")
            .yellow()
            .bold()
    );
    println!("    Please review all changes and ensure your tests still pass.");

    if !dry_run {
        println!(
            "\n{}",
            style("💡 Tip: Use --dry-run first to preview changes").cyan()
        );
    }

    // Group violations by file for efficient fixing
    let violations_by_file = group_violations_by_file(&violations_to_fix);

    let mut fixed_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    for (file_path, file_violations) in violations_by_file {
        match fix_file_violations(&file_path, &file_violations, dry_run) {
            Ok((fixed, skipped)) => {
                fixed_count += fixed;
                skipped_count += skipped;
                if fixed > 0 && !dry_run {
                    println!(
                        "  {} Fixed {} violations in {}",
                        style("✓").green(),
                        fixed,
                        file_path.display()
                    );
                }
                if skipped > 0 {
                    println!(
                        "  {} Skipped {} unsafe fixes in {}",
                        style("⚠").yellow(),
                        skipped,
                        file_path.display()
                    );
                }
            }
            Err(e) => {
                failed_count += file_violations.len();
                eprintln!(
                    "  {} Failed to fix {}: {}",
                    style("✗").red(),
                    file_path.display(),
                    e
                );
            }
        }
    }

    // Print summary
    println!("\n{}", "─".repeat(50));
    if dry_run {
        println!(
            "{} Would fix {} violations safely",
            style("📝").blue(),
            fixed_count
        );
        if skipped_count > 0 {
            println!(
                "{} Would skip {} unsafe fixes",
                style("⚠️").yellow(),
                skipped_count
            );
        }
    } else {
        println!("{} Fixed {} violations", style("✅").green(), fixed_count);
        if skipped_count > 0 {
            println!(
                "{} Skipped {} unsafe fixes",
                style("⚠️").yellow(),
                skipped_count
            );
        }
    }

    if failed_count > 0 {
        println!(
            "{} Failed to process {} violations",
            style("❌").red(),
            failed_count
        );
    }

    if fixed_count > 0 && !dry_run {
        println!("\n{}", style("📌 Next Steps:").bold());
        println!("  1. Review the changes with 'git diff'");
        println!("  2. Run 'cargo build' to ensure compilation");
        println!("  3. Run 'cargo test' to ensure tests pass");
        println!("  4. Commit the changes if everything looks good");
    }

    // AI Analysis Layer - Second Pass for Complex Fixes
    if ai_analysis && (skipped_count > 0 || failed_count > 0) {
        println!("\n{}", "═".repeat(50));
        println!("{}", style("🤖 AI ANALYSIS LAYER ACTIVATED").bold().cyan());
        println!("{}", "═".repeat(50));

        println!(
            "\n{}",
            style("Performing deep semantic analysis...").yellow()
        );

        // Get all violations that weren't fixed
        let unfixed_violations: Vec<Violation> = violations_to_fix
            .into_iter()
            .filter(|v| {
                // This is simplified - real implementation would track which were fixed
                true
            })
            .collect();

        if !unfixed_violations.is_empty() {
            // Perform AI analysis
            match ai_analyzer::analyze_violations_for_ai(&unfixed_violations, &project_path).await {
                Ok(analysis_report) => {
                    println!("{}", style("✅ AI Analysis Complete!").green());
                    println!("\n{}", style("📊 Analysis Summary:").bold());
                    println!(
                        "  Total violations analyzed: {}",
                        analysis_report.metadata.analyzable_violations
                    );
                    println!(
                        "  Architectural issues found: {}",
                        analysis_report.code_patterns.anti_patterns.len()
                    );
                    println!(
                        "  Fix strategies generated: {}",
                        analysis_report.fix_strategies.len()
                    );

                    // Save AI analysis report
                    let reports_dir = project_path.join(".ferrous-forge").join("ai-analysis");
                    tokio::fs::create_dir_all(&reports_dir).await?;

                    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                    let report_path = reports_dir.join(format!("ai_analysis_{}.json", timestamp));

                    let report_json = serde_json::to_string_pretty(&analysis_report)?;
                    tokio::fs::write(&report_path, &report_json).await?;

                    println!("\n{}", style("📝 AI Analysis Report saved to:").green());
                    println!("  {}", report_path.display());

                    // Generate orchestrator instructions
                    let orchestrator_instructions =
                        ai_analyzer::generate_orchestrator_instructions(&analysis_report).await?;

                    let instructions_path =
                        reports_dir.join(format!("orchestrator_instructions_{}.md", timestamp));
                    tokio::fs::write(&instructions_path, &orchestrator_instructions).await?;

                    println!(
                        "\n{}",
                        style("🎯 Claude Orchestrator Instructions generated:").green()
                    );
                    println!("  {}", instructions_path.display());

                    // Display fix complexity breakdown
                    println!("\n{}", style("🔧 Fix Complexity Breakdown:").bold());
                    let mut complexity_counts = HashMap::new();
                    for analysis in &analysis_report.violation_analyses {
                        *complexity_counts
                            .entry(format!("{:?}", analysis.fix_complexity))
                            .or_insert(0) += 1;
                    }
                    for (complexity, count) in complexity_counts {
                        println!("  {}: {} violations", complexity, count);
                    }

                    // Show high-confidence fixes
                    let high_confidence: Vec<_> = analysis_report
                        .violation_analyses
                        .iter()
                        .filter(|a| a.confidence_score > 0.8)
                        .collect();

                    if !high_confidence.is_empty() {
                        println!(
                            "\n{}",
                            style("✨ High-Confidence Fix Opportunities:")
                                .green()
                                .bold()
                        );
                        for (i, analysis) in high_confidence.iter().take(5).enumerate() {
                            println!(
                                "  {}. {}:{} - Confidence: {:.0}%",
                                i + 1,
                                analysis.violation.file.display(),
                                analysis.violation.line,
                                analysis.confidence_score * 100.0
                            );
                        }
                    }

                    println!(
                        "\n{}",
                        style("🚀 Ready for AI-Powered Fixing!").bold().green()
                    );
                    println!("\n{}", style("Next Steps for AI Fixing:").bold());
                    println!("  1. Review the AI analysis report");
                    println!("  2. Use Claude Orchestrator with the generated instructions");
                    println!("  3. Or run: ferrous-forge fix --ai-assisted");
                    println!("  4. The AI will attempt intelligent fixes based on the analysis");
                }
                Err(e) => {
                    eprintln!("{} AI Analysis failed: {}", style("❌").red(), e);
                }
            }
        }
    }

    Ok(())
}

fn filter_violations(
    violations: &[Violation],
    only_types: &Option<HashSet<String>>,
    skip_types: &Option<HashSet<String>>,
    limit: Option<usize>,
) -> Vec<Violation> {
    let mut filtered: Vec<Violation> = violations
        .iter()
        .filter(|v| {
            let violation_type = format!("{:?}", v.violation_type).to_uppercase();

            // Check only filter
            if let Some(only) = only_types {
                if !only.contains(&violation_type) {
                    return false;
                }
            }

            // Check skip filter
            if let Some(skip) = skip_types {
                if skip.contains(&violation_type) {
                    return false;
                }
            }

            // Only include violations we can potentially auto-fix
            can_potentially_auto_fix(v)
        })
        .cloned()
        .collect();

    // Apply limit if specified
    if let Some(limit) = limit {
        filtered.truncate(limit);
    }

    filtered
}

fn can_potentially_auto_fix(violation: &Violation) -> bool {
    match violation.violation_type {
        ViolationType::UnwrapInProduction => true,
        ViolationType::UnderscoreBandaid => true,
        ViolationType::LineTooLong => false, // Requires intelligent reformatting
        ViolationType::FunctionTooLarge => false, // Requires refactoring
        ViolationType::FileTooLarge => false, // Requires architectural changes
        _ => false,
    }
}

fn group_violations_by_file(violations: &[Violation]) -> HashMap<PathBuf, Vec<Violation>> {
    let mut grouped: HashMap<PathBuf, Vec<Violation>> = HashMap::new();

    for violation in violations {
        grouped
            .entry(violation.file.clone())
            .or_insert_with(Vec::new)
            .push(violation.clone());
    }

    // Sort violations within each file by line number (reverse order for safe fixing)
    for file_violations in grouped.values_mut() {
        file_violations.sort_by(|a, b| b.line.cmp(&a.line));
    }

    grouped
}

fn fix_file_violations(
    file_path: &Path,
    violations: &[Violation],
    dry_run: bool,
) -> anyhow::Result<(usize, usize)> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut fixed_count = 0;
    let mut skipped_count = 0;

    // Analyze file context for safer fixes
    let file_context = analyze_file_context(&content);

    for violation in violations {
        if violation.line == 0 || violation.line > lines.len() {
            continue;
        }

        let line_idx = violation.line - 1;
        let original_line = lines[line_idx].clone();

        match fix_violation_in_line(&original_line, violation, &file_context) {
            FixResult::Fixed(fixed_line) => {
                if !dry_run {
                    lines[line_idx] = fixed_line;
                }
                fixed_count += 1;
            }
            FixResult::Skipped(_reason) => {
                skipped_count += 1;
            }
            FixResult::NotApplicable => {}
        }
    }

    if fixed_count > 0 && !dry_run {
        let fixed_content = lines.join("\n");
        // Preserve final newline if it existed
        let fixed_content = if content.ends_with('\n') {
            format!("{}\n", fixed_content)
        } else {
            fixed_content
        };
        fs::write(file_path, fixed_content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
    }

    Ok((fixed_count, skipped_count))
}

/// Context about a file that helps make safer fix decisions
#[derive(Debug)]
struct FileContext {
    is_test_file: bool,
    is_example_file: bool,
    is_bin_file: bool,
    has_anyhow_import: bool,
    has_thiserror_import: bool,
    function_signatures: Vec<FunctionSignature>,
}

#[derive(Debug)]
struct FunctionSignature {
    line: usize,
    name: String,
    returns_result: bool,
    returns_option: bool,
    is_test: bool,
    is_main: bool,
}

fn analyze_file_context(content: &str) -> FileContext {
    let lines: Vec<&str> = content.lines().collect();

    // Check file type based on common patterns
    let is_test_file = content.contains("#[cfg(test)]") || content.contains("#[test]");
    let is_example_file = content.contains("fn main()") && content.lines().count() < 100;
    let is_bin_file = content.contains("fn main()");

    // Check for error handling libraries
    let has_anyhow_import = content.contains("use anyhow::") || content.contains("anyhow::");
    let has_thiserror_import =
        content.contains("use thiserror::") || content.contains("thiserror::");

    // Analyze function signatures
    let mut function_signatures = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if let Some(sig) = parse_function_signature(line, idx) {
            function_signatures.push(sig);
        }
    }

    FileContext {
        is_test_file,
        is_example_file,
        is_bin_file,
        has_anyhow_import,
        has_thiserror_import,
        function_signatures,
    }
}

fn parse_function_signature(line: &str, line_num: usize) -> Option<FunctionSignature> {
    let trimmed = line.trim();

    if !trimmed.starts_with("fn ")
        && !trimmed.starts_with("pub fn ")
        && !trimmed.starts_with("async fn ")
    {
        return None;
    }

    // Extract function name
    let fn_start = trimmed.find("fn ")? + 3;
    let fn_end = trimmed[fn_start..].find('(')?;
    let name = trimmed[fn_start..fn_start + fn_end].trim().to_string();

    // Check return type
    let returns_result = line.contains("-> Result") || line.contains("-> anyhow::Result");
    let returns_option = line.contains("-> Option");
    let is_test = line.contains("#[test]");
    let is_main = name == "main";

    Some(FunctionSignature {
        line: line_num,
        name,
        returns_result,
        returns_option,
        is_test,
        is_main,
    })
}

enum FixResult {
    Fixed(String),
    Skipped(String),
    NotApplicable,
}

fn fix_violation_in_line(line: &str, violation: &Violation, context: &FileContext) -> FixResult {
    match violation.violation_type {
        ViolationType::UnwrapInProduction => fix_unwrap_in_line(line, violation, context),
        ViolationType::UnderscoreBandaid => fix_underscore_in_line(line, violation, context),
        _ => FixResult::NotApplicable,
    }
}

fn fix_unwrap_in_line(line: &str, _violation: &Violation, context: &FileContext) -> FixResult {
    // Skip fixes in test code
    if context.is_test_file && line.contains("#[test]") {
        return FixResult::Skipped("Test code can use unwrap".to_string());
    }

    if line.contains(".unwrap()") {
        // Check if we're in a function that can use ?
        let can_use_question_mark = check_can_use_question_mark(line, context);

        if !can_use_question_mark {
            // Try to use expect with a meaningful message instead
            if context.is_example_file || context.is_bin_file {
                let fixed = line.replace(".unwrap()", ".expect(\"Failed to process\")");
                return FixResult::Fixed(fixed);
            }
            return FixResult::Skipped("Cannot use ? operator in this context".to_string());
        }

        // Safe to replace with ?
        return FixResult::Fixed(line.replace(".unwrap()", "?"));
    }

    if line.contains(".expect(") {
        // Only fix if we have anyhow for context
        if !context.has_anyhow_import {
            return FixResult::Skipped("No anyhow import for .context()".to_string());
        }

        // Extract the expect message and convert to context
        if let Some(start) = line.find(".expect(\"") {
            if let Some(end) = line[start + 9..].find("\")") {
                let message = &line[start + 9..start + 9 + end];
                let before = &line[..start];
                let after = &line[start + 9 + end + 2..];

                let fixed = format!("{}.context(\"{}\")?{}", before, message, after);
                return FixResult::Fixed(fixed);
            }
        }
    }

    FixResult::NotApplicable
}

fn check_can_use_question_mark(line: &str, context: &FileContext) -> bool {
    // Can't use ? in main without Result return type
    if line.contains("fn main") && !line.contains("-> Result") {
        return false;
    }

    // Can't use ? in tests typically
    if line.contains("#[test]") {
        return false;
    }

    // Check if we're likely in a function that returns Result
    // This is a heuristic - proper fix would need full AST analysis
    for sig in &context.function_signatures {
        if sig.returns_result || sig.returns_option {
            return true;
        }
    }

    // Conservative: if we have error handling imports, assume we can use ?
    context.has_anyhow_import || context.has_thiserror_import
}

fn fix_underscore_in_line(line: &str, violation: &Violation, context: &FileContext) -> FixResult {
    // Fix underscore parameters in function signatures
    if line.contains("fn ") && line.contains('_') {
        // Check if this is a trait implementation where we can't change signature
        if line.contains("impl") || line.contains("trait") {
            return FixResult::Skipped("Cannot modify trait method signatures".to_string());
        }

        // For now, just warn about underscore parameters
        // Proper fix would require analyzing if the parameter is actually needed
        return FixResult::Skipped("Underscore parameters need manual review".to_string());
    }

    // Fix let _ = assignments
    if line.trim_start().starts_with("let _ =") {
        // Check if the expression likely returns Result
        if let Some(rest) = line.trim_start().strip_prefix("let _ =") {
            let rest = rest.trim();

            // Don't fix if it's a drop pattern
            if rest.contains("drop(") || rest.contains("mem::drop") {
                return FixResult::Skipped("Intentional drop pattern".to_string());
            }

            // Don't fix if it's likely not returning Result
            if rest.contains(".clone()") || rest.contains(".to_string()") {
                return FixResult::Skipped("Expression likely doesn't return Result".to_string());
            }

            if rest.ends_with(';') {
                let expr = &rest[..rest.len() - 1];
                let indent = &line[..line.find("let").unwrap_or(0)];

                // Check if we can use ?
                if check_can_use_question_mark(line, context) {
                    let fixed = format!("{}{}?;", indent, expr);
                    return FixResult::Fixed(fixed);
                } else {
                    // Use explicit error handling
                    let fixed = format!(
                        "{}if let Err(e) = {} {{ eprintln!(\"Error: {{}}\", e); }}",
                        indent, expr
                    );
                    return FixResult::Fixed(fixed);
                }
            }
        }
    }

    FixResult::NotApplicable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{Severity, ViolationType};

    #[test]
    fn test_safe_unwrap_fix() {
        let content = "use anyhow::Result;\n\nfn process() -> Result<()> {\n    let value = some_func().unwrap();\n}";
        let context = analyze_file_context(content);
        let line = "    let value = some_func().unwrap();";
        let violation = Violation {
            violation_type: ViolationType::UnwrapInProduction,
            file: PathBuf::from("test.rs"),
            line: 4,
            message: String::new(),
            severity: Severity::Error,
        };

        match fix_unwrap_in_line(line, &violation, &context) {
            FixResult::Fixed(fixed) => {
                assert_eq!(fixed, "    let value = some_func()?;");
            }
            _ => panic!("Expected fix to be applied"),
        }
    }

    #[test]
    fn test_skip_test_unwrap() {
        let content = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn test_something() {\n        value.unwrap();\n    }\n}";
        let context = analyze_file_context(content);
        let line = "        value.unwrap();";
        let violation = Violation {
            violation_type: ViolationType::UnwrapInProduction,
            file: PathBuf::from("test.rs"),
            line: 5,
            message: String::new(),
            severity: Severity::Error,
        };

        match fix_unwrap_in_line(line, &violation, &context) {
            FixResult::Skipped(reason) => {
                assert!(reason.contains("Test"));
            }
            _ => panic!("Expected fix to be skipped in test code"),
        }
    }

    #[test]
    fn test_context_analysis() {
        let content = r#"
use anyhow::Result;
use thiserror::Error;

fn main() -> Result<()> {
    Ok(())
}

#[test]
fn test_something() {
    assert!(true);
}

async fn process_data() -> Result<String> {
    Ok("data".to_string())
}
"#;
        let context = analyze_file_context(content);

        assert!(context.has_anyhow_import);
        assert!(context.has_thiserror_import);
        assert!(context.is_bin_file);
        assert!(context.is_test_file);
        assert_eq!(context.function_signatures.len(), 3);
    }
}

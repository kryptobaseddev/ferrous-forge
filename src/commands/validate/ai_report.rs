//! AI-friendly compliance report generation

use super::{markdown::generate_markdown_report, utils};
use crate::{validation::Violation, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// AI-friendly compliance report structure
#[derive(Serialize, Deserialize)]
pub struct AIReport {
    /// Report metadata including timestamp and version
    pub metadata: AIMetadata,
    /// Summary of violations found
    pub summary: AISummary,
    /// Detailed list of violations
    pub violations: Vec<AIViolation>,
    /// Instructions for fixing violations
    pub fix_instructions: Vec<FixInstruction>,
}

#[derive(Serialize, Deserialize)]
/// Metadata for AI compliance reports
pub struct AIMetadata {
    /// ISO timestamp when report was generated
    pub timestamp: String,
    /// Path to the project being analyzed
    pub project_path: String,
    /// Version of Ferrous Forge used
    pub ferrous_forge_version: String,
    /// Total number of violations found in the project
    pub total_violations: usize,
    /// Version of the report format
    pub report_version: String,
}

#[derive(Serialize, Deserialize)]
/// Summary statistics for the AI compliance report
pub struct AISummary {
    /// Percentage of code that meets Ferrous Forge standards (0-100)
    pub compliance_percentage: f64,
    /// Number of files analyzed in the project
    pub files_analyzed: usize,
    /// List of the most critical issues found
    pub most_critical_issues: Vec<String>,
    /// Estimated time in hours to fix all violations
    pub estimated_fix_time_hours: f64,
}

#[derive(Serialize, Deserialize)]
/// Individual violation details for AI analysis
pub struct AIViolation {
    /// Type of violation (e.g., "UNWRAPINPRODUCTION", "FUNCTIONTOOLARGE")
    pub violation_type: String,
    /// Path to the file containing the violation
    pub file: String,
    /// Line number where the violation occurs
    pub line: usize,
    /// Descriptive message about the violation
    pub message: String,
    /// Code snippet showing the violation in context
    pub code_snippet: String,
    /// Suggested fix for the violation
    pub suggested_fix: String,
    /// Whether this violation can be automatically fixed
    pub auto_fixable: bool,
    /// Priority level (1 = highest, 5 = lowest)
    pub priority: u8,
}

#[derive(Serialize, Deserialize)]
/// Instructions for fixing a specific type of violation
pub struct FixInstruction {
    /// Type of violation this instruction applies to
    pub violation_type: String,
    /// Number of this type of violation found
    pub count: usize,
    /// Strategy for fixing this type of violation
    pub fix_strategy: String,
    /// Example code showing how to fix this violation
    pub example_fix: String,
    /// Effort level required ("trivial", "moderate", "complex")
    pub effort_level: String,
}

/// Generate AI-friendly compliance report
pub async fn generate_ai_report(project_path: &PathBuf, violations: &[Violation]) -> Result<()> {
    let timestamp = chrono::Utc::now();
    let reports_dir = setup_reports_directory(project_path).await?;
    let timestamp_str = timestamp.format("%Y%m%d_%H%M%S").to_string();

    let violation_counts = utils::count_violations_by_type(violations);
    let ai_violations = create_ai_violations(violations).await;
    let fix_instructions = generate_fix_instructions(violation_counts);
    let compliance = utils::calculate_compliance(project_path, violations).await?;

    let report = build_ai_report(
        project_path,
        &timestamp,
        violations.len(),
        ai_violations,
        fix_instructions,
        compliance,
    );

    save_and_link_reports(&reports_dir, &timestamp_str, &report).await?;
    print_report_summary(&reports_dir, &timestamp_str);

    Ok(())
}

async fn setup_reports_directory(project_path: &PathBuf) -> Result<PathBuf> {
    let reports_dir = project_path.join(".ferrous-forge").join("reports");
    fs::create_dir_all(&reports_dir).await?;
    Ok(reports_dir)
}


async fn create_ai_violations(violations: &[Violation]) -> Vec<AIViolation> {
    let mut ai_violations = Vec::new();
    for violation in violations {
        let code_snippet = utils::get_code_snippet(&violation.file, violation.line)
            .await
            .unwrap_or_else(|_| "Could not read file".to_string());

        let (suggested_fix, auto_fixable, priority) = 
            get_fix_suggestion(&violation.violation_type, &violation.message);

        ai_violations.push(AIViolation {
            violation_type: format!("{:?}", violation.violation_type),
            file: violation.file.display().to_string(),
            line: violation.line,
            message: violation.message.clone(),
            code_snippet,
            suggested_fix,
            auto_fixable,
            priority,
        });
    }
    ai_violations
}

fn get_fix_suggestion(
    violation_type: &crate::validation::ViolationType,
    _message: &str,
) -> (String, bool, u8) {
    match violation_type {
        crate::validation::ViolationType::UnderscoreBandaid => (
            "Remove unused parameter or implement missing functionality".to_string(),
            true,
            2,
        ),
        crate::validation::ViolationType::UnwrapInProduction => (
            "Replace `.unwrap()` with `?` or proper error handling".to_string(),
            true,
            1,
        ),
        crate::validation::ViolationType::FileTooLarge => (
            "Split file into smaller modules following single responsibility principle".to_string(),
            false,
            3,
        ),
        crate::validation::ViolationType::FunctionTooLarge => (
            "Extract helper functions or split into smaller, focused functions".to_string(),
            false,
            3,
        ),
        _ => (
            "Review and fix according to Ferrous Forge standards".to_string(),
            false,
            4,
        ),
    }
}

fn generate_fix_instructions(violation_counts: HashMap<String, usize>) -> Vec<FixInstruction> {
    let mut fix_instructions = Vec::new();
    for (vtype, count) in violation_counts {
        let (strategy, example, effort) = get_fix_strategy(&vtype);
        fix_instructions.push(FixInstruction {
            violation_type: vtype,
            count,
            fix_strategy: strategy,
            example_fix: example,
            effort_level: effort,
        });
    }
    fix_instructions
}

fn get_fix_strategy(vtype: &str) -> (String, String, String) {
    match vtype {
        "UnderscoreBandaid" => (
            "1. Identify what functionality the parameter should provide\n\
            2. Either implement the functionality or remove the parameter\n\
            3. Update function signature and callers".to_string(),
            "// Before: fn process(_unused: String, data: Data)\n\
            // After: fn process(data: Data) or implement the unused parameter".to_string(),
            "Moderate".to_string(),
        ),
        "UnwrapInProduction" => (
            "1. Change function to return Result<T, Error>\n\
            2. Replace unwrap with ?\n3. Handle errors at call sites".to_string(),
            "// Before: value.unwrap()\n// After: value?".to_string(),
            "Easy".to_string(),
        ),
        "FileTooLarge" => (
            "1. Identify logical boundaries\n2. Extract modules\n\
            3. Move related functions to new modules\n\
            4. Update imports".to_string(),
            "// Extract to separate modules like: \n\
            validation/core.rs, validation/types.rs".to_string(),
            "Hard".to_string(),
        ),
        _ => (
            "Review and fix manually".to_string(),
            "".to_string(),
            "Moderate".to_string(),
        ),
    }
}


fn build_ai_report(
    project_path: &PathBuf,
    timestamp: &chrono::DateTime<chrono::Utc>,
    total_violations: usize,
    ai_violations: Vec<AIViolation>,
    fix_instructions: Vec<FixInstruction>,
    compliance_percentage: f64,
) -> AIReport {
    AIReport {
        metadata: AIMetadata {
            timestamp: timestamp.to_rfc3339(),
            project_path: project_path.display().to_string(),
            ferrous_forge_version: env!("CARGO_PKG_VERSION").to_string(),
            total_violations,
            report_version: "1.0.0".to_string(),
        },
        summary: AISummary {
            compliance_percentage,
            files_analyzed: 0, // TODO: pass this properly
            most_critical_issues: vec![
                "UnderscoreBandaid violations (implement missing functionality)".to_string(),
                "Large files need splitting".to_string(),
                "Large functions need refactoring".to_string(),
            ],
            estimated_fix_time_hours: total_violations as f64 * 0.25,
        },
        violations: ai_violations,
        fix_instructions,
    }
}

async fn save_and_link_reports(
    reports_dir: &PathBuf,
    timestamp_str: &str,
    report: &AIReport,
) -> Result<()> {
    let json_path = reports_dir.join(format!("ai_compliance_{}.json", timestamp_str));
    let json_content = serde_json::to_string_pretty(&report)
        .map_err(|e| crate::Error::config(format!("Failed to serialize AI report: {}", e)))?;
    fs::write(&json_path, json_content).await?;

    let md_path = reports_dir.join(format!("ai_compliance_{}.md", timestamp_str));
    let md_content = generate_markdown_report(&report);
    fs::write(&md_path, md_content).await?;

    let latest_json = reports_dir.join("latest_ai_report.json");
    let latest_md = reports_dir.join("latest_ai_report.md");
    fs::copy(&json_path, &latest_json).await?;
    fs::copy(&md_path, &latest_md).await?;

    Ok(())
}

fn print_report_summary(reports_dir: &PathBuf, timestamp_str: &str) {
    println!("🤖 AI compliance report generated:");
    println!("  📄 JSON: {}/ai_compliance_{}.json", reports_dir.display(), timestamp_str);
    println!("  📝 Markdown: {}/ai_compliance_{}.md", reports_dir.display(), timestamp_str);
    println!("  🔗 Latest: {}/latest_ai_report.*", reports_dir.display());
    println!();
}


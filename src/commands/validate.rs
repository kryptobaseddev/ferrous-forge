//! Validate command implementation

use crate::{
    doc_coverage, formatting, security,
    validation::{RustValidator, Violation},
    Result,
};
use console::style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// AI-friendly compliance report structure
#[derive(Serialize, Deserialize)]
struct AIReport {
    metadata: AIMetadata,
    summary: AISummary,
    violations: Vec<AIViolation>,
    fix_instructions: Vec<FixInstruction>,
}

#[derive(Serialize, Deserialize)]
struct AIMetadata {
    timestamp: String,
    project_path: String,
    ferrous_forge_version: String,
    total_violations: usize,
    report_version: String,
}

#[derive(Serialize, Deserialize)]
struct AISummary {
    compliance_percentage: f64,
    files_analyzed: usize,
    most_critical_issues: Vec<String>,
    estimated_fix_time_hours: f64,
}

#[derive(Serialize, Deserialize)]
struct AIViolation {
    violation_type: String,
    file: String,
    line: usize,
    message: String,
    code_snippet: String,
    suggested_fix: String,
    auto_fixable: bool,
    priority: u8,
}

#[derive(Serialize, Deserialize)]
struct FixInstruction {
    violation_type: String,
    count: usize,
    fix_strategy: String,
    example_fix: String,
    effort_level: String,
}

/// Execute the validate command
pub async fn execute(
    path: Option<PathBuf>,
    ai_report: bool,
    _compare_previous: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    println!(
        "{}",
        style("🦀 Running Ferrous Forge validation...")
            .bold()
            .cyan()
    );
    println!("📁 Project: {}", project_path.display());
    println!();

    // Create validator
    let validator = RustValidator::new(project_path.clone())?;

    // Run validation
    let violations = validator.validate_project().await?;

    // Generate and display report
    let report = validator.generate_report(&violations);
    println!("{}", report);

    // Generate AI-friendly report if requested
    if ai_report {
        println!("\n🤖 Generating AI-friendly compliance report...");
        generate_ai_report(&project_path, &violations).await?;
    }

    // Run clippy with our strict configuration
    println!(
        "{}",
        style("🔧 Running Clippy with strict configuration...")
            .bold()
            .yellow()
    );
    let clippy_result = validator.run_clippy().await?;

    if !clippy_result.success {
        println!("{}", style("❌ Clippy found issues:").red());
        println!("{}", clippy_result.output);
    } else {
        println!("{}", style("✅ Clippy validation passed!").green());
    }

    // Check documentation coverage
    println!();
    println!(
        "{}",
        style("📚 Checking documentation coverage...")
            .bold()
            .yellow()
    );
    match doc_coverage::check_documentation_coverage(&project_path).await {
        Ok(coverage) => {
            println!("{}", coverage.report());
            if coverage.coverage_percent < 80.0 {
                println!("{}", style("⚠️  Documentation coverage below 80%").yellow());
            }
        }
        Err(e) => {
            println!(
                "{}",
                style(format!("⚠️  Could not check documentation: {}", e)).yellow()
            );
        }
    }

    // Check formatting
    println!();
    println!(
        "{}",
        style("📝 Checking code formatting...").bold().yellow()
    );
    match formatting::check_formatting(&project_path).await {
        Ok(format_result) => {
            println!("{}", format_result.report());
        }
        Err(e) => {
            println!(
                "{}",
                style(format!("⚠️  Could not check formatting: {}", e)).yellow()
            );
        }
    }

    // Run security audit
    println!();
    println!("{}", style("🔒 Running security audit...").bold().yellow());
    match security::run_security_audit(&project_path).await {
        Ok(audit_report) => {
            println!("{}", audit_report.report());
        }
        Err(e) => {
            println!(
                "{}",
                style(format!("⚠️  Could not run security audit: {}", e)).yellow()
            );
        }
    }

    // Exit with error code if violations found
    if !violations.is_empty() || !clippy_result.success {
        std::process::exit(1);
    } else {
        println!();
        println!(
            "{}",
            style("🎉 All validations passed! Code meets Ferrous Forge standards.")
                .bold()
                .green()
        );
    }

    Ok(())
}

/// Generate AI-friendly compliance report
async fn generate_ai_report(project_path: &PathBuf, violations: &[Violation]) -> Result<()> {
    use chrono::Utc;

    // Create reports directory
    let reports_dir = project_path.join(".ferrous-forge").join("reports");
    fs::create_dir_all(&reports_dir).await?;

    // Generate timestamp
    let timestamp = Utc::now();
    let timestamp_str = timestamp.format("%Y%m%d_%H%M%S").to_string();

    // Count violations by type
    let mut violation_counts = HashMap::new();
    for violation in violations {
        *violation_counts
            .entry(format!("{:?}", violation.violation_type))
            .or_insert(0) += 1;
    }

    // Create AI violations with context
    let mut ai_violations = Vec::new();
    for violation in violations.iter().take(50) {
        // Limit to 50 for AI processing
        let code_snippet = get_code_snippet(&violation.file, violation.line)
            .await
            .unwrap_or_else(|_| "Could not read file".to_string());

        let (suggested_fix, auto_fixable, priority) = match violation.violation_type {
            crate::validation::ViolationType::UnderscoreBandaid => {
                if violation.message.contains("parameter") {
                    (
                        "Remove unused parameter or implement missing functionality".to_string(),
                        false,
                        2,
                    )
                } else {
                    (
                        "Replace `let _ =` with proper error handling using `?`".to_string(),
                        true,
                        1,
                    )
                }
            }
            crate::validation::ViolationType::UnwrapInProduction => (
                "Replace `.unwrap()` with `?` or proper error handling".to_string(),
                true,
                1,
            ),
            crate::validation::ViolationType::FileTooLarge => (
                "Split file into smaller modules following single responsibility principle"
                    .to_string(),
                false,
                4,
            ),
            crate::validation::ViolationType::FunctionTooLarge => (
                "Extract helper functions or split into smaller, focused functions".to_string(),
                false,
                3,
            ),
            _ => (
                "Review and fix according to Ferrous Forge standards".to_string(),
                false,
                3,
            ),
        };

        ai_violations.push(AIViolation {
            violation_type: format!("{:?}", violation.violation_type),
            file: violation.file.display().to_string(),
            line: violation.line + 1, // Convert to 1-based
            message: violation.message.clone(),
            code_snippet,
            suggested_fix,
            auto_fixable,
            priority,
        });
    }

    // Generate fix instructions
    let mut fix_instructions = Vec::new();
    for (vtype, count) in violation_counts {
        let (strategy, example, effort) = match vtype.as_str() {
            "UnderscoreBandaid" => (
                "1. Identify what functionality the parameter should provide\n\
                2. Either implement the functionality or remove the parameter\n\
                3. Update function signature and callers"
                    .to_string(),
                "// Before: fn process(_unused: String, data: Data)\n\
                // After: fn process(data: Data) or implement the unused parameter"
                    .to_string(),
                "Moderate".to_string(),
            ),
            "UnwrapInProduction" => (
                "1. Change function to return Result<T, Error>\n\
                2. Replace ? with ?\n3. Handle errors at call sites"
                    .to_string(),
                "// Before: value.unwrap()\n// After: value?".to_string(),
                "Easy".to_string(),
            ),
            "FileTooLarge" => (
                "1. Identify logical boundaries in the file\n\
                2. Create new module directory\n3. Split into focused modules\n\
                4. Update imports"
                    .to_string(),
                "// Split validation.rs into validation/mod.rs, \
                validation/core.rs, validation/types.rs"
                    .to_string(),
                "Hard".to_string(),
            ),
            _ => ("Review and fix manually".to_string(), "".to_string(), "Moderate".to_string()),
        };

        fix_instructions.push(FixInstruction {
            violation_type: vtype,
            count,
            fix_strategy: strategy,
            example_fix: example,
            effort_level: effort,
        });
    }

    // Calculate compliance
    let total_files = count_rust_files(project_path).await?;
    let files_with_violations = violations
        .iter()
        .map(|v| &v.file)
        .collect::<std::collections::HashSet<_>>()
        .len();

    let compliance_percentage = if total_files > 0 && files_with_violations <= total_files {
        ((total_files - files_with_violations) as f64 / total_files as f64) * 100.0
    } else {
        0.0 // If we have more violations than files, compliance is 0%
    };

    // Create report
    let report = AIReport {
        metadata: AIMetadata {
            timestamp: timestamp.to_rfc3339(),
            project_path: project_path.display().to_string(),
            ferrous_forge_version: env!("CARGO_PKG_VERSION").to_string(),
            total_violations: violations.len(),
            report_version: "1.0.0".to_string(),
        },
        summary: AISummary {
            compliance_percentage,
            files_analyzed: total_files,
            most_critical_issues: vec![
                "UnderscoreBandaid violations (implement missing functionality)".to_string(),
                "Large files need splitting (validation.rs: 1133 lines)".to_string(),
                "Large functions need refactoring".to_string(),
            ],
            estimated_fix_time_hours: violations.len() as f64 * 0.25, // 15 minutes per violation average
        },
        violations: ai_violations,
        fix_instructions,
    };

    // Save JSON report
    let json_path = reports_dir.join(format!("ai_compliance_{}.json", timestamp_str));
    let json_content = serde_json::to_string_pretty(&report)
        .map_err(|e| crate::Error::config(format!("Failed to serialize AI report: {}", e)))?;
    fs::write(&json_path, json_content).await?;

    // Save human-readable markdown
    let md_path = reports_dir.join(format!("ai_compliance_{}.md", timestamp_str));
    let md_content = generate_markdown_report(&report);
    fs::write(&md_path, md_content).await?;

    // Create latest links
    let latest_json = reports_dir.join("latest_ai_report.json");
    let latest_md = reports_dir.join("latest_ai_report.md");
    fs::copy(&json_path, &latest_json).await?;
    fs::copy(&md_path, &latest_md).await?;

    println!("📊 AI Compliance Report Generated:");
    println!("  📄 JSON: {}", json_path.display());
    println!("  📝 Markdown: {}", md_path.display());
    println!("  🔗 Latest JSON: {}", latest_json.display());
    println!("  🔗 Latest MD: {}", latest_md.display());
    println!("\n🤖 This report is optimized for AI assistant consumption");
    println!("   Use the JSON file for automated processing and fix suggestions");

    Ok(())
}

/// Get code snippet around a violation
async fn get_code_snippet(file_path: &PathBuf, line: usize) -> Result<String> {
    if !file_path.exists() {
        return Ok("File not found".to_string());
    }

    let contents = fs::read_to_string(file_path).await?;
    let lines: Vec<&str> = contents.lines().collect();

    if line < lines.len() {
        Ok(lines[line].to_string())
    } else {
        Ok("Line not found".to_string())
    }
}

/// Count Rust files in project
async fn count_rust_files(project_path: &PathBuf) -> Result<usize> {
    let mut count = 0;

    // Simple count for now - would need recursive implementation for full accuracy
    let mut entries = fs::read_dir(project_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if let Some(ext) = entry.path().extension() {
            if ext == "rs" {
                count += 1;
            }
        }
    }

    Ok(count.max(1))
}

/// Generate human-readable markdown from AI report
fn generate_markdown_report(report: &AIReport) -> String {
    let mut md = String::new();

    md.push_str("# 🤖 AI-Friendly Compliance Report\n\n");
    md.push_str(&format!("**Generated**: {}\n", report.metadata.timestamp));
    md.push_str(&format!("**Project**: {}\n", report.metadata.project_path));
    md.push_str(&format!(
        "**Total Violations**: {}\n",
        report.metadata.total_violations
    ));
    md.push_str(&format!(
        "**Compliance**: {:.1}%\n\n",
        report.summary.compliance_percentage
    ));

    md.push_str("## 🎯 Fix Priority Order\n\n");
    md.push_str("1. **UnwrapInProduction** - Critical for safety\n");
    md.push_str("2. **UnderscoreBandaid** - Implement missing functionality\n");
    md.push_str("3. **FunctionTooLarge** - Refactor for maintainability\n");
    md.push_str("4. **FileTooLarge** - Split into modules\n\n");

    md.push_str("## 🔧 Automated Fix Commands\n\n");
    md.push_str("```bash\n");
    md.push_str("# Generate this report\n");
    md.push_str("ferrous-forge validate . --ai-report\n\n");
    md.push_str("# Use AI assistant with the JSON report to implement fixes\n");
    md.push_str("# The JSON contains structured data for automated processing\n");
    md.push_str("```\n\n");

    md.push_str("## 📊 Violation Summary\n\n");
    for instruction in &report.fix_instructions {
        md.push_str(&format!(
            "### {} ({} violations)\n",
            instruction.violation_type, instruction.count
        ));
        md.push_str(&format!("**Strategy**: {}\n\n", instruction.fix_strategy));
        md.push_str(&format!(
            "**Example**: \n```rust\n{}\n```\n\n",
            instruction.example_fix
        ));
    }

    md
}

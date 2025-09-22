//! Markdown report generation

use super::ai_report::AIReport;

/// Generate a markdown-formatted report from an AI compliance report
pub fn generate_markdown_report(report: &AIReport) -> String {
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
            "### {} ({} violations)\n\n",
            instruction.violation_type, instruction.count
        ));
        md.push_str(&format!("**Strategy**: {}\n\n", instruction.fix_strategy));
        md.push_str(&format!(
            "**Example**:\n```\n{}\n```\n\n",
            instruction.example_fix
        ));
    }
    
    md
}
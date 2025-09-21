//! AI analysis module for automated violation analysis

/// Main AI analyzer implementation
pub mod analyzer;
/// Code context extraction utilities
pub mod context;
/// Semantic analysis of code violations
pub mod semantic;
/// Fix strategy generation
pub mod strategies;
/// Type definitions for AI analysis
pub mod types;

pub use analyzer::AIAnalyzer;
pub use types::*;

use crate::validation::Violation;
use std::path::Path;

/// Legacy async API for backwards compatibility
pub async fn analyze_violations_for_ai(
    violations: &[Violation],
    project_path: &Path,
) -> anyhow::Result<AIAnalysisReport> {
    let analyzer = AIAnalyzer::new(project_path.to_path_buf());
    analyzer.analyze_violations_async(violations.to_vec()).await
}

/// Analyze violations and generate a comprehensive report
pub async fn analyze_and_generate_report(
    project_path: &Path,
    violations: &[Violation],
) -> anyhow::Result<()> {
    let analyzer = AIAnalyzer::new(project_path.to_path_buf());
    let report = analyzer
        .analyze_violations_async(violations.to_vec())
        .await?;

    // Save the analysis report
    analyzer.save_analysis(&report)?;

    // Generate and save orchestrator instructions
    let instructions = generate_orchestrator_instructions(&report).await?;
    analyzer.save_orchestrator_instructions(&instructions)?;

    Ok(())
}

/// Generate orchestrator instructions from analysis report
pub async fn generate_orchestrator_instructions(
    report: &AIAnalysisReport,
) -> anyhow::Result<AIAnalysisReport> {
    println!("📝 Generated orchestrator instructions");
    // Instructions are already generated as part of the report
    Ok(report.clone())
}

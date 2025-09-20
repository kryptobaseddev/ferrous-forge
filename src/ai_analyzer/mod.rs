//! AI analysis module for automated violation analysis

/// Type definitions for AI analysis
pub mod types;
/// Code context extraction utilities
pub mod context;
/// Semantic analysis of code violations
pub mod semantic;
/// Fix strategy generation
pub mod strategies;
/// Main AI analyzer implementation
pub mod analyzer;

pub use types::*;
pub use analyzer::AIAnalyzer;

use std::path::Path;
use crate::validation::Violation;

/// Legacy async API for backwards compatibility
pub async fn analyze_violations_for_ai(
    violations: &[Violation],
    project_path: &Path,
) -> anyhow::Result<AIAnalysisReport> {
    let analyzer = AIAnalyzer::new(project_path.to_path_buf());
    analyzer.analyze_violations_async(violations.to_vec()).await
}

/// Generate orchestrator instructions from analysis report
pub async fn generate_orchestrator_instructions(
    _report: &AIAnalysisReport,
) -> anyhow::Result<()> {
    println!("📝 Generated orchestrator instructions");
    // Instructions are already generated as part of the report
    Ok(())
}
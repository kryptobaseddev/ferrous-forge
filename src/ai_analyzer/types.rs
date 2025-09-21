use serde::{Deserialize, Serialize};

use crate::validation::ViolationType;

/// Main AI analysis report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisReport {
    /// Analysis metadata
    pub metadata: AnalysisMetadata,
    /// Individual violation analyses
    pub violation_analyses: Vec<ViolationAnalysis>,
    /// Detected code patterns
    pub code_patterns: CodePatterns,
    /// Generated fix strategies
    pub fix_strategies: Vec<FixStrategy>,
    /// AI instructions for fixes
    pub ai_instructions: AIInstructions,
}

/// Metadata about AI analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Total violations found
    pub total_violations: usize,
    /// Violations that can be analyzed
    pub analyzable_violations: usize,
    /// Project path analyzed
    pub project_path: String,
    /// Depth of analysis performed
    pub analysis_depth: AnalysisDepth,
}

/// Depth level of AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    /// Basic surface analysis
    Surface,
    /// Context-aware analysis
    Contextual,
    /// Semantic code analysis
    Semantic,
    /// Architectural level analysis
    Architectural,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationAnalysis {
    pub violation: crate::validation::Violation,
    pub code_context: CodeContext,
    pub semantic_analysis: SemanticAnalysis,
    pub fix_complexity: FixComplexity,
    pub ai_fixable: bool,
    pub fix_recommendation: Option<String>,
    pub side_effects: Vec<String>,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub function_name: Option<String>,
    pub function_signature: Option<String>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub is_generic: bool,
    pub trait_impl: Option<String>,
    pub surrounding_code: Vec<String>,
    pub imports: Vec<String>,
    pub error_handling_style: ErrorHandlingStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStyle {
    StdResult,
    CustomResult,
    OptionBased,
    AnyhowResult,
    Panic,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    pub actual_type: Option<String>,
    pub expected_type: Option<String>,
    pub data_flow: Vec<String>,
    pub control_flow: Vec<String>,
    pub dependencies: Vec<String>,
    pub error_propagation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixComplexity {
    Trivial,
    Simple,
    Moderate,
    Complex,
    Architectural,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePatterns {
    pub architectural_style: ArchitecturalStyle,
    pub error_patterns: Vec<ErrorPattern>,
    pub common_patterns: Vec<Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub occurrences: usize,
    pub locations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturalStyle {
    Monolithic,
    Modular,
    Microservices,
    EventDriven,
    Layered,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorPattern {
    UnhandledResult,
    PanicUsage,
    UnwrapUsage,
    ExpectUsage,
    IgnoredError,
    PropagatedError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStrategy {
    pub violation_type: ViolationType,
    pub strategy_name: String,
    pub description: String,
    pub implementation_steps: Vec<String>,
    pub estimated_effort: String,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInstructions {
    pub summary: String,
    pub prioritized_fixes: Vec<String>,
    pub architectural_recommendations: Vec<String>,
    pub code_quality_improvements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationPrompt {
    pub violation_summary: String,
    pub context_description: String,
    pub fix_approach: String,
    pub constraints: Vec<String>,
}
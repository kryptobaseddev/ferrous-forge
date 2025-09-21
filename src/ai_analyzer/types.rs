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

/// Analysis of a code violation with AI-powered insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationAnalysis {
    /// The underlying violation being analyzed
    pub violation: crate::validation::Violation,
    /// Contextual information about the code
    pub code_context: CodeContext,
    /// Semantic analysis of the violation
    pub semantic_analysis: SemanticAnalysis,
    /// Complexity level of the required fix
    pub fix_complexity: FixComplexity,
    /// Whether AI can automatically fix this
    pub ai_fixable: bool,
    /// Recommended fix strategy
    pub fix_recommendation: Option<String>,
    /// Potential side effects of applying the fix
    pub side_effects: Vec<String>,
    /// Confidence in the fix recommendation (0-100)
    pub confidence_score: f32,
}

/// Context information about the code being analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    /// Name of the function containing the violation
    pub function_name: Option<String>,
    /// Full function signature
    pub function_signature: Option<String>,
    /// Return type of the function
    pub return_type: Option<String>,
    /// Whether the function is async
    pub is_async: bool,
    /// Whether the function uses generics
    pub is_generic: bool,
    /// Trait implementation context if any
    pub trait_impl: Option<String>,
    /// Lines of code surrounding the violation
    pub surrounding_code: Vec<String>,
    /// Import statements in the file
    pub imports: Vec<String>,
    /// Error handling style used in the code
    pub error_handling_style: ErrorHandlingStyle,
}

/// Error handling style used in the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStyle {
    /// Standard Result type
    StdResult,
    /// Custom Result type
    CustomResult,
    /// Option-based error handling
    OptionBased,
    /// Anyhow Result type
    AnyhowResult,
    /// Uses panic for error handling
    Panic,
    /// Unknown error handling style
    Unknown,
}

/// Semantic analysis of code patterns and types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    /// Actual type found in code
    pub actual_type: Option<String>,
    /// Expected type based on context
    pub expected_type: Option<String>,
    /// Data flow analysis results
    pub data_flow: Vec<String>,
    /// Control flow analysis results
    pub control_flow: Vec<String>,
    /// Dependencies and imports required
    pub dependencies: Vec<String>,
    /// How errors propagate through code
    pub error_propagation: Vec<String>,
}

/// Complexity level of a fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixComplexity {
    /// Trivial fix - simple replacement
    Trivial,
    /// Simple fix - straightforward change
    Simple,
    /// Moderate fix - requires some refactoring
    Moderate,
    /// Complex fix - significant changes needed
    Complex,
    /// Architectural fix - design changes required
    Architectural,
}

/// Code patterns found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePatterns {
    /// Overall architectural style
    pub architectural_style: ArchitecturalStyle,
    /// Error handling patterns found
    pub error_patterns: Vec<ErrorPattern>,
    /// Common code patterns detected
    pub common_patterns: Vec<Pattern>,
}

/// A code pattern occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Name of the pattern
    pub name: String,
    /// Number of occurrences
    pub occurrences: usize,
    /// Locations where pattern appears
    pub locations: Vec<String>,
}

/// Architectural style of the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturalStyle {
    /// Monolithic architecture
    Monolithic,
    /// Modular architecture
    Modular,
    /// Microservices architecture
    Microservices,
    /// Event-driven architecture
    EventDriven,
    /// Layered architecture
    Layered,
    /// Unknown architecture style
    Unknown,
}

/// Common error handling patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorPattern {
    /// Unhandled Result type
    UnhandledResult,
    /// Using panic for errors
    PanicUsage,
    /// Using unwrap
    UnwrapUsage,
    /// Using expect
    ExpectUsage,
    /// Ignoring errors
    IgnoredError,
    /// Propagating errors with ?
    PropagatedError,
}

/// Strategy for fixing a violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStrategy {
    /// Type of violation this strategy addresses
    pub violation_type: ViolationType,
    /// Name of the strategy
    pub strategy_name: String,
    /// Description of the strategy
    pub description: String,
    /// Steps to implement the fix
    pub implementation_steps: Vec<String>,
    /// Estimated effort required
    pub estimated_effort: String,
    /// Risk level of the fix
    pub risk_level: String,
}

/// Instructions for AI-based fixes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInstructions {
    /// Summary of the instructions
    pub summary: String,
    /// List of prioritized fixes
    pub prioritized_fixes: Vec<String>,
    /// Architectural recommendations
    pub architectural_recommendations: Vec<String>,
    /// Code quality improvement suggestions
    pub code_quality_improvements: Vec<String>,
}

/// Prompt for generating violation fixes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationPrompt {
    /// Summary of the violation
    pub violation_summary: String,
    /// Description of the context
    pub context_description: String,
    /// Approach for fixing the violation
    pub fix_approach: String,
    /// Constraints to consider
    pub constraints: Vec<String>,
}

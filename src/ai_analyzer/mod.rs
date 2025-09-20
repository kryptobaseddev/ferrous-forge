//! AI-Powered Code Analysis for Intelligent Fixing
//!
//! This module performs deep semantic analysis of code violations to generate
//! comprehensive context and instructions for AI-powered fixing through the
//! Claude Orchestrator or other LLM agents.

mod types;
mod analysis;
mod context;
mod strategies;

pub use types::*;
pub use analysis::analyze_violations;
pub use context::extract_code_context;
pub use strategies::generate_fix_strategies;
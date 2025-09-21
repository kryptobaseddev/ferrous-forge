use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;

use super::types::*;
use super::context::extract_code_context;
use super::semantic::{perform_semantic_analysis, assess_fix_complexity};
use super::strategies::{generate_fix_strategies, generate_ai_instructions, identify_code_patterns};
use crate::validation::Violation;

/// AI analyzer for automated violation analysis
pub struct AIAnalyzer {
    project_root: PathBuf,
}

impl AIAnalyzer {
    /// Create a new AI analyzer
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Analyze violations and generate report
    pub fn analyze_violations(&self, violations: Vec<Violation>) -> Result<AIAnalysisReport> {
        let mut violation_analyses = Vec::new();
        let mut analyzable_count = 0;

        for violation in &violations {
            if let Ok(analysis) = self.analyze_single_violation(&violation) {
                if analysis.ai_fixable {
                    analyzable_count += 1;
                }
                violation_analyses.push(analysis);
            }
        }

        let code_patterns = self.analyze_project_patterns()?;
        let fix_strategies = generate_fix_strategies(&violation_analyses);
        let ai_instructions = generate_ai_instructions(&violation_analyses, &fix_strategies);

        let metadata = AnalysisMetadata {
            total_violations: violations.len(),
            analyzable_violations: analyzable_count,
            project_path: self.project_root.display().to_string(),
            analysis_depth: AnalysisDepth::Semantic,
        };

        Ok(AIAnalysisReport {
            metadata,
            violation_analyses,
            code_patterns,
            fix_strategies,
            ai_instructions,
        })
    }

    fn analyze_single_violation(&self, violation: &Violation) -> Result<ViolationAnalysis> {
        let content = fs::read_to_string(&violation.file)?;
        let code_context = extract_code_context(violation.line, &content);
        let semantic_analysis = perform_semantic_analysis(violation, &code_context, &content);
        let fix_complexity = assess_fix_complexity(violation, &code_context, &semantic_analysis);
        
        let (ai_fixable, confidence_score) = self.assess_fixability(
            &violation,
            &code_context,
            &semantic_analysis,
            &fix_complexity,
        );

        let fix_recommendation = if ai_fixable {
            self.generate_fix_recommendation(&violation, &code_context, &semantic_analysis)
        } else {
            None
        };

        let side_effects = self.identify_side_effects(&violation, &code_context);

        Ok(ViolationAnalysis {
            violation: violation.clone(),
            code_context,
            semantic_analysis,
            fix_complexity,
            ai_fixable,
            fix_recommendation,
            side_effects,
            confidence_score,
        })
    }

    fn assess_fixability(
        &self,
        violation: &Violation,
        context: &CodeContext,
        _semantic: &SemanticAnalysis,
        complexity: &FixComplexity,
    ) -> (bool, f32) {
        match (&violation.violation_type, complexity) {
            (crate::validation::ViolationType::UnwrapInProduction, FixComplexity::Trivial) => {
                if context.return_type.as_ref().map_or(false, |r| r.contains("Result")) {
                    (true, 0.95)
                } else {
                    (true, 0.75)
                }
            }
            (crate::validation::ViolationType::UnwrapInProduction, FixComplexity::Simple) => {
                (true, 0.65)
            }
            (crate::validation::ViolationType::LineTooLong, _) => (true, 1.0),
            (crate::validation::ViolationType::UnderscoreBandaid, _) => (true, 0.85),
            (crate::validation::ViolationType::FunctionTooLarge, _) => (false, 0.3),
            (crate::validation::ViolationType::FileTooLarge, _) => (false, 0.2),
            _ => (false, 0.0),
        }
    }

    fn generate_fix_recommendation(
        &self,
        violation: &Violation,
        context: &CodeContext,
        _semantic: &SemanticAnalysis,
    ) -> Option<String> {
        match violation.violation_type {
            crate::validation::ViolationType::UnwrapInProduction => {
                if context.return_type.as_ref().map_or(false, |r| r.contains("Result")) {
                    Some("Replace ? with ? operator".to_string())
                } else {
                    Some("Change function return type to Result and use ?".to_string())
                }
            }
            crate::validation::ViolationType::LineTooLong => {
                Some("Break line at appropriate point (e.g., after comma, operator)".to_string())
            }
            crate::validation::ViolationType::UnderscoreBandaid => {
                Some("Either use the parameter or remove it from function signature".to_string())
            }
            _ => None,
        }
    }

    fn identify_side_effects(
        &self,
        violation: &Violation,
        context: &CodeContext,
    ) -> Vec<String> {
        let mut effects = Vec::new();

        match violation.violation_type {
            crate::validation::ViolationType::UnwrapInProduction => {
                if !context.return_type.as_ref().map_or(false, |r| r.contains("Result")) {
                    effects.push("Function signature change required".to_string());
                    effects.push("All callers must be updated".to_string());
                }
            }
            crate::validation::ViolationType::FunctionTooLarge => {
                effects.push("May require creating new helper functions".to_string());
                effects.push("Could affect function testing".to_string());
            }
            _ => {}
        }

        effects
    }

    fn analyze_project_patterns(&self) -> Result<CodePatterns> {
        let mut all_content = String::new();
        
        // Sample a few files for pattern analysis
        use std::fs;
        let mut count = 0;
        
        // Simple file traversal
        fn visit_dir(
            dir: &std::path::Path, 
            content: &mut String, 
            count: &mut usize, 
            max: usize
        ) -> Result<()> {
            if *count >= max {
                return Ok(());
            }
            
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() 
                    && !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') 
                {
                    visit_dir(&path, content, count, max)?;
                } else if path.extension().map_or(false, |ext| ext == "rs") {
                    if let Ok(file_content) = fs::read_to_string(&path) {
                        content.push_str(&file_content);
                        *count += 1;
                        if *count >= max {
                            break;
                        }
                    }
                }
            }
            Ok(())
        }
        
        visit_dir(
            &self.project_root.join("src"), 
            &mut all_content, 
            &mut count, 
            10
        )?;

        Ok(identify_code_patterns(&all_content))
    }

    /// Async version of analyze_violations
    pub async fn analyze_violations_async(
        &self, 
        violations: Vec<Violation>
    ) -> Result<AIAnalysisReport> {
        // For now, just call the sync version
        // In future could parallelize with tokio
        self.analyze_violations(violations)
    }

    /// Save analysis report to disk
    pub fn save_analysis(&self, report: &AIAnalysisReport) -> Result<()> {
        let analysis_dir = self.project_root.join(".ferrous-forge").join("ai-analysis");
        fs::create_dir_all(&analysis_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("ai_analysis_{}.json", timestamp);
        let filepath = analysis_dir.join(&filename);

        let json = serde_json::to_string_pretty(&report)?;
        fs::write(&filepath, json)?;

        println!("📊 AI analysis saved to: {}", filepath.display());

        // Also save orchestrator instructions
        self.save_orchestrator_instructions(report)?;

        Ok(())
    }

    pub fn save_orchestrator_instructions(&self, report: &AIAnalysisReport) -> Result<()> {
        let analysis_dir = self.project_root.join(".ferrous-forge").join("ai-analysis");
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("orchestrator_instructions_{}.md", timestamp);
        let filepath = analysis_dir.join(&filename);

        let mut instructions = String::new();
        instructions.push_str("# AI Orchestrator Instructions\n\n");
        instructions.push_str(&format!("## Summary\n{}\n\n", report.ai_instructions.summary));
        
        instructions.push_str("## Prioritized Fixes\n");
        for fix in &report.ai_instructions.prioritized_fixes {
            instructions.push_str(&format!("- {}\n", fix));
        }
        
        instructions.push_str("\n## Architectural Recommendations\n");
        for rec in &report.ai_instructions.architectural_recommendations {
            instructions.push_str(&format!("- {}\n", rec));
        }

        fs::write(&filepath, instructions)?;
        println!("📝 Orchestrator instructions saved to: {}", filepath.display());

        Ok(())
    }
}
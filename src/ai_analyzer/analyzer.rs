use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;

use super::context::extract_code_context;
use super::semantic::{assess_fix_complexity, perform_semantic_analysis};
use super::strategies::{
    generate_ai_instructions, generate_fix_strategies, identify_code_patterns,
};
use super::types::*;
use crate::validation::{Violation, ViolationType};

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
    ///
    /// # Errors
    ///
    /// Returns an error if reading source files or analyzing project patterns fails.
    pub fn analyze_violations(&self, violations: Vec<Violation>) -> Result<AIAnalysisReport> {
        let mut violation_analyses = Vec::new();
        let mut analyzable_count = 0;

        for violation in &violations {
            if let Ok(analysis) = self.analyze_single_violation(violation) {
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
        // Locked settings are never AI-fixable — return early with explicit guidance
        if violation.is_locked_setting() {
            return Ok(self.build_locked_analysis(violation));
        }

        let content = fs::read_to_string(&violation.file)?;
        let code_context = extract_code_context(violation.line, &content);
        let semantic_analysis = perform_semantic_analysis(violation, &code_context, &content);
        let fix_complexity = assess_fix_complexity(violation, &code_context, &semantic_analysis);

        let (ai_fixable, confidence_score) = self.assess_fixability(
            violation,
            &code_context,
            &semantic_analysis,
            &fix_complexity,
        );

        let fix_recommendation = if ai_fixable {
            self.generate_fix_recommendation(violation, &code_context, &semantic_analysis)
        } else {
            None
        };

        let side_effects = self.identify_side_effects(violation, &code_context);

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

    /// Build a [`ViolationAnalysis`] for locked settings (edition, rust-version)
    fn build_locked_analysis(&self, violation: &Violation) -> ViolationAnalysis {
        ViolationAnalysis {
            violation: violation.clone(),
            code_context: CodeContext {
                function_name: None,
                function_signature: None,
                return_type: None,
                is_async: false,
                is_generic: false,
                trait_impl: None,
                surrounding_code: vec![],
                imports: vec![],
                error_handling_style: ErrorHandlingStyle::Unknown,
            },
            semantic_analysis: super::semantic::empty_semantic_analysis(),
            fix_complexity: FixComplexity::Architectural,
            ai_fixable: false,
            fix_recommendation: Some(
                "DO NOT change edition or rust-version in Cargo.toml.\n\
                 These are locked by .ferrous-forge/config.toml.\n\
                 This violation requires human intervention — escalate to the project owner."
                    .to_string(),
            ),
            side_effects: vec![
                "Changing locked settings may break CI, team standards, and edition guarantees."
                    .to_string(),
            ],
            confidence_score: 0.0,
        }
    }

    fn assess_fixability(
        &self,
        violation: &Violation,
        context: &CodeContext,
        _semantic: &SemanticAnalysis,
        complexity: &FixComplexity,
    ) -> (bool, f32) {
        match (&violation.violation_type, complexity) {
            (ViolationType::UnwrapInProduction, FixComplexity::Trivial) => {
                if context
                    .return_type
                    .as_ref()
                    .is_some_and(|r| r.contains("Result"))
                {
                    (true, 0.95)
                } else {
                    (true, 0.75)
                }
            }
            (ViolationType::UnwrapInProduction, FixComplexity::Simple) => (true, 0.65),
            (ViolationType::LineTooLong, _) => (true, 1.0),
            (ViolationType::UnderscoreBandaid, _) => (true, 0.85),
            (ViolationType::FunctionTooLarge, _) => (false, 0.3),
            (ViolationType::FileTooLarge, _) => (false, 0.2),
            // Locked settings are never AI-fixable (handled in build_locked_analysis)
            (ViolationType::WrongEdition, _)
            | (ViolationType::OldRustVersion, _)
            | (ViolationType::LockedSetting, _) => (false, 0.0),
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
            ViolationType::UnwrapInProduction => {
                if context
                    .return_type
                    .as_ref()
                    .is_some_and(|r| r.contains("Result"))
                {
                    Some("Replace ? with ? operator".to_string())
                } else {
                    Some("Change function return type to Result and use ?".to_string())
                }
            }
            ViolationType::LineTooLong => {
                Some("Break line at appropriate point (e.g., after comma, operator)".to_string())
            }
            ViolationType::UnderscoreBandaid => {
                Some("Either use the parameter or remove it from function signature".to_string())
            }
            _ => None,
        }
    }

    fn identify_side_effects(&self, violation: &Violation, context: &CodeContext) -> Vec<String> {
        let mut effects = Vec::new();

        match violation.violation_type {
            ViolationType::UnwrapInProduction
                if !context
                    .return_type
                    .as_ref()
                    .is_some_and(|r| r.contains("Result")) =>
            {
                effects.push("Function signature change required".to_string());
                effects.push("All callers must be updated".to_string());
            }
            ViolationType::FunctionTooLarge => {
                effects.push("May require creating new helper functions".to_string());
                effects.push("Could affect function testing".to_string());
            }
            _ => {}
        }

        effects
    }

    fn analyze_project_patterns(&self) -> Result<CodePatterns> {
        let mut all_content = String::new();
        let mut count = 0;

        fn visit_dir(
            dir: &std::path::Path,
            content: &mut String,
            count: &mut usize,
            max: usize,
        ) -> Result<()> {
            if *count >= max {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir()
                    && !path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .starts_with('.')
                {
                    visit_dir(&path, content, count, max)?;
                } else if path.extension().is_some_and(|ext| ext == "rs")
                    && let Ok(file_content) = fs::read_to_string(&path)
                {
                    content.push_str(&file_content);
                    *count += 1;
                    if *count >= max {
                        break;
                    }
                }
            }
            Ok(())
        }

        visit_dir(
            &self.project_root.join("src"),
            &mut all_content,
            &mut count,
            10,
        )?;

        Ok(identify_code_patterns(&all_content))
    }

    /// Async version of [`analyze_violations`](Self::analyze_violations)
    ///
    /// # Errors
    ///
    /// Returns an error if reading source files or analyzing project patterns fails.
    pub async fn analyze_violations_async(
        &self,
        violations: Vec<Violation>,
    ) -> Result<AIAnalysisReport> {
        self.analyze_violations(violations)
    }

    /// Save analysis report to disk
    ///
    /// # Errors
    ///
    /// Returns an error if creating the analysis directory, serializing the report,
    /// or writing files fails.
    pub fn save_analysis(&self, report: &AIAnalysisReport) -> Result<()> {
        let analysis_dir = self.project_root.join(".ferrous-forge").join("ai-analysis");
        fs::create_dir_all(&analysis_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("ai_analysis_{}.json", timestamp);
        let filepath = analysis_dir.join(&filename);

        let json = serde_json::to_string_pretty(&report)?;
        fs::write(&filepath, json)?;

        println!("📊 AI analysis saved to: {}", filepath.display());

        self.save_orchestrator_instructions(report)?;

        Ok(())
    }

    /// Save orchestrator instructions to file, including locked settings section
    ///
    /// # Errors
    ///
    /// Returns an error if writing the instructions file fails.
    pub fn save_orchestrator_instructions(&self, report: &AIAnalysisReport) -> Result<()> {
        let analysis_dir = self.project_root.join(".ferrous-forge").join("ai-analysis");
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("orchestrator_instructions_{}.md", timestamp);
        let filepath = analysis_dir.join(&filename);

        let mut instructions = String::new();
        instructions.push_str("# AI Orchestrator Instructions\n\n");

        // Locked settings section — appears first so agents see it immediately
        let locked_analyses: Vec<&ViolationAnalysis> = report
            .violation_analyses
            .iter()
            .filter(|a| a.violation.is_locked_setting())
            .collect();

        if !locked_analyses.is_empty() {
            instructions.push_str("## Locked Settings (DO NOT MODIFY)\n\n");
            instructions.push_str(
                "The following are locked by Ferrous Forge project configuration.\n\
                 DO NOT change these to resolve compilation errors — escalate to human.\n\n",
            );
            instructions.push_str("| Setting | Violation | Config |\n");
            instructions.push_str("|---------|-----------|--------|\n");
            for a in &locked_analyses {
                let setting = match a.violation.violation_type {
                    ViolationType::WrongEdition => "edition",
                    ViolationType::OldRustVersion => "rust-version",
                    _ => "locked setting",
                };
                instructions.push_str(&format!(
                    "| {} | {} | .ferrous-forge/config.toml |\n",
                    setting,
                    a.violation.file.display()
                ));
            }
            instructions.push_str(
                "\n**AI Agent Rule**: `ai_fixable = false`, \
                 `confidence = 0%` for all locked violations above.\n\n",
            );
        }

        instructions.push_str(&format!(
            "## Summary\n{}\n\n",
            report.ai_instructions.summary
        ));

        instructions.push_str("## Prioritized Fixes\n");
        for fix in &report.ai_instructions.prioritized_fixes {
            instructions.push_str(&format!("- {}\n", fix));
        }

        instructions.push_str("\n## Architectural Recommendations\n");
        for rec in &report.ai_instructions.architectural_recommendations {
            instructions.push_str(&format!("- {}\n", rec));
        }

        fs::write(&filepath, instructions)?;
        println!(
            "📝 Orchestrator instructions saved to: {}",
            filepath.display()
        );

        Ok(())
    }
}

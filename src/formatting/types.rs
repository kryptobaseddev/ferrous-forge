//! Formatting types and reporting

use serde::{Deserialize, Serialize};

/// Formatting check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResult {
    /// Whether the code is properly formatted
    pub formatted: bool,
    /// Files that need formatting
    pub unformatted_files: Vec<String>,
    /// Suggested changes
    pub suggestions: Vec<FormatSuggestion>,
}

/// A formatting suggestion for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatSuggestion {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Description of the formatting issue
    pub description: String,
}

impl FormatResult {
    /// Generate a human-readable report
    pub fn report(&self) -> String {
        let mut report = String::new();

        if self.formatted {
            report.push_str("✅ Code formatting check passed - All files properly formatted!\n");
        } else {
            report.push_str(&format!(
                "⚠️ Code formatting issues found in {} files\n\n",
                self.unformatted_files.len()
            ));

            report.push_str("Files needing formatting:\n");
            for file in &self.unformatted_files {
                report.push_str(&format!("  • {}\n", file));
            }

            if !self.suggestions.is_empty() {
                report.push_str("\nFormatting suggestions:\n");
                for suggestion in &self.suggestions {
                    report.push_str(&format!(
                        "  {}:{} - {}\n",
                        suggestion.file, suggestion.line, suggestion.description
                    ));
                }
            }

            report.push_str("\nRun 'ferrous-forge fix --format' to auto-fix these issues.\n");
        }

        report
    }
}

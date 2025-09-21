//! Documentation checking - placeholder for now, using standards module

use crate::Result;
use std::path::Path;
use std::time::Instant;

use super::SafetyCheck;
use crate::safety::{report::CheckResult, CheckType};

/// Doc check implementation
pub struct DocCheck;

impl SafetyCheck for DocCheck {
    async fn run(project_path: &Path) -> Result<CheckResult> {
        run(project_path).await
    }

    fn name() -> &'static str {
        "doc"
    }

    fn description() -> &'static str {
        "Builds project documentation"
    }
}

/// Run documentation build check (placeholder)
pub async fn run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::Doc);
    result.set_duration(start.elapsed());
    result.add_context(&format!(
        "Documentation check placeholder for {} - always passes",
        project_path.display()
    ));
    Ok(result)
}

/// Check documentation coverage (placeholder)
pub async fn coverage_check(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult::new(CheckType::DocCoverage);
    result.set_duration(start.elapsed());
    result.add_context(&format!(
        "Documentation coverage check placeholder for {} - always passes",
        project_path.display()
    ));
    Ok(result)
}

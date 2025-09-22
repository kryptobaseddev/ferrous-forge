//! File and function size validation

use crate::validation::{Severity, Violation, ViolationType};
use crate::Result;
use std::path::Path;

/// Validate file size constraints
pub fn validate_file_size(
    rust_file: &Path,
    lines: &[&str],
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let line_count = lines.len();

    if line_count > 300 {
        violations.push(Violation {
            violation_type: ViolationType::FileTooLarge,
            file: rust_file.to_path_buf(),
            line: line_count,
            message: format!(
                "File has {} lines, maximum allowed is 300",
                line_count
            ),
            severity: Severity::Error,
        });
    }

    Ok(())
}
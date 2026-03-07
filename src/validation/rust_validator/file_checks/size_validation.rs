//! File and function size validation

use crate::Result;
use crate::validation::{Severity, Violation, ViolationType};
use std::path::Path;

/// Validate file size constraints
pub fn validate_file_size(
    rust_file: &Path,
    lines: &[&str],
    violations: &mut Vec<Violation>,
    max_lines: usize,
) -> Result<()> {
    let line_count = lines.len();

    if line_count > max_lines {
        violations.push(Violation {
            violation_type: ViolationType::FileTooLarge,
            file: rust_file.to_path_buf(),
            line: line_count,
            message: format!(
                "File has {} lines, maximum allowed is {}",
                line_count, max_lines
            ),
            severity: Severity::Error,
        });
    }

    Ok(())
}

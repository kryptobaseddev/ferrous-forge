//! Test detection utilities

use std::path::Path;

/// Check if a file is a test file based on its path
pub fn is_test_file(rust_file: &Path) -> bool {
    rust_file
        .to_str()
        .map(|s| s.contains("/tests/") || s.contains("\\tests\\") || s.ends_with("_test.rs"))
        .unwrap_or(false)
}

/// Check for allow attributes in file
pub fn check_allow_attributes(lines: &[&str]) -> (bool, bool) {
    let mut allow_unwrap = false;
    let mut allow_expect = false;

    for line in lines {
        let trimmed = line.trim();

        // Check for file-level allows
        if trimmed.starts_with("#![allow(") && trimmed.contains("unwrap_used") {
            allow_unwrap = true;
        }
        if trimmed.starts_with("#![allow(") && trimmed.contains("expect_used") {
            allow_expect = true;
        }

        // Check for clippy::unwrap_used or clippy::expect_used
        if trimmed.contains("clippy::unwrap_used") {
            allow_unwrap = true;
        }
        if trimmed.contains("clippy::expect_used") {
            allow_expect = true;
        }
    }

    (allow_unwrap, allow_expect)
}

//! Test detection utilities

use std::path::Path;

/// Check if a file is a test file based on its path
pub fn is_test_file(rust_file: &Path) -> bool {
    rust_file
        .to_str()
        .map(|s| s.contains("/tests/") || s.contains("\\tests\\") || s.ends_with("_test.rs"))
        .unwrap_or(false)
}

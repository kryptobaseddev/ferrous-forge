//! Code formatting and auto-correction module
//!
//! This module provides integration with rustfmt for code formatting
//! validation and automatic correction.

mod file_ops;
mod project_ops;
mod types;
mod utils;

// Re-export public types and functions
pub use file_ops::{check_file_formatting, format_file};
pub use project_ops::{apply_rustfmt_config, auto_format, check_formatting, get_format_diff};
pub use types::{FormatResult, FormatSuggestion};

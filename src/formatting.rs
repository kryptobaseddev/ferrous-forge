//! Code formatting and auto-correction module
//!
//! This module provides integration with rustfmt for code formatting
//! validation and automatic correction.

mod types;
mod project_ops;
mod file_ops;
mod utils;

// Re-export public types and functions
pub use types::{FormatResult, FormatSuggestion};
pub use project_ops::{check_formatting, auto_format, get_format_diff, apply_rustfmt_config};
pub use file_ops::{check_file_formatting, format_file};
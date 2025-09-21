//! Validation patterns for Rust code

use crate::{Error, Result};
use regex::Regex;

/// Compiled regex patterns for validation
pub struct ValidationPatterns {
    /// Pattern for detecting function definitions
    pub function_def: Regex,
    /// Pattern for detecting underscore parameters
    pub underscore_param: Regex,
    /// Pattern for detecting underscore in let bindings
    pub underscore_let: Regex,
    /// Pattern for detecting unwrap() calls
    pub unwrap_call: Regex,
    /// Pattern for detecting expect() calls
    pub expect_call: Regex,
}

impl ValidationPatterns {
    /// Create and compile all validation patterns
    pub fn new() -> Result<Self> {
        Ok(Self {
            function_def: Regex::new(r"^\s*(pub\s+)?(async\s+)?fn\s+")
                .map_err(|e| Error::validation(format!("Invalid function regex: {}", e)))?,

            underscore_param: Regex::new(r"fn\s+\w+[^{]*\b_\w+\s*:")
                .map_err(|e| Error::validation(format!("Invalid underscore param regex: {}", e)))?,

            underscore_let: Regex::new(r"^\s*let\s+_\s*=")
                .map_err(|e| Error::validation(format!("Invalid underscore let regex: {}", e)))?,

            unwrap_call: Regex::new(r"\.unwrap\(\)")
                .map_err(|e| Error::validation(format!("Invalid unwrap regex: {}", e)))?,

            expect_call: Regex::new(r"\.expect\(")
                .map_err(|e| Error::validation(format!("Invalid expect regex: {}", e)))?,
        })
    }
}

/// Helper function to check if a pattern is inside a string literal
pub fn is_in_string_literal(line: &str, pattern: &str) -> bool {
    let mut in_string = false;
    let mut escaped = false;
    let chars = line.chars();
    let mut pos = 0;

    for c in chars {
        if escaped {
            escaped = false;
            pos += c.len_utf8();
            continue;
        }

        match c {
            '\\' => escaped = true,
            '"' => in_string = !in_string,
            _ if !in_string => {
                // Check if pattern starts at this position
                let remaining = &line[pos..];
                if remaining.starts_with(pattern) {
                    return false; // Pattern found outside string
                }
            }
            _ => {}
        }

        pos += c.len_utf8();
    }

    // If we get here, pattern is only in strings (or not found)
    line.contains(pattern)
}

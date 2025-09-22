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

/// Helper function to check if a pattern is inside a string literal or comment
pub fn is_in_string_literal(line: &str, pattern: &str) -> bool {
    // First check if pattern exists at all
    if !line.contains(pattern) {
        return false;
    }

    // Find all occurrences of the pattern
    let pattern_positions: Vec<usize> = line.match_indices(pattern).map(|(i, _)| i).collect();
    if pattern_positions.is_empty() {
        return false;
    }

    // Check each occurrence to see if it's in a string or comment
    for pattern_pos in pattern_positions {
        let mut in_string = false;
        let mut in_raw_string = false;
        let mut escaped = false;
        let mut pos = 0;

        // Check if we're in a comment
        if let Some(comment_pos) = line.find("//") {
            if pattern_pos >= comment_pos {
                continue; // This occurrence is in a comment, check next
            }
        }

        for c in line.chars() {
            if pos >= pattern_pos {
                // We've reached the pattern position
                // If we're not in a string, it's a real occurrence
                if !in_string && !in_raw_string {
                    return false;
                }
                break;
            }

            if escaped {
                escaped = false;
                pos += c.len_utf8();
                continue;
            }

            match c {
                '\\' if in_string && !in_raw_string => escaped = true,
                '"' if !in_raw_string => in_string = !in_string,
                'r' if !in_string && !in_raw_string => {
                    // Check for raw string
                    let remaining = &line[pos..];
                    if remaining.starts_with("r\"") || remaining.starts_with("r#\"") {
                        in_raw_string = true;
                    }
                }
                _ => {}
            }

            pos += c.len_utf8();
        }
    }

    // All occurrences are in strings or comments
    true
}

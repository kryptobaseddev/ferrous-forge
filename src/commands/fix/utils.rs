//! Utility functions for the fix command

use super::strategies::can_potentially_auto_fix;
use crate::validation::Violation;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Filter violations based on user options
pub fn filter_violations(
    violations: &[Violation],
    only_types: &Option<HashSet<String>>,
    skip_types: &Option<HashSet<String>>,
    limit: Option<usize>,
) -> Vec<Violation> {
    let mut filtered: Vec<Violation> = violations
        .iter()
        .filter(|v| {
            // Check if this type can be auto-fixed
            if !can_potentially_auto_fix(v) {
                return false;
            }

            let violation_type_str = format!("{:?}", v.violation_type).to_uppercase();

            // Apply only filter
            if let Some(only) = only_types {
                if !only.contains(&violation_type_str) {
                    return false;
                }
            }

            // Apply skip filter
            if let Some(skip) = skip_types {
                if skip.contains(&violation_type_str) {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect();

    // Apply limit if specified
    if let Some(limit) = limit {
        filtered.truncate(limit);
    }

    filtered
}

/// Group violations by file path
pub fn group_violations_by_file(violations: &[Violation]) -> HashMap<PathBuf, Vec<Violation>> {
    let mut grouped = HashMap::new();

    for violation in violations {
        grouped
            .entry(violation.file.clone())
            .or_insert_with(Vec::new)
            .push(violation.clone());
    }

    // Sort violations in each file by line number (descending)
    // so we fix from bottom to top and don't mess up line numbers
    for violations in grouped.values_mut() {
        violations.sort_by(|a, b| b.line.cmp(&a.line));
    }

    grouped
}

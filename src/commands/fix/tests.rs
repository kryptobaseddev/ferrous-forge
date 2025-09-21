#[cfg(test)]
mod tests {
    use super::super::{analyze_file_context, FixResult};
    use super::super::strategies;
    use crate::validation::{Severity, Violation, ViolationType};
    use std::path::PathBuf;

    #[test]
    fn test_safe_unwrap_fix() {
        let content = "use anyhow::Result;\n\nfn process() -> Result<()> {\n    \
                        let value = some_func().unwrap();\n}";
        let context = analyze_file_context(content);
        let line = "    let value = some_func().unwrap();";
        let violation = Violation {
            violation_type: ViolationType::UnwrapInProduction,
            file: PathBuf::from("test.rs"),
            line: 4,
            message: String::new(),
            severity: Severity::Error,
        };

        match strategies::fix_violation_in_line(line, &violation, &context) {
            FixResult::Fixed(fixed) => {
                assert_eq!(fixed, "    let value = some_func()?;");
            }
            _ => panic!("Expected fix to be applied"),
        }
    }

    #[test]
    fn test_skip_test_file_fixes() {
        let content = "#[cfg(test)]\nmod tests {\n    #[test]\n    \
                        fn test_something() {\n        value.unwrap();\n    }\n}";
        let context = analyze_file_context(content);
        let line = "        value.unwrap();";
        let violation = Violation {
            violation_type: ViolationType::UnwrapInProduction,
            file: PathBuf::from("test.rs"),
            line: 5,
            message: String::new(),
            severity: Severity::Error,
        };

        match strategies::fix_violation_in_line(line, &violation, &context) {
            FixResult::Skipped(reason) => {
                assert!(reason.contains("Test file"));
            }
            _ => panic!("Expected fix to be skipped for test file"),
        }
    }

    #[test]
    fn test_context_analysis() {
        let content = "#[cfg(test)]
use anyhow::Result;
use thiserror::Error;

fn main() -> Result<()> {
    Ok(())
}

#[test]
fn test_something() {
    assert!(true);
}

async fn process_data() -> Result<String> {
    Ok(\"data\".to_string())
}";
        let context = analyze_file_context(content);

        assert!(context.is_bin_file);
        assert!(context.is_test_file);
        // Debug: print function signatures found
        eprintln!("Found {} function signatures:", context.function_signatures.len());
        for sig in &context.function_signatures {
            eprintln!("  - {} at lines {}-{}", sig.name, sig.line_start, sig.line_end);
        }
        assert_eq!(context.function_signatures.len(), 3);
    }
}
#[cfg(test)]
use super::context::analyze_file_context;
use super::strategies;
use super::types::FixResult;
use crate::validation::{Severity, Violation, ViolationType};
use std::path::PathBuf;

#[test]
fn test_safe_unwrap_fix() {
    let violation = Violation {
        violation_type: ViolationType::UnwrapInProduction,
        file: PathBuf::from("test.rs"),
        line: 5,
        message: "unwrap used".to_string(),
        severity: Severity::Error,
    };

    // Test with Result return type
    let context_code = "fn test() -> Result<()> {\n    let x = Some(5).unwrap();\n}";
    let context = analyze_file_context(context_code);

    let fix_result = strategies::attempt_safe_unwrap_fix(&violation, context_code, &context);

    match fix_result {
        FixResult::Fixed(new_code) => {
            assert!(new_code.contains("Some(5)?"));
        }
        _ => assert!(false, "Expected fix to be applied"),
    }
}

#[test]
fn test_safe_unwrap_fix_skip_test() {
    let violation = Violation {
        violation_type: ViolationType::UnwrapInProduction,
        file: PathBuf::from("test.rs"),
        line: 2,
        message: "unwrap used".to_string(),
        severity: Severity::Error,
    };

    // Test in test context
    let context_code = "#[test]\nfn test_something() {\n    let x = Some(5).unwrap();\n}";
    let context = analyze_file_context(context_code);

    let fix_result = strategies::attempt_safe_unwrap_fix(&violation, context_code, &context);

    match fix_result {
        FixResult::Skipped(_) => {
            // Expected - test code should be skipped
        }
        _ => assert!(false, "Expected fix to be skipped for test file"),
    }
}

#[test]
fn test_context_analysis() {
    let code = r#"
use std::io;

fn function_one() {
    println!("One");
}

fn function_two() -> Result<(), io::Error> {
    Ok(())
}

fn function_three() -> Option<i32> {
    Some(42)
}
"#;

    let context = analyze_file_context(code);

    // Check function signatures
    for sig in &context.function_signatures {
        println!(
            "Function: {} at lines {}-{}",
            sig.name, sig.line_start, sig.line_end
        );
    }
    assert_eq!(context.function_signatures.len(), 3);
}

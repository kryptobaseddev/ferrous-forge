//! Additional validation checks (documentation, formatting, security)

use crate::{Result, doc_coverage, formatting, security, validation::RustValidator};
use std::path::Path;

/// Run all additional checks (documentation, formatting, security)
pub async fn run_additional_checks(project_path: &Path) {
    check_documentation_coverage(project_path).await;
    check_code_formatting(project_path).await;
    run_security_audit_check(project_path).await;
}

/// Check documentation coverage
async fn check_documentation_coverage(project_path: &Path) {
    println!("📚 Checking documentation coverage...");

    // Add small delay to ensure proper output ordering
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    match doc_coverage::check_documentation_coverage(project_path).await {
        Ok(coverage) => {
            println!("{}", coverage.report());
            if coverage.coverage_percent < 80.0 {
                println!("⚠️  Documentation coverage below 80%");
            }
        }
        Err(e) => {
            println!("❌ Documentation coverage check failed: {}", e);
        }
    }
    println!();
}

/// Check code formatting
async fn check_code_formatting(project_path: &Path) {
    println!("📝 Checking code formatting...");

    match formatting::check_formatting(project_path).await {
        Ok(format_result) => {
            println!("{}", format_result.report());
            if !format_result.unformatted_files.is_empty() {
                println!("Files needing formatting:");
                for file in &format_result.unformatted_files {
                    println!("  {}", file);
                }
            }
        }
        Err(e) => {
            println!("❌ Code formatting check failed: {}", e);
        }
    }
    println!();
}

/// Run security audit check
async fn run_security_audit_check(project_path: &Path) {
    println!("🔒 Running security audit...");

    match security::run_security_audit(project_path).await {
        Ok(audit_report) => {
            println!("{}", audit_report.report());
        }
        Err(e) => {
            println!("❌ Security audit failed: {}", e);
        }
    }
}

/// Run clippy validation
pub async fn run_clippy_validation(
    validator: &RustValidator,
) -> Result<crate::validation::ClippyResult> {
    println!("🔧 Running Clippy with strict configuration...");

    let clippy_result = validator.run_clippy().await?;

    if !clippy_result.success {
        println!("❌ Clippy found issues:");
        println!("{}", clippy_result.output);
    } else {
        println!("✅ Clippy validation passed!");
    }

    Ok(clippy_result)
}

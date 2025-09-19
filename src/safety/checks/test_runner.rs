//! Test runner for safety checks

use crate::Result;
use std::path::Path;

use super::{build, clippy, format, standards};

/// Run a quick test of the safety checks
pub async fn test_safety_checks(project_path: &Path) -> Result<()> {
    println!("🧪 Testing Safety Pipeline Checks");
    println!("==================================");

    // Test format check
    println!("\n📝 Testing Format Check...");
    match format::run(project_path).await {
        Ok(result) => {
            println!(
                "  {} Format Check ({:.2}s)",
                result.status_emoji(),
                result.duration.as_secs_f64()
            );
            if !result.passed {
                for error in &result.errors {
                    println!("    ⚠️  {}", error);
                }
            }
        }
        Err(e) => println!("  ❌ Format check error: {}", e),
    }

    // Test clippy check
    println!("\n🔍 Testing Clippy Check...");
    match clippy::run(project_path).await {
        Ok(result) => {
            println!(
                "  {} Clippy Check ({:.2}s)",
                result.status_emoji(),
                result.duration.as_secs_f64()
            );
            if !result.passed {
                for error in result.errors.iter().take(3) {
                    println!("    ⚠️  {}", error);
                }
            }
        }
        Err(e) => println!("  ❌ Clippy check error: {}", e),
    }

    // Test build check
    println!("\n🏗️  Testing Build Check...");
    match build::run(project_path).await {
        Ok(result) => {
            println!(
                "  {} Build Check ({:.2}s)",
                result.status_emoji(),
                result.duration.as_secs_f64()
            );
            if !result.passed {
                for error in result.errors.iter().take(2) {
                    println!("    ⚠️  {}", error);
                }
            }
        }
        Err(e) => println!("  ❌ Build check error: {}", e),
    }

    // Test standards check
    println!("\n📋 Testing Standards Check...");
    match standards::run(project_path).await {
        Ok(result) => {
            println!(
                "  {} Standards Check ({:.2}s)",
                result.status_emoji(),
                result.duration.as_secs_f64()
            );
            if !result.passed {
                for error in result.errors.iter().take(3) {
                    println!("    ⚠️  {}", error);
                }
            }
        }
        Err(e) => println!("  ❌ Standards check error: {}", e),
    }

    println!("\n🎉 Safety pipeline test complete!");
    Ok(())
}

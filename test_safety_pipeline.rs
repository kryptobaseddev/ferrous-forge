#!/usr/bin/env rust-script
//! Test script to verify the safety pipeline works

use ferrous_forge::safety::{SafetyPipeline, PipelineStage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Ferrous Forge Safety Pipeline");
    
    // Test the safety pipeline on current project
    let pipeline = SafetyPipeline::new(".").await?;
    
    println!("\n🛡️  Testing Pre-Commit Stage...");
    let report = pipeline.run_checks(PipelineStage::PreCommit).await?;
    report.print_summary();
    
    println!("\n📊 Report Details:");
    println!("  Checks run: {}", report.checks.len());
    println!("  Passed: {}", report.passed);
    println!("  Duration: {:.2}s", report.total_duration.as_secs_f64());
    
    if !report.passed {
        println!("\n❌ Failed checks:");
        for failed in report.failed_checks() {
            println!("  • {}: {} errors", failed.check_type.display_name(), failed.errors.len());
        }
    }
    
    Ok(())
}

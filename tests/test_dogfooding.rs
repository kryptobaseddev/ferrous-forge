#!/usr/bin/env rust-script
//! Test using ferrous-forge crate on itself

use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍖 Testing Dogfooding - Using Ferrous Forge on Itself");
    println!("====================================================");

    // Test if we can use the external ferrous-forge crate
    println!("\n📦 Testing external ferrous-forge crate...");

    // Run ferrous-forge validate using the published crate
    let output = Command::new("ferrous-forge")
        .args(&["validate", "."])
        .output()?;

    if output.status.success() {
        println!("✅ External ferrous-forge crate works!");
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("Output: {}", stdout);
        }
    } else {
        println!("❌ External ferrous-forge failed");
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            println!("Error: {}", stderr);
        }
    }

    // Test rust version check
    println!("\n🦀 Testing rust version check...");
    let output = Command::new("ferrous-forge")
        .args(&["rust", "check"])
        .output()?;

    if output.status.success() {
        println!("✅ Rust version check works!");
    } else {
        println!("❌ Rust version check failed");
    }

    // Test edition check
    println!("\n📚 Testing edition check...");
    let output = Command::new("ferrous-forge")
        .args(&["edition", "check"])
        .output()?;

    if output.status.success() {
        println!("✅ Edition check works!");
    } else {
        println!("❌ Edition check failed");
    }

    println!("\n🎉 Dogfooding test complete!");
    Ok(())
}

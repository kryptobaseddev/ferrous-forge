#![allow(clippy::expect_used, clippy::unwrap_used)] // expect()/unwrap() are fine in benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ferrous_forge::validation::RustValidator;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn benchmark_validate_cargo_toml(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    std::fs::write(
        &cargo_toml,
        r#"
[package]
name = "test"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = "1.0"
"#,
    )
    .expect("Failed to write Cargo.toml");

    c.bench_function("validate_cargo_toml", |b| {
        b.iter(|| {
            rt.block_on(async {
                let validator = RustValidator::new(temp_dir.path().to_path_buf())
                    .expect("Failed to create validator");
                let mut violations = Vec::new();
                validator
                    .validate_cargo_toml(black_box(&cargo_toml), &mut violations)
                    .await
                    .expect("Failed to validate");
                violations
            })
        })
    });
}

fn benchmark_validate_rust_file(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let rust_file = temp_dir.path().join("test.rs");

    std::fs::write(
        &rust_file,
        r#"
use std::collections::HashMap;

pub fn example_function() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example() {
        let result = example_function();
        assert!(!result.is_empty());
    }
}
"#,
    )
    .expect("Failed to write Rust file");

    c.bench_function("validate_rust_file", |b| {
        b.iter(|| {
            rt.block_on(async {
                let validator = RustValidator::new(temp_dir.path().to_path_buf())
                    .expect("Failed to create validator");
                let mut violations = Vec::new();
                validator
                    .validate_rust_file(black_box(&rust_file), &mut violations)
                    .await
                    .expect("Failed to validate");
                violations
            })
        })
    });
}

fn benchmark_validate_project(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create minimal project structure
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir(&src_dir).expect("Failed to create src dir");

    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#,
    )
    .expect("Failed to write Cargo.toml");

    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
//! Test library

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .expect("Failed to write lib.rs");

    c.bench_function("validate_project", |b| {
        b.iter(|| {
            rt.block_on(async {
                let validator = RustValidator::new(black_box(temp_dir.path().to_path_buf()))
                    .expect("Failed to create validator");
                validator
                    .validate_project()
                    .await
                    .expect("Failed to validate")
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_validate_cargo_toml,
    benchmark_validate_rust_file,
    benchmark_validate_project
);
criterion_main!(benches);

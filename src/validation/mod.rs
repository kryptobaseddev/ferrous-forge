//! Rust code validation engine

/// Rust source code and Cargo.toml validator.
pub mod rust_validator;
/// Violation types, severity levels, and diagnostics.
pub mod violation;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

pub use rust_validator::{ClippyResult, RustValidator};
pub use violation::{Severity, Violation, ViolationType};

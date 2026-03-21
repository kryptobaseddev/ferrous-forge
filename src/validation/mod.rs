//! Rust code validation engine

/// Rust source code and Cargo.toml validator.
pub mod rust_validator;
/// Version consistency validation (SSoT enforcement)
pub mod version_consistency;
/// Violation types, severity levels, and diagnostics.
pub mod violation;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

pub use rust_validator::{ClippyResult, RustValidator};
pub use version_consistency::{VersionConsistencyValidator, VersionValidationResult};
pub use violation::{Severity, Violation, ViolationType};

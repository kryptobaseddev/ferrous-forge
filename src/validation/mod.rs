//! Rust code validation engine

pub mod rust_validator;
pub mod violation;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

pub use rust_validator::{ClippyResult, RustValidator};
pub use violation::{Severity, Violation, ViolationType};

//! Rust code validation engine
#![allow(clippy::unwrap_used)]

pub mod rust_validator;
pub mod violation;

#[cfg(test)]
mod tests;

pub use rust_validator::{ClippyResult, RustValidator};
pub use violation::{Severity, Violation, ViolationType};

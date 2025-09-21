//! Rust code validation engine
#![allow(clippy::unwrap_used)]

pub mod violation;
pub mod rust_validator;

pub use violation::{Severity, Violation, ViolationType};
pub use rust_validator::{ClippyResult, RustValidator};
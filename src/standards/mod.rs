//! Rust coding standards definitions and enforcement
#![allow(clippy::unwrap_used)]
//!
//! This module defines the specific standards that Ferrous Forge enforces
//! and provides utilities for checking compliance.

mod defaults;
mod implementation;
#[cfg(test)]
mod tests;
mod types;

pub use types::*;

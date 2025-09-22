//! # Ferrous Forge
//!
//! The Type-Safe Rust Development Standards Enforcer
//!
//! Ferrous Forge is a comprehensive system-wide tool that automatically enforces
//! professional Rust development standards across all projects on your machine.
//!
//! ## Features
//!
//! - Zero underscore bandaid coding enforcement
//! - Edition 2024 automatic upgrades
//! - System-wide cargo command hijacking
//! - Automatic project template injection
//! - Real-time code validation
//! - Professional CI/CD setup
//!
//! ## Quick Start
//!
//! ```bash
//! cargo install ferrous-forge
//! ferrous-forge init
//! cargo new my-project  # Now follows all standards automatically
//! ```
//!
//! ## Modules
//!
//! - [`cli`] - Command line interface definitions
//! - [`commands`] - Command implementations
//! - [`config`] - Configuration management
//! - [`standards`] - Standards definitions and enforcement
//! - [`templates`] - Project template system
//! - [`validation`] - Code validation and linting
//! - [`updater`] - Self-update and version management

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// AI-powered violation analysis module
pub mod ai_analyzer;
pub mod cli;
pub mod commands;
pub mod config;
pub mod doc_coverage;
pub mod edition;
pub mod error;
pub mod formatting;
pub mod git_hooks;
pub mod rust_version;
pub mod safety;
pub mod security;
pub mod standards;
pub mod templates;
pub mod test_coverage;
pub mod updater;
pub mod validation;

// Re-export commonly used types
pub use crate::config::Config;
pub use crate::error::{Error, Result};

/// Current version of Ferrous Forge
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum supported Rust version
pub const MIN_RUST_VERSION: &str = "1.82.0";

/// Edition enforced by Ferrous Forge
pub const REQUIRED_EDITION: &str = "2024";

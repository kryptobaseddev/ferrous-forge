//! # Ferrous Forge
//!
//! The Type-Safe Rust Development Standards Enforcer
//!
//! Ferrous Forge is a comprehensive system-wide tool that automatically enforces
//! professional Rust development standards across all projects on your machine.
//!
//! @task T016
//! @epic T014
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
//! ```rust,no_run
//! use ferrous_forge::{Config, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Load or create default configuration
//!     let config = Config::load_or_default().await?;
//!     println!("Ferrous Forge v{}", ferrous_forge::VERSION);
//!     println!("Required edition: {}", config.required_edition);
//!     Ok(())
//! }
//! ```
//!
//! ### CLI Usage
//!
//! ```bash
//! cargo install ferrous-forge
//! ferrous-forge init
//! cargo new my-project  # Now follows all standards automatically
//! ```
//!
//! ## Modules
//!
//! ### Core
//! - [`cli`] — Command line interface definitions and argument parsing
//! - [`commands`] — Implementation of all Ferrous Forge commands
//! - [`config`] — Configuration management and hierarchical config system
//! - [`error`] — Error types and result handling
//!
//! ### Standards & Validation
//! - [`standards`] — Development standards definitions and enforcement
//! - [`validation`] — Core validation logic and rule enforcement
//! - [`safety`] — Safety pipeline and enforcement mechanisms
//! - [`formatting`] — Code formatting enforcement and validation
//!
//! ### Rust Ecosystem
//! - [`edition`] — Rust edition management and upgrade assistance
//! - [`rust_version`] — Rust version checking and compatibility validation
//! - [`doc_coverage`] — Documentation coverage checking and reporting
//! - [`security`] — Security auditing and vulnerability scanning
//! - [`test_coverage`] — Test coverage integration and reporting
//!
//! ### Tooling
//! - [`templates`] — Project template system and built-in templates
//! - [`git_hooks`] — Git hooks installation and management
//! - [`cargo_intercept`] — Cargo command interception for publish validation
//! - [`updater`] — Self-update functionality and version management
//!
//! ### Analysis
//! - [`ai_analyzer`] — AI-powered violation analysis and fix suggestions
//! - [`performance`] — Performance optimizations for validation

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// AI-powered violation analysis and fix suggestions
pub mod ai_analyzer;
/// Cargo command interception for publish validation
pub mod cargo_intercept;
/// Command line interface definitions and argument parsing
pub mod cli;
/// Implementation of all Ferrous Forge commands
pub mod commands;
/// Configuration management and hierarchical config system
pub mod config;
/// Documentation coverage checking and reporting
pub mod doc_coverage;
/// Rust edition management and upgrade assistance
pub mod edition;
/// Error types and result handling
pub mod error;
/// Code formatting enforcement and validation
pub mod formatting;
/// Git hooks installation and management
pub mod git_hooks;
/// Performance optimizations for validation
pub mod performance;
/// Rust version checking and compatibility validation
pub mod rust_version;
/// Safety pipeline and enforcement mechanisms
pub mod safety;
/// Security auditing and vulnerability scanning
pub mod security;
/// Development standards definitions and enforcement
pub mod standards;
/// Project template system and built-in templates
pub mod templates;
/// Test coverage integration and reporting
pub mod test_coverage;
/// Self-update functionality and version management
pub mod updater;
/// Core validation logic and rule enforcement
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

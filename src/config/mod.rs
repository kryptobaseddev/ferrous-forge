//! Configuration management for Ferrous Forge
//!
//! Provides a hierarchical configuration system with three levels:
//! - System: /etc/ferrous-forge/config.toml
//! - User: ~/.config/ferrous-forge/config.toml
//! - Project: ./.ferrous-forge/config.toml

/// Hierarchical configuration system with system, user, and project levels.
pub mod hierarchy;
/// Configuration file reading and writing.
pub mod io;
/// Configuration manipulation operations (get, set, reset).
pub mod operations;
/// Configuration data types and defaults.
pub mod types;

pub use hierarchy::{ConfigLevel, HierarchicalConfig};
pub use types::{Config, CustomRule};

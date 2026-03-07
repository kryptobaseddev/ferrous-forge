//! Configuration management for Ferrous Forge
//!
//! Provides a hierarchical configuration system with three levels:
//! - System: /etc/ferrous-forge/config.toml
//! - User: ~/.config/ferrous-forge/config.toml
//! - Project: ./.ferrous-forge/config.toml

pub mod hierarchy;
pub mod io;
pub mod operations;
pub mod types;

pub use hierarchy::{ConfigLevel, HierarchicalConfig};
pub use types::{Config, CustomRule};

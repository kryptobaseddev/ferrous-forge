//! Configuration management for Ferrous Forge
//!
//! Provides a hierarchical configuration system with three levels:
//! - System: /etc/ferrous-forge/config.toml
//! - User: ~/.config/ferrous-forge/config.toml
//! - Project: ./.ferrous-forge/config.toml
//!
//! @task T018
//! @epic T014

/// Hierarchical configuration system with system, user, and project levels.
pub mod hierarchy;
/// Configuration file reading and writing.
pub mod io;
/// Configuration locking mechanism for critical values.
pub mod locking;
/// Configuration manipulation operations (get, set, reset).
pub mod operations;
/// Configuration sharing utilities for team-wide config export/import.
pub mod sharing;
/// Configuration data types and defaults.
pub mod types;

pub use hierarchy::{ConfigLevel, HierarchicalConfig};
pub use locking::{ConfigValidator, HierarchicalLockManager, LockEntry, LockedConfig, audit_log};
pub use sharing::{ImportOptions, ImportReport, SharedConfig, import_shared_config};
pub use types::{Config, CustomRule};

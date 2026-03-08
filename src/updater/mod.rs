//! Self-update system for Ferrous Forge

/// Automatic update checking on command invocation.
pub mod auto;
/// GitHub release fetching for update discovery.
pub mod github;
/// Update orchestration and binary replacement.
pub mod manager;
/// Update data types for channels and release info.
pub mod types;

pub use auto::check_auto_update;
pub use types::{UpdateChannel, UpdateInfo, UpdateManager};

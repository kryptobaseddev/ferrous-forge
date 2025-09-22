//! Self-update system for Ferrous Forge

pub mod auto;
pub mod github;
pub mod manager;
pub mod types;

pub use auto::check_auto_update;
pub use types::{UpdateChannel, UpdateInfo, UpdateManager};

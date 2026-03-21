//! Git hooks installation and management
//!
//! This module provides functionality to install and manage git hooks
//! for automatic validation on commits.
//!
//! @task T017
//! @epic T014

mod installer;
mod scripts;

pub use installer::{check_hooks_status, install_git_hooks, uninstall_git_hooks};

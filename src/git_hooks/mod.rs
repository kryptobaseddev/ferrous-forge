//! Git hooks installation and management
//!
//! This module provides functionality to install and manage git hooks
//! for automatic validation on commits.

mod installer;
mod scripts;

pub use installer::{install_git_hooks, uninstall_git_hooks};

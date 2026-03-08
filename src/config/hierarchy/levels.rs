//! Configuration level definitions

use crate::{Error, Result};
use std::path::PathBuf;

/// Configuration level in the hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigLevel {
    /// System-wide configuration
    System,
    /// User-specific configuration
    User,
    /// Project-specific configuration
    Project,
}

impl ConfigLevel {
    /// Get the configuration file path for this level
    ///
    /// # Errors
    ///
    /// Returns an error if the user config directory cannot be determined.
    pub fn path(&self) -> Result<PathBuf> {
        match self {
            ConfigLevel::System => Ok(PathBuf::from("/etc/ferrous-forge/config.toml")),
            ConfigLevel::User => {
                let config_dir = dirs::config_dir()
                    .ok_or_else(|| Error::config("Could not find config directory"))?;
                Ok(config_dir.join("ferrous-forge").join("config.toml"))
            }
            ConfigLevel::Project => Ok(PathBuf::from(".ferrous-forge/config.toml")),
        }
    }

    /// Display name for this level
    pub fn display_name(&self) -> &'static str {
        match self {
            ConfigLevel::System => "System",
            ConfigLevel::User => "User",
            ConfigLevel::Project => "Project",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_level_ordering() {
        assert!(ConfigLevel::System < ConfigLevel::User);
        assert!(ConfigLevel::User < ConfigLevel::Project);
    }
}

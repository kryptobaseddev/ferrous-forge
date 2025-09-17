//! Error handling for Ferrous Forge
//!
//! This module provides a unified error handling system for all Ferrous Forge operations.


/// Ferrous Forge specific errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO errors (file operations, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Template errors
    #[error("Template error: {0}")]
    Template(String),

    /// Update system errors
    #[error("Update error: {0}")]
    Update(String),

    /// Standards enforcement errors
    #[error("Standards violation: {0}")]
    Standards(String),

    /// CLI argument errors
    #[error("CLI error: {0}")]
    Cli(String),

    /// External process errors
    #[error("Process error: {0}")]
    Process(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML parsing errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Network/HTTP errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Semver parsing errors
    #[error("Version error: {0}")]
    Version(#[from] semver::Error),
}

/// Result type alias for Ferrous Forge operations
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a new template error
    pub fn template(msg: impl Into<String>) -> Self {
        Self::Template(msg.into())
    }

    /// Create a new update error
    pub fn update(msg: impl Into<String>) -> Self {
        Self::Update(msg.into())
    }

    /// Create a new standards error
    pub fn standards(msg: impl Into<String>) -> Self {
        Self::Standards(msg.into())
    }

    /// Create a new CLI error
    pub fn cli(msg: impl Into<String>) -> Self {
        Self::Cli(msg.into())
    }

    /// Create a new process error
    pub fn process(msg: impl Into<String>) -> Self {
        Self::Process(msg.into())
    }
}

/// Convert anyhow errors to our error type
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Process(err.to_string())
    }
}
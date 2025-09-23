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
    Network(String),

    /// Semver parsing errors
    #[error("Version error: {0}")]
    Version(#[from] semver::Error),

    /// Parse errors
    #[error("Parse error: {0}")]
    Parse(String),

    /// Rust not found
    #[error("Rust not found: {0}")]
    RustNotFound(String),

    /// Command execution error
    #[error("Command error: {0}")]
    Command(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Rate limited
    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u64),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Regex error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// UTF-8 conversion error
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// Safety pipeline error
    #[error("Safety error: {0}")]
    Safety(String),

    /// Safety pipeline blocked operation
    #[error("Safety blocked: {0}")]
    SafetyBlocked(String),
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

    /// Create a new parse error
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Create a new rust not found error
    pub fn rust_not_found(msg: impl Into<String>) -> Self {
        Self::RustNotFound(msg.into())
    }

    /// Create a new command error
    pub fn command(msg: impl Into<String>) -> Self {
        Self::Command(msg.into())
    }

    /// Create a new file not found error
    pub fn file_not_found(msg: impl Into<String>) -> Self {
        Self::FileNotFound(msg.into())
    }

    /// Create a new rate limited error
    pub fn rate_limited(retry_after: u64) -> Self {
        Self::RateLimited(retry_after)
    }

    /// Create a new IO error from a string
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(std::io::Error::other(msg.into()))
    }

    /// Create a new security error
    pub fn security(msg: impl Into<String>) -> Self {
        Self::Validation(format!("Security: {}", msg.into()))
    }

    /// Create a new migration error
    pub fn migration(msg: impl Into<String>) -> Self {
        Self::Migration(msg.into())
    }

    /// Create a new network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a new safety error
    pub fn safety(msg: impl Into<String>) -> Self {
        Self::Safety(msg.into())
    }

    /// Create a new safety blocked error
    pub fn safety_blocked(msg: impl Into<String>) -> Self {
        Self::SafetyBlocked(msg.into())
    }

    /// Create a new tool not found error
    pub fn tool_not_found(tool: impl Into<String>) -> Self {
        Self::Process(format!("Tool not found: {}", tool.into()))
    }
}

/// Convert anyhow errors to our error type
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Process(err.to_string())
    }
}

/// Convert reqwest errors to our error type
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

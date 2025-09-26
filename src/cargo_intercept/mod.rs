//! Cargo publish interception system
//!
//! This module provides functionality to intercept `cargo publish` commands
//! and run Ferrous Forge validation before allowing publication to crates.io.

pub mod validation;
pub mod wrapper;

use crate::Result;
use std::env;
use std::path::Path;

/// Intercepts and validates cargo publish commands
pub struct CargoInterceptor {
    /// Whether to enforce dogfooding (using Ferrous Forge on itself)
    enforce_dogfooding: bool,
    /// Bypass mode for emergencies
    bypass_enabled: bool,
}

impl Default for CargoInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl CargoInterceptor {
    /// Create a new cargo interceptor
    pub fn new() -> Self {
        let bypass_enabled = env::var("FERROUS_FORGE_BYPASS")
            .unwrap_or_default()
            .eq_ignore_ascii_case("true");

        Self {
            enforce_dogfooding: true,
            bypass_enabled,
        }
    }

    /// Create interceptor with custom settings
    pub fn with_dogfooding(enforce_dogfooding: bool) -> Self {
        let bypass_enabled = env::var("FERROUS_FORGE_BYPASS")
            .unwrap_or_default()
            .eq_ignore_ascii_case("true");

        Self {
            enforce_dogfooding,
            bypass_enabled,
        }
    }
}

/// Intercept cargo publish command and run validation
pub async fn intercept_publish_command(project_path: &Path) -> Result<()> {
    let interceptor = CargoInterceptor::new();

    if interceptor.bypass_enabled {
        tracing::warn!("FERROUS_FORGE_BYPASS enabled - skipping validation");
        return Ok(());
    }

    tracing::info!("Intercepting cargo publish - running validation");

    // Run pre-publish validation
    validation::pre_publish_validation(project_path).await?;

    // Check version consistency
    validation::version_consistency_check(project_path)?;

    // Enforce dogfooding if enabled
    if interceptor.enforce_dogfooding {
        validation::enforce_dogfooding(project_path).await?;
    }

    tracing::info!("Pre-publish validation passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_interceptor_creation() {
        let interceptor = CargoInterceptor::new();
        assert!(interceptor.enforce_dogfooding);
    }
}

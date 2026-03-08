//! Cargo publish interception system
//!
//! This module provides functionality to intercept `cargo publish` commands
//! and run Ferrous Forge validation before allowing publication to crates.io.
//!
//! ## Tiered Blocking Behavior
//!
//! - **Edition/version violations** → block ALL cargo commands (build, test, run, check, publish)
//! - **Code style violations** (file size, function size) → WARN during dev commands, block at publish
//! - `FERROUS_FORGE_BYPASS=true` → skips style checks; edition/version still
//!   enforced
//! - `FERROUS_FORGE_FORCE_BYPASS=true` → absolute override with visible "BYPASSED" warning

/// Validation logic for intercepted cargo commands.
pub mod validation;
/// Cargo command wrapper for transparent interception.
pub mod wrapper;

use crate::Result;
use crate::validation::ViolationType;
use std::env;
use std::path::Path;

/// Intercepts and validates cargo publish commands
pub struct CargoInterceptor {
    /// Whether to enforce dogfooding (using Ferrous Forge on itself)
    enforce_dogfooding: bool,
    /// Skip style checks (edition/version still enforced)
    bypass_style: bool,
    /// Absolute override — skip all checks (with warning)
    force_bypass: bool,
}

impl Default for CargoInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl CargoInterceptor {
    /// Create a new cargo interceptor, reading bypass state from environment
    pub fn new() -> Self {
        let bypass_style = env::var("FERROUS_FORGE_BYPASS")
            .unwrap_or_default()
            .eq_ignore_ascii_case("true");

        let force_bypass = env::var("FERROUS_FORGE_FORCE_BYPASS")
            .unwrap_or_default()
            .eq_ignore_ascii_case("true");

        Self {
            enforce_dogfooding: true,
            bypass_style,
            force_bypass,
        }
    }

    /// Create interceptor with custom settings
    pub fn with_dogfooding(enforce_dogfooding: bool) -> Self {
        let mut interceptor = Self::new();
        interceptor.enforce_dogfooding = enforce_dogfooding;
        interceptor
    }
}

/// Intercept cargo publish command and run full validation (all violations block)
///
/// # Errors
///
/// Returns [`crate::Error::Validation`] if pre-publish validation or version consistency
/// checks fail. Returns [`crate::Error::Standards`] if dogfooding enforcement detects
/// violations.
pub async fn intercept_publish_command(project_path: &Path) -> Result<()> {
    let interceptor = CargoInterceptor::new();

    if interceptor.force_bypass {
        eprintln!(
            "\n⚠️  FERROUS FORGE FORCE BYPASSED — FERROUS_FORGE_FORCE_BYPASS=true\n\
             All validation skipped. This should NEVER happen in production.\n"
        );
        return Ok(());
    }

    if interceptor.bypass_style {
        tracing::warn!(
            "FERROUS_FORGE_BYPASS enabled — style checks skipped, locked settings still enforced"
        );
    }

    tracing::info!("Intercepting cargo publish — running validation");

    // For publish: ALL violations block (both locked and style)
    validation::pre_publish_validation(project_path).await?;
    validation::version_consistency_check(project_path)?;

    if interceptor.enforce_dogfooding {
        validation::enforce_dogfooding(project_path).await?;
    }

    tracing::info!("Pre-publish validation passed");
    Ok(())
}

/// Intercept dev commands (build, test, run, check) with tiered blocking:
/// - Locked settings (edition/version) → ALWAYS block
/// - Style violations → WARN only (unless `enforce_style` is true)
///
/// # Errors
///
/// Returns [`crate::Error::Validation`] if locked settings (edition, rust-version) are
/// violated. Style violations produce warnings but do not return errors.
pub async fn intercept_dev_command(project_path: &Path) -> Result<()> {
    let interceptor = CargoInterceptor::new();

    if interceptor.force_bypass {
        eprintln!(
            "\n⚠️  FERROUS FORGE FORCE BYPASSED — FERROUS_FORGE_FORCE_BYPASS=true\n\
             All validation skipped. This should NEVER happen in production.\n"
        );
        return Ok(());
    }

    // Check locked settings violations first — these always block
    let locked_violations = validation::check_locked_settings(project_path).await?;
    if !locked_violations.is_empty() {
        eprintln!("\n❌ FERROUS FORGE — Locked Setting Violations Detected\n");
        for v in &locked_violations {
            eprintln!("{}\n", v.message);
        }
        return Err(crate::Error::validation(
            "Locked setting violations must be resolved before building. \
             See messages above. DO NOT change locked values — escalate to human.",
        ));
    }

    // Style violations — warn but don't block during dev (unless bypass disabled)
    if !interceptor.bypass_style {
        let style_violations = validation::check_style_violations(project_path).await?;
        if !style_violations.is_empty() {
            eprintln!(
                "\n⚠️  Ferrous Forge style warnings ({} violations):",
                style_violations.len()
            );
            for v in style_violations.iter().take(5) {
                eprintln!(
                    "   {:?}: {}:{} — {}",
                    v.violation_type,
                    v.file.display(),
                    v.line,
                    v.message.lines().next().unwrap_or("")
                );
            }
            if style_violations.len() > 5 {
                eprintln!(
                    "   ... and {} more (run 'ferrous-forge validate' \
                 for full list)",
                    style_violations.len() - 5
                );
            }
            eprintln!("   (These will block 'cargo publish'. Fix before publishing.)");
            eprintln!("   (Set FERROUS_FORGE_BYPASS=true to suppress these warnings.)\n");
        }
    } else {
        tracing::info!(
            "FERROUS_FORGE_BYPASS — style warnings suppressed (locked settings still checked)"
        );
    }

    Ok(())
}

/// Check if violations include any locked setting violations
pub fn has_locked_violations(violations: &[crate::validation::Violation]) -> bool {
    violations.iter().any(|v| {
        matches!(
            v.violation_type,
            ViolationType::WrongEdition
                | ViolationType::OldRustVersion
                | ViolationType::LockedSetting
        )
    })
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

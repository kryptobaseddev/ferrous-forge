//! Documentation presence validation
//!
//! Validates documentation standards that clippy cannot catch when lints are not yet configured:
//! - lib.rs and mod.rs files must have //! module-level documentation
//! - Cargo.toml should have [lints.rustdoc] section configured
//!
//! These checks prompt users toward running `ferrous-forge init --project` to properly
//! configure rustdoc lints, after which clippy takes ownership of enforcement.

use crate::Result;
use crate::validation::{Severity, Violation, ViolationType};
use std::path::Path;

/// Check that module root files (lib.rs, mod.rs) have //! documentation
pub fn validate_doc_presence(
    rust_file: &Path,
    lines: &[&str],
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let filename = rust_file.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Only check module root files
    if filename != "lib.rs" && filename != "mod.rs" {
        return Ok(());
    }

    let has_module_doc = lines
        .iter()
        .any(|line| line.trim_start().starts_with("//!"));

    if !has_module_doc {
        violations.push(Violation {
            violation_type: ViolationType::MissingModuleDoc,
            file: rust_file.to_path_buf(),
            line: 1,
            message: format!(
                "FERROUS FORGE [DOC STANDARD] — Missing Module Documentation\n  \
                 File: {}\n  \
                 Module root missing //! documentation. Add at minimum:\n  \
                   //! Brief description of what this module does.",
                rust_file.display()
            ),
            severity: Severity::Warning,
        });
    }

    Ok(())
}

/// Check that Cargo.toml has a [lints.rustdoc] section configured
pub fn validate_cargo_doc_config(
    cargo_file: &Path,
    content: &str,
    violations: &mut Vec<Violation>,
) -> Result<()> {
    if !content.contains("[lints.rustdoc]") && !content.contains("[lints]") {
        violations.push(Violation {
            violation_type: ViolationType::MissingDocConfig,
            file: cargo_file.to_path_buf(),
            line: 0,
            message: "FERROUS FORGE [DOC STANDARD] — Missing rustdoc lint configuration\n  \
                     Cargo.toml is missing [lints.rustdoc] section.\n  \
                     Run 'ferrous-forge init --project' to inject the full rustdoc lint block."
                .to_string(),
            severity: Severity::Warning,
        });
    }

    Ok(())
}

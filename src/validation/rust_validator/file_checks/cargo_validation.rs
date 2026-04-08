//! Cargo.toml validation functions

use crate::Result;
use crate::validation::{Severity, Violation, ViolationType};
use std::path::Path;
use tokio::fs;
use toml::Value;

/// Validates a `Cargo.toml` file for standards compliance (reads file itself).
/// Used by tests and legacy callers.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub async fn validate_cargo_toml(
    cargo_file: &Path,
    violations: &mut Vec<Violation>,
    required_edition: &str,
    required_rust_version: &str,
) -> Result<()> {
    let content = fs::read_to_string(cargo_file).await?;
    let lines: Vec<&str> = content.lines().collect();
    validate_cargo_toml_content(
        cargo_file,
        &lines,
        violations,
        required_edition,
        required_rust_version,
    );
    Ok(())
}

/// Validates `Cargo.toml` content that has already been read into lines.
/// Used by `file_checks.rs` to avoid double file reads.
pub fn validate_cargo_toml_content(
    cargo_file: &Path,
    lines: &[&str],
    violations: &mut Vec<Violation>,
    required_edition: &str,
    required_rust_version: &str,
) {
    // Parse the manifest with the TOML parser so we correctly understand
    // workspace inheritance (`version.workspace = true`, `edition = { workspace = true }`)
    // and virtual workspaces. If parsing fails cargo itself will surface the error;
    // don't double-report.
    let content = lines.join("\n");
    let parsed = match toml::from_str::<Value>(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    validate_edition(&parsed, lines, cargo_file, violations, required_edition);
    validate_rust_version_field(
        &parsed,
        lines,
        cargo_file,
        violations,
        required_rust_version,
    );
}

/// Resolution of a package field after accounting for workspace inheritance
enum ResolvedField<'a> {
    /// Literal string value (e.g. `edition = "2024"`)
    Value(&'a str),
    /// `{ workspace = true }` — the member inherits from the workspace root
    WorkspaceInherited,
    /// Present but not a string and not workspace inheritance — malformed; skip
    Other,
    /// Not present at all
    Absent,
}

fn classify_field(value: &Value) -> ResolvedField<'_> {
    if let Some(s) = value.as_str() {
        return ResolvedField::Value(s);
    }
    if let Some(tbl) = value.as_table()
        && tbl.get("workspace").and_then(Value::as_bool) == Some(true)
    {
        return ResolvedField::WorkspaceInherited;
    }
    ResolvedField::Other
}

/// Resolve a field by checking `[package].<field>` then `[workspace.package].<field>`.
fn resolve_field<'a>(parsed: &'a Value, field: &str) -> ResolvedField<'a> {
    if let Some(value) = parsed.get("package").and_then(|p| p.get(field)) {
        return classify_field(value);
    }
    if let Some(value) = parsed
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get(field))
    {
        return classify_field(value);
    }
    ResolvedField::Absent
}

/// A pure virtual manifest has `[workspace]` but no `[package]` section.
fn is_virtual_workspace(parsed: &Value) -> bool {
    parsed.get("package").is_none() && parsed.get("workspace").is_some()
}

/// Locate the 1-indexed line of a field in the original source for error reporting.
/// Matches both `edition = "..."` and `edition.workspace = true` forms.
fn find_field_line(lines: &[&str], field: &str) -> usize {
    let eq_key = format!("{field} =");
    let tight_key = format!("{field}=");
    let dotted_key = format!("{field}.workspace");
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&eq_key)
            || trimmed.starts_with(&tight_key)
            || trimmed.starts_with(&dotted_key)
        {
            return i + 1;
        }
    }
    0
}

/// Check edition field against locked required value
fn validate_edition(
    parsed: &Value,
    lines: &[&str],
    cargo_file: &Path,
    violations: &mut Vec<Violation>,
    required_edition: &str,
) {
    match resolve_field(parsed, "edition") {
        ResolvedField::Value(found_edition) => {
            if found_edition != required_edition {
                violations.push(Violation {
                    violation_type: ViolationType::WrongEdition,
                    file: cargo_file.to_path_buf(),
                    line: find_field_line(lines, "edition"),
                    message: format!(
                        "FERROUS FORGE [LOCKED SETTING] — Edition Violation\n\
                         ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                         \n\
                           Found:    edition = \"{found_edition}\"\n\
                           Required: edition = \"{required_edition}\"\n\
                           Lock:     .ferrous-forge/config.toml → required_edition = \"{required_edition}\"\n\
                         \n\
                           ⚠ AI AGENT NOTICE: This edition is locked by project configuration.\n\
                           DO NOT change required_edition without human approval.\n\
                           DO NOT downgrade Cargo.toml to match a lower edition.\n\
                         \n\
                           To fix: Update edition = \"{required_edition}\" in Cargo.toml, then run:\n\
                             cargo fix --edition"
                    ),
                    severity: Severity::Error,
                });
            }
        }
        ResolvedField::WorkspaceInherited => {
            // Member inherits from workspace root; that root's manifest is
            // validated separately when walking the project.
        }
        ResolvedField::Other => {
            // Malformed TOML value — cargo itself will reject; don't double-error.
        }
        ResolvedField::Absent => {
            // Virtual workspaces without an explicit [workspace.package].edition
            // are valid in cargo; members then must set their own. Only report if
            // this manifest actually defines a [package] section.
            if !is_virtual_workspace(parsed) {
                violations.push(Violation {
                    violation_type: ViolationType::WrongEdition,
                    file: cargo_file.to_path_buf(),
                    line: 0,
                    message: format!(
                        "FERROUS FORGE [LOCKED SETTING] — Missing Edition\n\
                         ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                         \n\
                           Missing edition specification in Cargo.toml.\n\
                           Required: edition = \"{required_edition}\"\n\
                         \n\
                           ⚠ AI AGENT NOTICE: Add edition = \"{required_edition}\" to [package] section."
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }
}

/// Check rust-version field against locked required value
fn validate_rust_version_field(
    parsed: &Value,
    lines: &[&str],
    cargo_file: &Path,
    violations: &mut Vec<Violation>,
    required_rust_version: &str,
) {
    if required_rust_version.is_empty() {
        return;
    }

    match resolve_field(parsed, "rust-version") {
        ResolvedField::Value(found_version) => {
            if found_version != required_rust_version {
                violations.push(Violation {
                    violation_type: ViolationType::OldRustVersion,
                    file: cargo_file.to_path_buf(),
                    line: find_field_line(lines, "rust-version"),
                    message: format!(
                        "FERROUS FORGE [LOCKED SETTING] — Rust Version Violation\n\
                         ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                         \n\
                           Found:    rust-version = \"{found_version}\"\n\
                           Required: rust-version = \"{required_rust_version}\"\n\
                           Lock:     .ferrous-forge/config.toml → required_rust_version = \"{required_rust_version}\"\n\
                         \n\
                           ⚠ AI AGENT NOTICE: This rust-version is locked by project configuration.\n\
                           DO NOT change required_rust_version without human approval.\n\
                           DO NOT downgrade rust-version to resolve compilation errors."
                    ),
                    severity: Severity::Error,
                });
            }
        }
        // Workspace-inherited or absent → nothing to validate here.
        // `rust-version` is optional in cargo so missing is acceptable.
        ResolvedField::WorkspaceInherited | ResolvedField::Other | ResolvedField::Absent => {}
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn parse(src: &str) -> Value {
        toml::from_str(src).unwrap()
    }

    /// Build a [package] fixture that matches the current default config so tests
    /// remain correct if `Config::default()` evolves.
    fn manifest_with_literal_fields() -> (String, Config) {
        let config = Config::default();
        let toml = format!(
            r#"
[package]
name = "x"
version = "0.1.0"
edition = "{edition}"
rust-version = "{rust}"
"#,
            edition = config.required_edition,
            rust = config.required_rust_version,
        );
        (toml, config)
    }

    #[test]
    fn test_resolve_edition_literal() {
        let (toml, config) = manifest_with_literal_fields();
        let v = parse(&toml);
        match resolve_field(&v, "edition") {
            ResolvedField::Value(s) => assert_eq!(s, config.required_edition),
            other => panic!("expected Value, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn test_resolve_edition_dotted_workspace_inherit() {
        // `edition.workspace = true` must be understood as inheritance,
        // not as `edition = "true"`.
        let v = parse(
            r#"
[package]
name = "x"
version.workspace = true
edition.workspace = true
"#,
        );
        assert!(matches!(
            resolve_field(&v, "edition"),
            ResolvedField::WorkspaceInherited
        ));
    }

    #[test]
    fn test_resolve_edition_inline_workspace_inherit() {
        let v = parse(
            r#"
[package]
name = "x"
edition = { workspace = true }
"#,
        );
        assert!(matches!(
            resolve_field(&v, "edition"),
            ResolvedField::WorkspaceInherited
        ));
    }

    #[test]
    fn test_resolve_edition_from_workspace_package() {
        // Virtual workspace manifest with [workspace.package].edition
        let config = Config::default();
        let toml = format!(
            r#"
[workspace]
members = ["a", "b"]

[workspace.package]
edition = "{edition}"
version = "0.1.0"
"#,
            edition = config.required_edition,
        );
        let v = parse(&toml);
        match resolve_field(&v, "edition") {
            ResolvedField::Value(s) => assert_eq!(s, config.required_edition),
            other => panic!("expected Value, got {:?}", std::mem::discriminant(&other)),
        }
        assert!(is_virtual_workspace(&v));
    }

    #[test]
    fn test_workspace_member_inheriting_is_not_flagged() {
        let config = Config::default();
        let v = parse(
            r#"
[package]
name = "child"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
"#,
        );
        let mut violations = Vec::new();
        validate_edition(
            &v,
            &[],
            Path::new("Cargo.toml"),
            &mut violations,
            &config.required_edition,
        );
        validate_rust_version_field(
            &v,
            &[],
            Path::new("Cargo.toml"),
            &mut violations,
            &config.required_rust_version,
        );
        assert!(
            violations.is_empty(),
            "workspace-inherited member should not produce violations, got: {violations:?}"
        );
    }

    #[test]
    fn test_virtual_workspace_without_edition_is_not_flagged() {
        let config = Config::default();
        let v = parse(
            r#"
[workspace]
members = ["a"]
"#,
        );
        let mut violations = Vec::new();
        validate_edition(
            &v,
            &[],
            Path::new("Cargo.toml"),
            &mut violations,
            &config.required_edition,
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn test_wrong_edition_still_flagged() {
        // Deliberately set a literal edition that cannot equal the required edition
        // from the default config, regardless of how the default evolves.
        let config = Config::default();
        let wrong_edition = if config.required_edition == "2018" {
            "2015"
        } else {
            "2018"
        };
        let toml = format!(
            r#"
[package]
name = "x"
version = "0.1.0"
edition = "{wrong_edition}"
"#
        );
        let v = parse(&toml);
        let lines: Vec<&str> = toml.lines().collect();
        let mut violations = Vec::new();
        validate_edition(
            &v,
            &lines,
            Path::new("Cargo.toml"),
            &mut violations,
            &config.required_edition,
        );
        assert_eq!(violations.len(), 1);
        // The edition line is the 5th line of the fixture (index 5: blank, [package], name, version, edition)
        assert!(violations[0].line > 0, "expected a concrete line number");
    }

    #[test]
    fn test_find_field_line_handles_dotted_form() {
        let lines = [
            "[package]",
            "name = \"x\"",
            "version.workspace = true",
            "edition.workspace = true",
        ];
        assert_eq!(find_field_line(&lines, "edition"), 4);
        assert_eq!(find_field_line(&lines, "version"), 3);
    }
}

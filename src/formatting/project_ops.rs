//! Project-level formatting operations

use crate::{Error, Result};
use super::types::{FormatResult, FormatSuggestion};
use super::utils::ensure_rustfmt_installed;
use std::path::Path;
use std::process::Command;
use tokio::fs;

/// Check formatting for entire project
pub async fn check_formatting(project_path: &Path) -> Result<FormatResult> {
    ensure_rustfmt_installed().await?;

    let output = Command::new("cargo")
        .args(&["fmt", "--", "--check"])
        .current_dir(project_path)
        .output()
        .map_err(|e| Error::process(format!("Failed to run rustfmt: {}", e)))?;

    let formatted = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(FormatResult {
        formatted,
        unformatted_files: if formatted {
            vec![]
        } else {
            vec![format!("Multiple files need formatting (see: {})", stderr)]
        },
        suggestions: vec![],
    })
}

/// Auto-format entire project
pub async fn auto_format(project_path: &Path) -> Result<()> {
    ensure_rustfmt_installed().await?;

    let status = Command::new("cargo")
        .args(&["fmt", "--all"])
        .current_dir(project_path)
        .status()
        .map_err(|e| Error::process(format!("Failed to run cargo fmt: {}", e)))?;

    if !status.success() {
        return Err(Error::process("Formatting failed".to_string()));
    }

    Ok(())
}

/// Get formatting diff for the project
pub async fn get_format_diff(project_path: &Path) -> Result<String> {
    ensure_rustfmt_installed().await?;

    let output = Command::new("cargo")
        .args(&["fmt", "--", "--check", "--print-diff"])
        .current_dir(project_path)
        .output()
        .map_err(|e| Error::process(format!("Failed to get format diff: {}", e)))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Apply rustfmt configuration to project
pub async fn apply_rustfmt_config(project_path: &Path) -> Result<()> {
    let config_content = r#"# Ferrous Forge rustfmt configuration
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
edition = "2021"
"#;

    let config_path = project_path.join("rustfmt.toml");
    fs::write(&config_path, config_content).await?;

    Ok(())
}
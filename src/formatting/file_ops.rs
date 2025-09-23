//! File-level formatting operations

use super::utils::ensure_rustfmt_installed;
use crate::{Error, Result};
use std::path::Path;
use std::process::Command;

/// Check if a single file is properly formatted
pub async fn check_file_formatting(file_path: &Path) -> Result<bool> {
    ensure_rustfmt_installed().await?;

    let output = Command::new("rustfmt")
        .args(&["--check", "--edition", "2021"])
        .arg(file_path)
        .output()
        .map_err(|e| Error::process(format!("Failed to check file formatting: {}", e)))?;

    Ok(output.status.success())
}

/// Format a single file
pub async fn format_file(file_path: &Path) -> Result<()> {
    ensure_rustfmt_installed().await?;

    let status = Command::new("rustfmt")
        .args(&["--edition", "2021"])
        .arg(file_path)
        .status()
        .map_err(|e| Error::process(format!("Failed to format file: {}", e)))?;

    if !status.success() {
        return Err(Error::process(format!(
            "Failed to format file: {}",
            file_path.display()
        )));
    }

    Ok(())
}

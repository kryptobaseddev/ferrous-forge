//! Formatting utility functions

use crate::{Error, Result};
use std::process::Command;

/// Ensure rustfmt is installed and available
pub(super) async fn ensure_rustfmt_installed() -> Result<()> {
    let output = Command::new("rustfmt")
        .arg("--version")
        .output()
        .map_err(|_| Error::tool_not_found("rustfmt"))?;

    if !output.status.success() {
        return Err(Error::tool_not_found("rustfmt"));
    }

    Ok(())
}

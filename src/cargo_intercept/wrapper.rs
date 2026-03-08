//! Cargo wrapper script management

use crate::{Error, Result};
use std::path::Path;

/// Create wrapper script for cargo publish
///
/// # Errors
///
/// Returns an error if writing the wrapper script or setting file permissions fails.
pub fn create_publish_wrapper(install_path: &Path) -> Result<()> {
    let wrapper_content = include_str!("../../templates/cargo-publish-wrapper.sh");

    let wrapper_path = install_path.join("cargo");
    std::fs::write(&wrapper_path, wrapper_content)
        .map_err(|e| Error::config(format!("Failed to create wrapper script: {}", e)))?;

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&wrapper_path)
            .map_err(|e| Error::config(format!("Failed to get metadata: {}", e)))?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&wrapper_path, perms)
            .map_err(|e| Error::config(format!("Failed to set permissions: {}", e)))?;
    }

    tracing::info!(
        "Cargo publish wrapper created at {}",
        wrapper_path.display()
    );
    Ok(())
}

/// Install cargo interception system
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined, the install
/// directory cannot be created, or the wrapper script cannot be written.
pub fn install_cargo_intercept() -> Result<()> {
    let install_dir = dirs::home_dir()
        .ok_or_else(|| Error::config("Unable to determine home directory"))?
        .join(".ferrous-forge")
        .join("bin");

    std::fs::create_dir_all(&install_dir)
        .map_err(|e| Error::config(format!("Failed to create install directory: {}", e)))?;

    create_publish_wrapper(&install_dir)?;

    println!("✅ Cargo interception installed");
    println!(
        "Add {} to your PATH to enable cargo publish validation",
        install_dir.display()
    );

    Ok(())
}

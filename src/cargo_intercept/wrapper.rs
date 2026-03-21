//! Cargo wrapper script management

use crate::{Error, Result};
use std::path::Path;

/// Get the cargo publish wrapper script content
///
/// @task T016
/// @epic T014
pub fn get_publish_wrapper_content() -> &'static str {
    r#"#!/bin/bash
# Ferrous Forge cargo publish wrapper
# This script intercepts cargo publish commands to run validation
#
# @task T016
# @epic T014

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Find original cargo binary
ORIGINAL_CARGO=""
while IFS= read -r -d '' cargo_path; do
    # Skip our wrapper
    if [[ "$(readlink -f "$cargo_path")" != "$(readlink -f "$0")" ]]; then
        ORIGINAL_CARGO="$cargo_path"
        break
    fi
done < <(which -a cargo | tr '\n' '\0' 2>/dev/null || true)

if [[ -z "$ORIGINAL_CARGO" ]]; then
    echo -e "${RED}❌ Error: Could not find original cargo binary${NC}" >&2
    exit 1
fi

# Check if this is a publish command
is_publish=false
for arg in "$@"; do
    if [[ "$arg" == "publish" ]]; then
        is_publish=true
        break
    fi
done

if [[ "$is_publish" == "true" ]]; then
    echo -e "${BLUE}🦀 Ferrous Forge: Intercepting cargo publish${NC}"

    # Check for force bypass environment variable (absolute override)
    if [[ "${FERROUS_FORGE_FORCE_BYPASS:-}" == "true" ]]; then
        echo -e "${YELLOW}⚠️  FERROUS FORGE FORCE BYPASSED — FERROUS_FORGE_FORCE_BYPASS=true${NC}"
        echo -e "${YELLOW}   All validation skipped. This should NEVER happen in production.${NC}"
        exec "$ORIGINAL_CARGO" "$@"
    fi

    # Check for active bypass via ferrous-forge CLI
    if command -v ferrous-forge >/dev/null 2>&1; then
        # Check if there's an active bypass for the publish stage
        if ferrous-forge safety check --stage publish 2>/dev/null | grep -q "BYPASSED"; then
            echo -e "${YELLOW}⚠️  Safety checks bypassed - proceeding with publish${NC}"
            exec "$ORIGINAL_CARGO" "$@"
        fi
    fi

    # Check for bypass environment variable (legacy support)
    if [[ "${FERROUS_FORGE_BYPASS:-}" == "true" ]]; then
        echo -e "${YELLOW}⚠️  FERROUS_FORGE_BYPASS enabled - skipping style checks${NC}"
        echo -e "${YELLOW}   (locked settings like edition/version are still enforced)${NC}"
    fi

    # Check if ferrous-forge is available
    if ! command -v ferrous-forge >/dev/null 2>&1; then
        echo -e "${RED}❌ ferrous-forge not found in PATH${NC}" >&2
        echo -e "${YELLOW}Install with: cargo install ferrous-forge${NC}" >&2
        exit 1
    fi

    echo -e "${BLUE}🔍 Running Ferrous Forge validation...${NC}"

    # Run validation with appropriate flags
    VALIDATE_ARGS="."
    if [[ "${FERROUS_FORGE_BYPASS:-}" == "true" ]]; then
        VALIDATE_ARGS=". --locked-only"
    fi

    # shellcheck disable=SC2086
    if ferrous-forge validate $VALIDATE_ARGS; then
        echo -e "${GREEN}✅ Validation passed, proceeding with publish${NC}"
    else
        echo -e "${RED}🛡️ Ferrous Forge validation failed - publish blocked!${NC}" >&2
        echo -e "${YELLOW}Run 'ferrous-forge validate' to see issues${NC}" >&2
        echo -e "${YELLOW}To bypass with audit logging:${NC}" >&2
        echo -e "${YELLOW}   ferrous-forge safety bypass --stage=publish --reason='Emergency security patch'${NC}" >&2
        exit 1
    fi

    echo -e "${BLUE}📦 Proceeding with cargo publish...${NC}"
fi

# Execute original cargo with all arguments
exec "$ORIGINAL_CARGO" "$@"
"#
}

/// Create wrapper script for cargo publish
///
/// # Errors
///
/// Returns an error if writing the wrapper script or setting file permissions fails.
pub fn create_publish_wrapper(install_path: &Path) -> Result<()> {
    let wrapper_content = get_publish_wrapper_content();

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

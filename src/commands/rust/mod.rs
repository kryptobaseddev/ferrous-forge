//! Rust version management commands

pub mod display;
pub mod utils;

use crate::rust_version::VersionManager;
use crate::Result;
use display::{
    display_recommendation, display_recommendation_details, display_recommendation_header,
    display_releases_list, display_version_status,
};
use utils::{create_spinner, fetch_latest_version};

/// Handle rust version check command
pub async fn handle_check(verbose: bool) -> Result<()> {
    let spinner = create_spinner("Checking Rust version...");
    let manager = VersionManager::new()?;

    let current = manager.check_current().await?;
    spinner.set_message("Fetching latest release information...");

    let latest = match fetch_latest_version(&manager, &spinner, &current).await {
        Some(release) => release,
        None => return Ok(()), // Error already handled in fetch_latest_version
    };

    spinner.finish_and_clear();

    display_version_status(&current, &latest, verbose);
    let recommendation = manager.get_recommendation().await?;
    display_recommendation(&recommendation);

    Ok(())
}

/// Handle rust recommendation command
pub async fn handle_recommend(stable_only: bool) -> Result<()> {
    let manager = VersionManager::new()?;
    let current = manager.check_current().await?;

    display_recommendation_header(&current);

    let recommendation = manager.get_recommendation().await?;
    display_recommendation_details(&recommendation, stable_only);

    Ok(())
}

/// Handle rust list command
pub async fn handle_list(count: usize) -> Result<()> {
    let manager = VersionManager::new()?;
    let releases = manager.get_recent_releases(count).await?;

    display_releases_list(&releases);

    Ok(())
}

//! Auto-update functionality

use super::types::{UpdateChannel, UpdateManager};
use crate::{config::Config, Result};

/// Check for automatic updates based on configuration
pub async fn check_auto_update() -> Result<()> {
    let config = Config::load_or_default().await?;
    
    if !config.auto_update {
        return Ok(());
    }

    let channel = config.update_channel.parse::<UpdateChannel>()?;
    let manager = UpdateManager::new(channel)?;
    
    if let Some(update_info) = manager.check_for_updates().await? {
        if manager.prompt_for_update(&update_info).await? {
            manager.perform_update(&update_info).await?;
        }
    }

    Ok(())
}
//! Utility functions for Rust version commands

use crate::rust_version::VersionManager;
use crate::{Error, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use semver::Version;

/// Create a progress spinner with a message
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

/// Fetch latest version with progress indication
pub async fn fetch_latest_version(
    manager: &VersionManager,
    spinner: &ProgressBar,
    current: &crate::rust_version::RustVersion,
) -> Option<Version> {
    match manager.get_latest_stable().await {
        Ok(release) => {
            let latest = &release.version;
            if latest > &current.version {
                spinner.set_message("Found newer version available...");
            } else {
                spinner.set_message("You're up to date!");
            }
            Some(latest.clone())
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("{}", style("❌ Failed to check latest version").red());
            
            match e {
                Error::Network(msg) => {
                    eprintln!("Network error: {}", msg);
                    eprintln!("💡 Check your internet connection and try again");
                }
                Error::Parse(msg) => {
                    eprintln!("Parse error: {}", msg);
                    eprintln!("💡 This might be a temporary issue with the Rust release API");
                }
                _ => {
                    eprintln!("Error: {}", e);
                }
            }
            
            None
        }
    }
}
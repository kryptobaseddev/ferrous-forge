//! Git hooks installation and removal logic for mandatory safety pipeline
//!
//! @task T017
//! @epic T014

use super::scripts::{COMMIT_MSG_HOOK, PRE_COMMIT_HOOK, PRE_PUSH_HOOK};
use crate::{Error, Result};
use std::path::Path;
use tokio::fs;

/// Install mandatory blocking git hooks for a project
///
/// These hooks BLOCK commits/pushes when safety checks fail.
/// Bypass is available via: ferrous-forge safety bypass --stage=...
///
/// # Errors
///
/// Returns [`Error::Validation`] if the path is not a git repository.
/// Returns [`Error::Process`] if hook files cannot be written.
pub async fn install_git_hooks(project_path: &Path) -> Result<()> {
    // Check if we're in a git repository
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        return Err(Error::validation(
            "Not a git repository. Run 'git init' first.".to_string(),
        ));
    }

    // Ensure hooks directory exists
    let hooks_dir = git_dir.join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)
            .await
            .map_err(|e| Error::process(format!("Failed to create hooks directory: {}", e)))?;
    }

    println!("🔒 Installing mandatory safety hooks...");

    // Install pre-commit hook (blocking)
    install_hook(&hooks_dir, "pre-commit", PRE_COMMIT_HOOK).await?;
    println!("  ✅ Installed blocking pre-commit hook");

    // Install pre-push hook (blocking)
    install_hook(&hooks_dir, "pre-push", PRE_PUSH_HOOK).await?;
    println!("  ✅ Installed blocking pre-push hook");

    // Install commit-msg hook (non-blocking, format check)
    install_hook(&hooks_dir, "commit-msg", COMMIT_MSG_HOOK).await?;
    println!("  ✅ Installed commit-msg hook");

    println!();
    println!("🛡️  Mandatory safety hooks installed!");
    println!();
    println!("These hooks will BLOCK commits/pushes that fail checks.");
    println!("To bypass temporarily: ferrous-forge safety bypass --stage=...");

    Ok(())
}

/// Install a single hook
async fn install_hook(hooks_dir: &Path, name: &str, content: &str) -> Result<()> {
    let hook_path = hooks_dir.join(name);

    // Check if hook already exists
    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path)
            .await
            .map_err(|e| Error::process(format!("Failed to read existing hook: {}", e)))?;

        if existing.contains("Ferrous Forge") {
            // Our hook is already installed - check if it's the new blocking version
            if existing.contains("🛡️  FERROUS FORGE BLOCKED") {
                // Already has the blocking version
                return Ok(());
            }
            // Has old version, needs upgrade
            println!("  🔄 Upgrading {} hook to blocking version", name);
        } else {
            // Backup existing hook
            let backup_path = hooks_dir.join(format!("{}.backup", name));
            fs::rename(&hook_path, &backup_path)
                .await
                .map_err(|e| Error::process(format!("Failed to backup existing hook: {}", e)))?;

            println!(
                "  ⚠️  Backed up existing {} hook to {}",
                name,
                backup_path.display()
            );
        }
    }

    // Write hook content
    fs::write(&hook_path, content)
        .await
        .map_err(|e| Error::process(format!("Failed to write hook: {}", e)))?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)
            .await
            .map_err(|e| Error::process(format!("Failed to get hook metadata: {}", e)))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)
            .await
            .map_err(|e| Error::process(format!("Failed to set hook permissions: {}", e)))?;
    }

    Ok(())
}

/// Remove git hooks from a project
///
/// # Errors
///
/// Returns [`Error::Process`] if hook files cannot be removed or backups cannot be restored.
pub async fn uninstall_git_hooks(project_path: &Path) -> Result<()> {
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        return Ok(()); // No git repo, nothing to uninstall
    }

    let hooks_dir = git_dir.join("hooks");

    println!("🗑️  Removing safety hooks...");

    // Remove our hooks
    for hook_name in &["pre-commit", "pre-push", "commit-msg"] {
        let hook_path = hooks_dir.join(hook_name);
        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path).await.unwrap_or_default();

            if content.contains("Ferrous Forge") {
                fs::remove_file(&hook_path)
                    .await
                    .map_err(|e| Error::process(format!("Failed to remove hook: {}", e)))?;
                println!("  ✅ Removed {} hook", hook_name);

                // Restore backup if exists
                let backup_path = hooks_dir.join(format!("{}.backup", hook_name));
                if backup_path.exists() {
                    fs::rename(&backup_path, &hook_path)
                        .await
                        .map_err(|e| Error::process(format!("Failed to restore backup: {}", e)))?;
                    println!("  ✅ Restored original {} hook", hook_name);
                }
            }
        }
    }

    println!("🎉 Safety hooks removed successfully!");
    Ok(())
}

/// Check if mandatory safety hooks are installed
///
/// Returns a tuple of (`pre_commit_installed`, `pre_push_installed`)
pub fn check_hooks_status(project_path: &Path) -> (bool, bool) {
    let hooks_dir = project_path.join(".git").join("hooks");

    let pre_commit_installed =
        if let Ok(content) = std::fs::read_to_string(hooks_dir.join("pre-commit")) {
            content.contains("Ferrous Forge") && content.contains("🛡️  FERROUS FORGE BLOCKED")
        } else {
            false
        };

    let pre_push_installed =
        if let Ok(content) = std::fs::read_to_string(hooks_dir.join("pre-push")) {
            content.contains("Ferrous Forge") && content.contains("🛡️  FERROUS FORGE BLOCKED")
        } else {
            false
        };

    (pre_commit_installed, pre_push_installed)
}

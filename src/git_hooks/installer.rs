//! Git hooks installation and removal logic

use super::scripts::{COMMIT_MSG_HOOK, PRE_COMMIT_HOOK, PRE_PUSH_HOOK};
use crate::{Error, Result};
use std::path::Path;
use tokio::fs;

/// Install git hooks for a project
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

    println!("📎 Installing git hooks...");

    // Install pre-commit hook
    install_hook(&hooks_dir, "pre-commit", PRE_COMMIT_HOOK).await?;
    println!("  ✅ Installed pre-commit hook");

    // Install pre-push hook
    install_hook(&hooks_dir, "pre-push", PRE_PUSH_HOOK).await?;
    println!("  ✅ Installed pre-push hook");

    // Install commit-msg hook
    install_hook(&hooks_dir, "commit-msg", COMMIT_MSG_HOOK).await?;
    println!("  ✅ Installed commit-msg hook");

    println!("🎉 Git hooks installed successfully!");
    println!();
    println!("Hooks will now run automatically:");
    println!("  • pre-commit: Validates code before each commit");
    println!("  • pre-push: Runs tests and full validation before push");
    println!("  • commit-msg: Ensures conventional commit format");
    println!();
    println!("To bypass hooks temporarily, use: git commit --no-verify");

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
            // Our hook is already installed
            return Ok(());
        }

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
pub async fn uninstall_git_hooks(project_path: &Path) -> Result<()> {
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        return Ok(()); // No git repo, nothing to uninstall
    }

    let hooks_dir = git_dir.join("hooks");

    println!("🗑️  Removing git hooks...");

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

    println!("🎉 Git hooks removed successfully!");
    Ok(())
}

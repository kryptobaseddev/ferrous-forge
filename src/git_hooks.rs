//! Git hooks installation and management
//!
//! This module provides functionality to install and manage git hooks
//! for automatic validation on commits.

use crate::{Error, Result};
use std::path::Path;
use tokio::fs;

/// Pre-commit hook script content
const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# Ferrous Forge pre-commit hook
# Automatically validates code before allowing commits

set -e

echo "🔨 Running Ferrous Forge pre-commit validation..."

# Check if ferrous-forge is installed
if ! command -v ferrous-forge >/dev/null 2>&1; then
    echo "⚠️  Ferrous Forge not found in PATH"
    echo "   Install with: cargo install ferrous-forge"
    echo "   Skipping validation..."
    exit 0
fi

# Run formatting check
echo "📝 Checking code formatting..."
if ! cargo fmt -- --check >/dev/null 2>&1; then
    echo "❌ Code is not formatted properly"
    echo "   Run 'cargo fmt' to fix formatting"
    exit 1
fi

# Run Ferrous Forge validation
echo "🔍 Running standards validation..."
if ! ferrous-forge validate --quiet; then
    echo "❌ Ferrous Forge validation failed"
    echo "   Run 'ferrous-forge validate' to see detailed errors"
    echo "   Fix all violations before committing"
    exit 1
fi

# Run clippy
echo "📎 Running clippy checks..."
if ! cargo clippy -- -D warnings 2>/dev/null; then
    echo "❌ Clippy found issues"
    echo "   Run 'cargo clippy' to see warnings"
    exit 1
fi

# Check for security vulnerabilities (if cargo-audit is installed)
if command -v cargo-audit >/dev/null 2>&1; then
    echo "🔒 Checking for security vulnerabilities..."
    if ! cargo audit --quiet 2>/dev/null; then
        echo "⚠️  Security vulnerabilities detected"
        echo "   Run 'cargo audit' for details"
        echo "   Consider fixing before committing"
        # Don't block commit for security issues (just warn)
    fi
fi

echo "✅ All pre-commit checks passed!"
"#;

/// Pre-push hook script content
const PRE_PUSH_HOOK: &str = r#"#!/bin/sh
# Ferrous Forge pre-push hook
# Runs comprehensive validation before pushing

set -e

echo "🔨 Running Ferrous Forge pre-push validation..."

# Run all tests
echo "🧪 Running tests..."
if ! cargo test --quiet; then
    echo "❌ Tests failed"
    echo "   Fix failing tests before pushing"
    exit 1
fi

# Run documentation check
echo "📚 Checking documentation..."
if ! cargo doc --no-deps --document-private-items >/dev/null 2>&1; then
    echo "⚠️  Documentation issues found"
    echo "   Run 'cargo doc' to see warnings"
fi

# Full validation
echo "🔍 Running full validation..."
if ! ferrous-forge validate; then
    echo "❌ Validation failed"
    echo "   Fix all issues before pushing"
    exit 1
fi

echo "✅ All pre-push checks passed!"
"#;

/// Commit message hook script content
const COMMIT_MSG_HOOK: &str = r#"#!/bin/sh
# Ferrous Forge commit message hook
# Enforces conventional commit format

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Check for conventional commit format
if ! echo "$COMMIT_MSG" | grep -qE '^(feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)(\([a-z0-9-]+\))?: .+'; then
    echo "❌ Invalid commit message format!"
    echo ""
    echo "📝 Please use conventional commit format:"
    echo "   <type>(<scope>): <description>"
    echo ""
    echo "Types: feat, fix, docs, style, refactor, perf, test, chore, build, ci, revert"
    echo ""
    echo "Examples:"
    echo "   feat: add new validation rule"
    echo "   fix(validation): correct line counting logic"
    echo "   docs: update README with new features"
    echo ""
    exit 1
fi

# Check message length
FIRST_LINE=$(echo "$COMMIT_MSG" | head -n1)
if [ ${#FIRST_LINE} -gt 72 ]; then
    echo "⚠️  Commit message first line is too long (${#FIRST_LINE} > 72 characters)"
    echo "   Consider making it more concise"
fi

echo "✅ Commit message format valid"
"#;

/// Install git hooks in a project
pub async fn install_git_hooks(project_path: &Path) -> Result<()> {
    // Check if it's a git repository
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        return Err(Error::validation(
            "Not a git repository. Run 'git init' first.",
        ));
    }

    let hooks_dir = git_dir.join("hooks");

    // Create hooks directory if it doesn't exist
    fs::create_dir_all(&hooks_dir)
        .await
        .map_err(|e| Error::process(format!("Failed to create hooks directory: {}", e)))?;

    println!("📝 Installing git hooks...");

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

    for hook_name in &["pre-commit", "pre-push", "commit-msg"] {
        let hook_path = hooks_dir.join(hook_name);

        if hook_path.exists() {
            // Check if it's our hook
            let content = fs::read_to_string(&hook_path).await.unwrap_or_default();

            if content.contains("Ferrous Forge") {
                fs::remove_file(&hook_path)
                    .await
                    .map_err(|e| Error::process(format!("Failed to remove hook: {}", e)))?;
                println!("  ✅ Removed {} hook", hook_name);

                // Restore backup if it exists
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

    println!("✅ Git hooks removed");

    Ok(())
}

/// Check if git hooks are installed
pub async fn check_hooks_installed(project_path: &Path) -> Result<bool> {
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        return Ok(false);
    }

    let hooks_dir = git_dir.join("hooks");
    let pre_commit = hooks_dir.join("pre-commit");

    if pre_commit.exists() {
        let content = fs::read_to_string(&pre_commit).await.unwrap_or_default();
        Ok(content.contains("Ferrous Forge"))
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_hooks_require_git_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir for test");
        let result = install_git_hooks(temp_dir.path()).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("Should have failed")
            .to_string()
            .contains("Not a git repository"));
    }

    #[tokio::test]
    async fn test_check_hooks_not_installed() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir for test");
        let installed = check_hooks_installed(temp_dir.path())
            .await
            .expect("Check should succeed");
        assert!(!installed);
    }
}

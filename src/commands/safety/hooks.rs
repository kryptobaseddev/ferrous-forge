//! Git hooks installation and cargo interception
//!
//! This module provides mandatory blocking hooks that prevent commits and pushes
//! when safety checks fail. Blocking is the default behavior with bypass available
//! via explicit command.
//!
//! @task T017
//! @epic T014

use crate::Result;
use std::fs;
use std::path::Path;

/// Display installation header
pub fn display_install_header() {
    println!("🔧 Installing Ferrous Forge Mandatory Safety Hooks");
    println!("{}", "=".repeat(55));
    println!();
    println!("These hooks will BLOCK commits/pushes that fail safety checks.");
    println!("Use 'ferrous-forge safety bypass' if you need to bypass temporarily.");
    println!();
}

/// Validate git repository and create hooks directory
///
/// # Errors
///
/// Returns an error if the project path is not a git repository or the
/// hooks directory cannot be created.
pub fn validate_git_repo_and_create_hooks_dir(project_path: &Path) -> Result<std::path::PathBuf> {
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        return Err(crate::error::Error::Config(
            "Not a git repository. Run 'git init' first.".to_string(),
        ));
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;
    Ok(hooks_dir)
}

/// Install pre-commit hook
///
/// # Errors
///
/// Returns an error if the hook file cannot be written or permissions
/// cannot be set.
pub fn install_pre_commit_hook(hooks_dir: &Path, force: bool) -> Result<()> {
    let pre_commit_path = hooks_dir.join("pre-commit");

    if pre_commit_path.exists() && !force {
        let content = fs::read_to_string(&pre_commit_path)?;
        if !content.contains("Ferrous Forge") {
            println!("⚠️  Existing pre-commit hook found. Use --force to overwrite.");
            return Ok(());
        }
    }

    let content = get_pre_commit_hook_content();
    install_hook(&pre_commit_path, &content)?;
    println!("✅ Installed blocking pre-commit hook");
    Ok(())
}

/// Install pre-push hook
///
/// # Errors
///
/// Returns an error if the hook file cannot be written or permissions
/// cannot be set.
pub fn install_pre_push_hook(hooks_dir: &Path, force: bool) -> Result<()> {
    let pre_push_path = hooks_dir.join("pre-push");

    if pre_push_path.exists() && !force {
        let content = fs::read_to_string(&pre_push_path)?;
        if !content.contains("Ferrous Forge") {
            println!("⚠️  Existing pre-push hook found. Use --force to overwrite.");
            return Ok(());
        }
    }

    let content = get_pre_push_hook_content();
    install_hook(&pre_push_path, &content)?;
    println!("✅ Installed blocking pre-push hook");
    Ok(())
}

/// Install cargo interception system
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined, the install
/// directory cannot be created, or the wrapper script fails to write.
pub fn install_cargo_interception(force: bool) -> Result<()> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| crate::Error::Config("Unable to determine home directory".into()))?;

    let install_dir = home_dir.join(".ferrous-forge").join("bin");
    fs::create_dir_all(&install_dir)?;

    let wrapper_path = install_dir.join("cargo");

    if wrapper_path.exists() && !force {
        println!("⚠️  Cargo wrapper already exists. Use --force to overwrite.");
        return Ok(());
    }

    let wrapper_content = crate::cargo_intercept::wrapper::get_publish_wrapper_content();

    fs::write(&wrapper_path, wrapper_content)?;
    set_executable_permissions(&wrapper_path)?;

    println!("✅ Installed cargo publish interception");
    println!("   Add {} to your PATH to enable", install_dir.display());

    Ok(())
}

/// Display installation success message
pub fn display_install_success(cargo_installed: bool) {
    println!();
    println!("🛡️  Mandatory Safety Hooks Installed Successfully!");
    println!();
    println!("What happens now:");
    println!("  • git commit  → runs validation → BLOCKS if fails");
    println!("  • git push    → runs tests + validation → BLOCKS if fails");
    println!();
    println!("To bypass temporarily (requires reason):");
    println!("  ferrous-forge safety bypass --stage=pre-commit --reason=\"WIP\"");
    println!("  ferrous-forge safety bypass --stage=pre-push --reason=\"Emergency fix\"");
    println!();
    println!("To bypass for a single git operation:");
    println!("  git commit --no-verify  ⚠️  Use sparingly!");
    println!();

    if cargo_installed {
        println!("To enable cargo publish interception:");
        println!("  Add ~/.ferrous-forge/bin to your PATH");
        println!();
    }

    println!("To uninstall:");
    println!("  ferrous-forge safety uninstall");
}

/// Get pre-commit hook content with mandatory blocking
fn get_pre_commit_hook_content() -> &'static str {
    r#"#!/bin/bash
# Ferrous Forge Mandatory Safety Pipeline - Pre-Commit Hook
# @task T017 @epic T014
# 
# This hook BLOCKS commits if safety checks fail.
# Use 'ferrous-forge safety bypass --stage=pre-commit --reason="..."' to bypass.

set -e

echo ""
echo "🛡️  Ferrous Forge Safety Pipeline - Pre-Commit"
echo "═══════════════════════════════════════════════════"

# Check if ferrous-forge is installed
if ! command -v ferrous-forge >/dev/null 2>&1; then
    echo ""
    echo "⚠️  Ferrous Forge not found in PATH"
    echo "   Install with: cargo install ferrous-forge"
    echo "   Skipping safety checks..."
    exit 0
fi

# Check for active bypass
BYPASS_CHECK=$(ferrous-forge safety check-bypass --stage=pre-commit 2>/dev/null || echo "none")
if [ "$BYPASS_CHECK" = "active" ]; then
    echo ""
    echo "⚠️  Safety checks bypassed (active bypass found)"
    exit 0
fi

# Run Ferrous Forge validation
echo ""
echo "🔍 Running mandatory validation checks..."
echo ""

if ! ferrous-forge validate --quiet 2>&1; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED COMMIT"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Validation failed. Fix the issues above before committing."
    echo ""
    echo "How to fix:"
    echo "  1. Run 'ferrous-forge validate' to see all errors"
    echo "  2. Fix the reported violations"
    echo "  3. Try committing again"
    echo ""
    echo "To bypass (requires reason):"
    echo "  ferrous-forge safety bypass --stage=pre-commit --reason=\"WIP commit\""
    echo ""
    echo "To bypass for this commit only:"
    echo "  git commit --no-verify"
    echo ""
    exit 1
fi

# Run clippy checks
echo "📎 Running clippy checks..."
if ! cargo clippy -- -D warnings 2>/dev/null; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED COMMIT"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Clippy checks failed. Fix the warnings before committing."
    echo ""
    echo "How to fix:"
    echo "  Run 'cargo clippy -- -D warnings' to see all issues"
    echo ""
    exit 1
fi

echo ""
echo "✅ All safety checks passed! Commit allowed."
echo ""
exit 0
"#
}

/// Get pre-push hook content with mandatory blocking
fn get_pre_push_hook_content() -> &'static str {
    r#"#!/bin/bash
# Ferrous Forge Mandatory Safety Pipeline - Pre-Push Hook
# @task T017 @epic T014
#
# This hook BLOCKS pushes if safety checks fail.
# Use 'ferrous-forge safety bypass --stage=pre-push --reason="..."' to bypass.

set -e

echo ""
echo "🛡️  Ferrous Forge Safety Pipeline - Pre-Push"
echo "═══════════════════════════════════════════════════"

# Check if ferrous-forge is installed
if ! command -v ferrous-forge >/dev/null 2>&1; then
    echo ""
    echo "⚠️  Ferrous Forge not found in PATH"
    echo "   Install with: cargo install ferrous-forge"
    echo "   Skipping safety checks..."
    exit 0
fi

# Check for active bypass
BYPASS_CHECK=$(ferrous-forge safety check-bypass --stage=pre-push 2>/dev/null || echo "none")
if [ "$BYPASS_CHECK" = "active" ]; then
    echo ""
    echo "⚠️  Safety checks bypassed (active bypass found)"
    exit 0
fi

# Run comprehensive checks
echo ""
echo "🔍 Running comprehensive safety checks..."
echo ""

# Run tests
echo "🧪 Running tests..."
if ! cargo test --quiet 2>&1; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED PUSH"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Tests failed. Fix failing tests before pushing."
    echo ""
    echo "How to fix:"
    echo "  Run 'cargo test' to see detailed test failures"
    echo ""
    echo "To bypass (requires reason):"
    echo "  ferrous-forge safety bypass --stage=pre-push --reason=\"Emergency fix\""
    echo ""
    exit 1
fi

# Run full validation
echo "🔍 Running validation..."
if ! ferrous-forge validate --quiet 2>&1; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED PUSH"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Validation failed. Fix the issues before pushing."
    echo ""
    echo "How to fix:"
    echo "  1. Run 'ferrous-forge validate' to see all errors"
    echo "  2. Fix the reported violations"
    echo "  3. Try pushing again"
    echo ""
    echo "To bypass (requires reason):"
    echo "  ferrous-forge safety bypass --stage=pre-push --reason=\"Emergency fix\""
    echo ""
    exit 1
fi

# Run security audit
echo "🔒 Running security audit..."
if command -v cargo-audit >/dev/null 2>&1; then
    if ! cargo audit 2>/dev/null; then
        echo ""
        echo "⚠️  Security audit found vulnerabilities"
        echo "   Run 'cargo audit' for details"
        echo ""
    fi
else
    echo "   (cargo-audit not installed, skipping)"
fi

echo ""
echo "✅ All safety checks passed! Push allowed."
echo ""
exit 0
"#
}

/// Install a hook file with proper permissions
fn install_hook(hook_path: &Path, content: &str) -> Result<()> {
    fs::write(hook_path, content)?;
    set_executable_permissions(hook_path)?;
    Ok(())
}

/// Set executable permissions on Unix systems
fn set_executable_permissions(#[allow(unused)] path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Uninstall git hooks from a project
///
/// # Errors
///
/// Returns an error if the project path is not a git repository or hook
/// files cannot be removed.
pub fn uninstall_hooks(project_path: &Path) -> Result<()> {
    let hooks_dir = project_path.join(".git").join("hooks");

    for hook_name in &["pre-commit", "pre-push"] {
        let hook_path = hooks_dir.join(hook_name);
        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path)?;
            if content.contains("Ferrous Forge") {
                fs::remove_file(&hook_path)?;
                println!("✅ Removed {} hook", hook_name);
            }
        }
    }

    println!("\n🗑️  Safety hooks uninstalled");
    Ok(())
}

/// Check if hooks are installed and active
///
/// Returns a tuple of (`pre_commit_installed`, `pre_push_installed`)
pub fn check_hooks_status(project_path: &Path) -> (bool, bool) {
    let hooks_dir = project_path.join(".git").join("hooks");

    let pre_commit_installed = if let Ok(content) = fs::read_to_string(hooks_dir.join("pre-commit"))
    {
        content.contains("Ferrous Forge")
    } else {
        false
    };

    let pre_push_installed = if let Ok(content) = fs::read_to_string(hooks_dir.join("pre-push")) {
        content.contains("Ferrous Forge")
    } else {
        false
    };

    (pre_commit_installed, pre_push_installed)
}

/// Get detailed hook status for display
pub fn get_hook_status_display(project_path: &Path) -> String {
    let (pre_commit, pre_push) = check_hooks_status(project_path);

    let mut status = String::from("Hook Status:\n");
    status.push_str(&format!(
        "  Pre-commit: {}\n",
        if pre_commit {
            "✅ installed"
        } else {
            "❌ not installed"
        }
    ));
    status.push_str(&format!(
        "  Pre-push:   {}\n",
        if pre_push {
            "✅ installed"
        } else {
            "❌ not installed"
        }
    ));

    if pre_commit && pre_push {
        status.push_str("\n🛡️  Mandatory blocking is ACTIVE\n");
    } else if pre_commit || pre_push {
        status.push_str("\n⚠️  Partial installation detected\n");
    } else {
        status.push_str("\n⚠️  Safety hooks are NOT installed\n");
        status.push_str("   Run: ferrous-forge safety install\n");
    }

    status
}

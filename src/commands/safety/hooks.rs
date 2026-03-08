//! Git hooks installation and cargo interception

use crate::Result;
use std::fs;
use std::path::Path;

/// Display installation header
pub fn display_install_header() {
    println!("🔧 Installing Ferrous Forge Safety Pipeline Git Hooks");
    println!("{}", "=".repeat(50));
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
        println!("⚠️  Pre-commit hook already exists. Use --force to overwrite.");
        return Ok(());
    }

    let content = get_pre_commit_hook_content();
    install_hook(&pre_commit_path, &content)?;
    println!("✅ Installed pre-commit hook");
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
        println!("⚠️  Pre-push hook already exists. Use --force to overwrite.");
        return Ok(());
    }

    let content = get_pre_push_hook_content();
    install_hook(&pre_push_path, &content)?;
    println!("✅ Installed pre-push hook");
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

    // Read wrapper script from templates
    let wrapper_content = include_str!("../../../templates/cargo-publish-wrapper.sh");

    fs::write(&wrapper_path, wrapper_content)?;
    set_executable_permissions(&wrapper_path)?;

    println!("✅ Installed cargo publish interception");
    println!("   Add {} to your PATH to enable", install_dir.display());

    Ok(())
}

/// Display installation success message
pub fn display_install_success(cargo_installed: bool) {
    println!("\n🎉 Safety pipeline installed successfully!");
    println!("\n📝 Next steps:");
    println!("   1. The pre-commit hook will run before each commit");
    println!("   2. The pre-push hook will run before each push");
    println!("   3. To bypass temporarily: git commit --no-verify");

    if cargo_installed {
        println!("   4. Add ~/.ferrous-forge/bin to your PATH for cargo interception");
        println!("   5. Use FERROUS_FORGE_BYPASS=true for emergency publish bypass");
    }

    println!(
        "   {}. To uninstall: Remove .git/hooks/pre-commit and pre-push",
        if cargo_installed { "6" } else { "4" }
    );
}

/// Get pre-commit hook content
fn get_pre_commit_hook_content() -> &'static str {
    r#"#!/bin/bash
# Ferrous Forge Safety Pipeline - Pre-Commit Hook

echo "🦀 Running Ferrous Forge safety checks..."
ferrous-forge safety check --stage pre-commit

if [ $? -ne 0 ]; then
    echo "❌ Safety checks failed. Please fix the issues before committing."
    exit 1
fi

echo "✅ All safety checks passed!"
exit 0
"#
}

/// Get pre-push hook content
fn get_pre_push_hook_content() -> &'static str {
    r#"#!/bin/bash
# Ferrous Forge Safety Pipeline - Pre-Push Hook

echo "🦀 Running Ferrous Forge safety checks..."
ferrous-forge safety check --stage pre-push

if [ $? -ne 0 ]; then
    echo "❌ Safety checks failed. Please fix the issues before pushing."
    exit 1
fi

echo "✅ All safety checks passed!"
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
fn set_executable_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

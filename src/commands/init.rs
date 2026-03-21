//! Initialize command implementation

use crate::{Result, config::Config};
use console::style;

/// Execute the system-wide init command
///
/// # Errors
///
/// Returns an error if the configuration cannot be loaded, directories
/// cannot be created, or system integration files fail to install.
pub async fn execute(force: bool) -> Result<()> {
    println!(
        "{}",
        style("🔨 Initializing Ferrous Forge...").bold().cyan()
    );

    let config = Config::load_or_default().await?;
    if check_already_initialized(&config, force)? {
        return Ok(());
    }

    perform_initialization(config).await?;
    print_completion_message();

    Ok(())
}

/// Execute the project-level init command (`ferrous-forge init --project`)
///
/// Writes project-level tooling files into the current directory:
/// rustfmt.toml, clippy.toml, .vscode/settings.json, `Cargo.toml` `[lints]`,
/// docs scaffold, .github/workflows/ci.yml, and git hooks.
///
/// # Errors
///
/// Returns an error if the current directory cannot be determined, no
/// `Cargo.toml` is found, or any tooling file fails to write.
pub async fn execute_project() -> Result<()> {
    println!(
        "{}",
        style("🔨 Setting up Ferrous Forge project tooling...")
            .bold()
            .cyan()
    );

    let project_path = std::env::current_dir()
        .map_err(|e| crate::Error::config(format!("Failed to get current directory: {}", e)))?;

    // Verify this is a Rust project
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(crate::Error::config(
            "No Cargo.toml found. Run 'ferrous-forge init --project' inside a Rust project.",
        ));
    }

    println!("📁 Project: {}", project_path.display());
    println!();

    write_rustfmt_toml(&project_path).await?;
    write_clippy_toml(&project_path).await?;
    write_vscode_settings(&project_path).await?;
    inject_cargo_toml_lints(&cargo_toml).await?;
    write_ferrous_config(&project_path).await?;
    create_docs_scaffold(&project_path).await?;
    write_ci_workflow(&project_path).await?;

    // Install mandatory safety hooks automatically (T017)
    println!("\n🔒 Installing mandatory safety hooks...");
    install_project_git_hooks(&project_path).await?;

    println!();
    println!("{}", style("🎉 Project tooling installed!").bold().green());
    println!();
    println!("Next steps:");
    println!("  cargo fmt          — format code");
    println!("  cargo clippy       — enforce doc + quality lints (now configured)");
    println!("  cargo doc --no-deps — verify documentation builds");
    println!("  ferrous-forge validate — validate against Ferrous Forge standards");

    Ok(())
}

// ── System init helpers ──────────────────────────────────────────────────────

fn check_already_initialized(config: &Config, force: bool) -> Result<bool> {
    if config.is_initialized() && !force {
        println!(
            "{}",
            style("✅ Ferrous Forge is already initialized!").green()
        );
        println!("Use --force to reinitialize.");
        return Ok(true);
    }
    Ok(false)
}

async fn perform_initialization(config: Config) -> Result<()> {
    println!("📁 Creating configuration directories...");
    config.ensure_directories().await?;

    println!("🔧 Setting up cargo command hijacking...");
    install_cargo_hijacking().await?;

    println!("📋 Installing clippy configuration...");
    install_clippy_config().await?;

    println!("🐚 Installing shell integration...");
    install_shell_integration().await?;

    let mut config = config;
    config.mark_initialized();
    config.save().await?;

    Ok(())
}

fn print_completion_message() {
    println!(
        "{}",
        style("🎉 Ferrous Forge initialization complete!")
            .bold()
            .green()
    );
    println!();
    println!("Next steps:");
    println!("• Restart your shell or run: source ~/.bashrc");
    println!("• Create a new project: cargo new my-project");
    println!("• All new projects will automatically use Edition 2024 + strict standards!");
}

async fn install_cargo_hijacking() -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| crate::Error::config("Could not find home directory"))?;

    let bin_dir = home_dir.join(".local").join("bin");
    tokio::fs::create_dir_all(&bin_dir).await?;

    let cargo_wrapper = include_str!("../../templates/cargo-wrapper.sh");
    let cargo_path = bin_dir.join("cargo");
    tokio::fs::write(&cargo_path, cargo_wrapper).await?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&cargo_path).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&cargo_path, perms).await?;
    }

    Ok(())
}

async fn install_clippy_config() -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| crate::Error::config("Could not find home directory"))?;

    let clippy_config = include_str!("../../templates/clippy.toml");
    let clippy_path = home_dir.join(".clippy.toml");
    tokio::fs::write(&clippy_path, clippy_config).await?;

    Ok(())
}

async fn install_shell_integration() -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| crate::Error::config("Could not find home directory"))?;

    let shell_config = format!(
        r#"
# Ferrous Forge - Rust Development Standards Enforcer
export PATH="$HOME/.local/bin:$PATH"

# Enable Ferrous Forge for all Rust development
export FERROUS_FORGE_ENABLED=1
"#
    );

    for shell_file in &[".bashrc", ".zshrc", ".profile"] {
        let shell_path = home_dir.join(shell_file);
        if shell_path.exists() {
            let mut contents = tokio::fs::read_to_string(&shell_path).await?;
            if !contents.contains("Ferrous Forge") {
                contents.push_str(&shell_config);
                tokio::fs::write(&shell_path, contents).await?;
            }
        }
    }

    Ok(())
}

// ── Project init helpers ─────────────────────────────────────────────────────

async fn write_rustfmt_toml(project_path: &std::path::Path) -> Result<()> {
    let path = project_path.join("rustfmt.toml");
    if path.exists() {
        println!("  ⏭  rustfmt.toml already exists, skipping");
        return Ok(());
    }
    let content = r#"# Ferrous Forge project rustfmt configuration
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
edition = "2024"
"#;
    tokio::fs::write(&path, content).await?;
    println!("  ✅ Written: rustfmt.toml");
    Ok(())
}

async fn write_clippy_toml(project_path: &std::path::Path) -> Result<()> {
    let path = project_path.join("clippy.toml");
    if path.exists() {
        println!("  ⏭  clippy.toml already exists, skipping");
        return Ok(());
    }
    let content = r#"# Ferrous Forge project clippy configuration
too-many-lines-threshold = 50
cognitive-complexity-threshold = 25
"#;
    tokio::fs::write(&path, content).await?;
    println!("  ✅ Written: clippy.toml");
    Ok(())
}

async fn write_vscode_settings(project_path: &std::path::Path) -> Result<()> {
    let vscode_dir = project_path.join(".vscode");
    let settings_path = vscode_dir.join("settings.json");

    if settings_path.exists() {
        println!("  ⏭  .vscode/settings.json already exists, skipping");
        return Ok(());
    }

    tokio::fs::create_dir_all(&vscode_dir).await?;

    let content = r#"{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": [
    "--",
    "-W", "clippy::missing_docs_in_private_items",
    "-W", "rustdoc::broken_intra_doc_links",
    "-W", "rustdoc::missing_crate_level_docs"
  ],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
"#;
    tokio::fs::write(&settings_path, content).await?;
    println!("  ✅ Written: .vscode/settings.json");
    Ok(())
}

async fn inject_cargo_toml_lints(cargo_toml: &std::path::Path) -> Result<()> {
    let content = tokio::fs::read_to_string(cargo_toml).await?;

    if content.contains("[lints]") || content.contains("[lints.rust]") {
        println!("  ⏭  Cargo.toml already has [lints] section, skipping");
        return Ok(());
    }

    let lints_block = r#"
[lints.rust]
missing_docs = "warn"
unsafe_code = "forbid"

[lints.rustdoc]
broken_intra_doc_links = "deny"
invalid_html_tags = "deny"
missing_crate_level_docs = "warn"
bare_urls = "warn"
redundant_explicit_links = "warn"
unescaped_backticks = "warn"

[lints.clippy]
missing_safety_doc = "deny"
missing_errors_doc = "warn"
missing_panics_doc = "warn"
empty_docs = "warn"
doc_markdown = "warn"
needless_doctest_main = "warn"
suspicious_doc_comments = "warn"
too_long_first_doc_paragraph = "warn"
unwrap_used = "deny"
expect_used = "deny"
"#;

    let mut new_content = content;
    new_content.push_str(lints_block);
    tokio::fs::write(cargo_toml, new_content).await?;
    println!("  ✅ Injected [lints] block into Cargo.toml");
    Ok(())
}

async fn write_ferrous_config(project_path: &std::path::Path) -> Result<()> {
    let ff_dir = project_path.join(".ferrous-forge");
    let config_path = ff_dir.join("config.toml");

    if config_path.exists() {
        println!("  ⏭  .ferrous-forge/config.toml already exists, skipping");
        return Ok(());
    }

    tokio::fs::create_dir_all(&ff_dir).await?;

    let content = r#"# Ferrous Forge project configuration
# These values are LOCKED — LLM agents must not modify them without human approval.

[validation]
max_file_lines = 300
max_function_lines = 50
required_edition = "2024"
required_rust_version = "1.85.0"
ban_underscore_bandaid = true
require_documentation = true
"#;
    tokio::fs::write(&config_path, content).await?;
    println!("  ✅ Written: .ferrous-forge/config.toml");
    Ok(())
}

async fn create_docs_scaffold(project_path: &std::path::Path) -> Result<()> {
    let adr_dir = project_path.join("docs").join("dev").join("adr");
    let specs_dir = project_path.join("docs").join("dev").join("specs");

    if !adr_dir.exists() {
        tokio::fs::create_dir_all(&adr_dir).await?;
        let readme = adr_dir.join("README.md");
        tokio::fs::write(
            &readme,
            "# Architecture Decision Records\n\n\
             This directory tracks architectural decisions for this project.\n\n\
             ## Format\n\n\
             Each ADR is a markdown file named `NNNN-short-title.md`.\n",
        )
        .await?;
        println!("  ✅ Created: docs/dev/adr/README.md");
    } else {
        println!("  ⏭  docs/dev/adr already exists, skipping");
    }

    if !specs_dir.exists() {
        tokio::fs::create_dir_all(&specs_dir).await?;
        println!("  ✅ Created: docs/dev/specs/");
    }

    Ok(())
}

async fn write_ci_workflow(project_path: &std::path::Path) -> Result<()> {
    let workflow_dir = project_path.join(".github").join("workflows");
    let ci_path = workflow_dir.join("ci.yml");

    if ci_path.exists() {
        println!("  ⏭  .github/workflows/ci.yml already exists, skipping");
        return Ok(());
    }

    tokio::fs::create_dir_all(&workflow_dir).await?;

    let content = r#"name: CI

on:
  push:
    branches: [main, master]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  ci:
    name: Build & Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache
        uses: Swatinem/rust-cache@v2

      - name: Format check
        run: cargo fmt --check

      - name: Clippy (with doc lints)
        run: cargo clippy --all-features -- -D warnings

      - name: Tests
        run: cargo test --all-features

      - name: Doc build
        run: cargo doc --no-deps --all-features
        env:
          RUSTDOCFLAGS: "-D warnings"

      - name: Security audit
        run: |
          cargo install cargo-audit --quiet
          cargo audit
"#;
    tokio::fs::write(&ci_path, content).await?;
    println!("  ✅ Written: .github/workflows/ci.yml");
    Ok(())
}

async fn install_project_git_hooks(project_path: &std::path::Path) -> Result<()> {
    // Use the existing git_hooks installer
    match crate::git_hooks::install_git_hooks(project_path).await {
        Ok(()) => {}
        Err(e) => {
            // Not fatal — project might not have a git repo yet
            println!("  ⚠️  Git hooks skipped: {}", e);
        }
    }
    Ok(())
}

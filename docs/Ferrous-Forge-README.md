# 🔨 Ferrous Forge
**The Aggressive Rust Development Standards Enforcer**
[![Crates.io](https://img.shields.io/crates/v/ferrous-forge)](https://crates.io/crates/ferrous-forge)
[![Documentation](https://docs.rs/ferrous-forge/badge.svg)](https://docs.rs/ferrous-forge)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![CI](https://github.com/kryptobaseddev/ferrous-forge/workflows/CI/badge.svg)](https://github.com/kryptobaseddev/ferrous-forge/actions)
> *"Like a blacksmith forges iron into steel, Ferrous Forge shapes your Rust code into perfection."*
Ferrous Forge is an **opinionated, aggressive Rust development standards enforcer**. Unlike other tools that suggest fixes, Forge **physically prevents non-compliant code** from being committed, pushed, or published.
**Version:** 1.7.6 | **MSRV:** 1.88 (Rust Edition 2024) | **License:** MIT OR Apache-2.0
## 🎯 Core Philosophy
1. **Preconfiguration** — Professional-grade standards from project creation
2. **Locking** — Critical settings are immutable without explicit justification
3. **Enforcement** — Blocks git operations by default when checks fail
4. **Agent-Proof** — Prevents LLM agents from "workaround-ing" standards
5. **Escape Hatches** — Bypass available with mandatory audit logging
## 🚀 Quick Start
```bash
# Install from crates.io
cargo install ferrous-forge
# Initialize system-wide (one-time setup)
ferrous-forge init
# Create a new project with Forge standards
cargo new my-project
cd my-project
ferrous-forge init --project
# That's it! All your development now follows professional standards
git commit -m "feat: initial commit"  # Will be validated
```
## ✨ Features
### 🔒 Configuration Locking
Lock critical settings to prevent accidental changes:
```bash
# Lock the Rust version
ferrous-forge config lock required_rust_version --reason="Team decision to stay on 1.88"
# Try to change it (blocked!)
# Editing Cargo.toml to change rust-version will fail validation
# Unlock with justification
ferrous-forge config unlock required_rust_version --reason="Security patch requires 1.89"
# View all locks
ferrous-forge config lock-status
```
### 🛡️ Safety Pipeline
Mandatory blocking hooks that prevent bad code:
```bash
# Install blocking hooks
ferrous-forge safety install
# Now git commit runs validation and blocks if failed
git commit -m "feat: add feature"  # Blocked if violations exist!
# Emergency bypass (requires reason)
ferrous-forge safety bypass --stage=pre-commit --reason="WIP commit, will fix later"
# View audit log
ferrous-forge safety audit
```
### 🦀 Rust Toolchain Management
Complete rustup integration:
```bash
# Check current version and available updates
ferrous-forge rust check
# List recent releases
ferrous-forge rust list
# Install specific toolchain
ferrous-forge rust install-toolchain nightly
# Switch default toolchain
ferrous-forge rust switch stable
# Update all toolchains
ferrous-forge rust update
```
### 📚 Edition Management
Migrate between Rust editions:
```bash
# Check edition compliance
ferrous-forge edition check
# Analyze before migrating
ferrous-forge edition analyze
# Migrate to Edition 2024
ferrous-forge edition migrate
```
### 📋 Project Templates
7 built-in templates with Forge standards pre-configured:
```bash
# List available templates
ferrous-forge template list
# Create from template
ferrous-forge template create my-cli --template=cli-app
ferrous-forge template create my-lib --template=library
ferrous-forge template create my-wasm --template=wasm
ferrous-forge template create my-embedded --template=embedded
```
## 🎯 CLI Reference
### Core Commands
| Command | Description |
|---------|-------------|
| `ferrous-forge init [--project]` | Initialize system or project |
| `ferrous-forge status` | Show installation status |
| `ferrous-forge validate [path]` | Validate project against standards |
| `ferrous-forge fix [path]` | Auto-fix code violations |
### Configuration Commands
| Command | Description |
|---------|-------------|
| `ferrous-forge config list` | Show all configuration |
| `ferrous-forge config set key=value` | Set configuration value |
| `ferrous-forge config lock <key>` | Lock a configuration value |
| `ferrous-forge config unlock <key>` | Unlock a configuration value |
| `ferrous-forge config lock-status` | Show lock status |
| `ferrous-forge config lock-audit` | View lock audit log |
| `ferrous-forge config export` | Export config for sharing |
| `ferrous-forge config import` | Import shared config |
### Safety Pipeline Commands
| Command | Description |
|---------|-------------|
| `ferrous-forge safety status` | Check pipeline status |
| `ferrous-forge safety install` | Install blocking git hooks |
| `ferrous-forge safety uninstall` | Remove git hooks |
| `ferrous-forge safety bypass` | Create emergency bypass |
| `ferrous-forge safety audit` | View bypass audit log |
| `ferrous-forge safety report` | View safety reports |
| `ferrous-forge safety stats` | Display safety statistics |
### Rust Management Commands
| Command | Description |
|---------|-------------|
| `ferrous-forge rust check` | Check version and updates |
| `ferrous-forge rust list` | List recent releases |
| `ferrous-forge rust update` | Update toolchains |
| `ferrous-forge rust install-toolchain <channel>` | Install toolchain |
| `ferrous-forge rust switch <channel>` | Switch default toolchain |
## ⚙️ Configuration
Ferrous Forge uses hierarchical configuration (System → User → Project):
```toml
# ~/.config/ferrous-forge/config.toml (User level)
[validation]
max_file_lines = 500
max_function_lines = 100
required_edition = "2024"
ban_underscore_bandaid = true
```
### Configuration Hierarchy
1. **System** (`/etc/ferrous-forge/config.toml`) — Organization-wide defaults
2. **User** (`~/.config/ferrous-forge/config.toml`) — Personal preferences
3. **Project** (`./.ferrous-forge/config.toml`) — Team-agreed standards
Later levels override earlier levels. Project config has highest priority.
## 🛡️ What Gets Enforced
### By Default (No Configuration Needed)
- **Edition 2024** — Latest Rust edition enforced
- **Zero Underscore Bandaid** — No `_unused` parameters
- **No unwrap/expect** — Proper error handling required
- **No panic/todo/unimplemented** — Production-ready code
- **File size limits** — 500 lines max per file
- **Function size limits** — 100 lines max per function
- **Documentation** — Public APIs require RustDoc
- **Clippy** — All, pedantic, and nursery lints enabled
### Git Hooks (Blocking)
When you run `ferrous-forge safety install`:
**Pre-commit hook:**
- ✅ Code formatting (`cargo fmt`)
- ✅ Clippy validation
- ✅ Ferrous Forge validation
**Pre-push hook:**
- ✅ All pre-commit checks
- ✅ Test suite (`cargo test`)
- ✅ Security audit (`cargo audit`)
**Both hooks:**
- Check for active bypasses before blocking
- Provide clear error messages
- Require explicit bypass with reason
## 📦 Installation
### From crates.io
```bash
cargo install ferrous-forge
```
### Package Managers (Coming Soon)
```bash
# Homebrew (macOS/Linux)
brew install ferrous-forge
# Arch Linux (AUR)
yay -S ferrous-forge
# Nix
nix-env -iA ferrous-forge
# Windows (Chocolatey)
choco install ferrous-forge
```
## 🔧 Project Setup
### New Project
```bash
cargo new my-project
cd my-project
ferrous-forge init --project
```
This creates:
- `.ferrous-forge/config.toml` — Project configuration
- `rustfmt.toml` — Formatting rules
- `clippy.toml` — Lint configuration
- `.git/hooks/pre-commit` — Blocking pre-commit hook
- `.git/hooks/pre-push` — Blocking pre-push hook
- `.vscode/settings.json` — IDE integration
- `.github/workflows/ci.yml` — CI/CD template
### Existing Project
```bash
cd existing-project
ferrous-forge init --project
ferrous-forge fix  # Auto-fix what can be fixed
```
## 🚫 Banned Patterns
These will cause validation to fail:
```rust
// ❌ Underscore bandaid
fn bad_function(_unused: String) {}
// ❌ Ignored results
let _ = some_result;
// ❌ Unwrap in production
some_value.unwrap();
// ❌ Wrong edition
edition = "2021"  // Should be "2024"
// ❌ Too many lines
fn huge_function() {  // 100+ lines
    // ...
}
```
## 🔒 Security
- **No unsafe code** — Ferrous Forge itself is 100% safe Rust
- **Sandboxed execution** — No elevated privileges required
- **Dependency scanning** — Automatic vulnerability detection
- **Audit logging** — All bypasses logged with timestamp and reason
## 📖 Documentation
- [**VISION.md**](VISION.md) — Core vision and philosophy
- [**ROADMAP.md**](ROADMAP.md) — Implementation roadmap
- [**FEATURES.md**](FEATURES.md) — Detailed feature status
## 🏗️ Architecture
```
┌─────────────────────────────────────────┐
│         Configuration Layers            │
├─────────────────────────────────────────┤
│ System → User → Project (merge order)   │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│          Safety Pipeline                │
├─────────────────────────────────────────┤
│ Pre-commit: Format → Clippy → Validate  │
│ Pre-push: Tests → Audit → Validate      │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│        Toolchain Management             │
├─────────────────────────────────────────┤
│ Rustup integration, edition management  │
│ GitHub API for release tracking         │
└─────────────────────────────────────────┘
```
## 🤝 Contributing
We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
**Priority Areas:**
1. Custom lint rules (Dylint integration)
2. IDE plugins (IntelliJ, Vim)
3. Documentation improvements
4. Bug fixes
## 📄 License
Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
at your option.
## 🎉 **My First Rust Crate!**
Hey there, fellow Rustaceans! 👋 This is my **very first Rust crate**.
After months of teaching myself Rust, I've built something I hope you'll find useful. This tool was born from frustration with inconsistent code standards and LLM agents working around lint rules.
**I'm super excited to learn and grow with the Rust community!** 🦀✨
### 💬 **Please, I Need Your Feedback!**
- 🤔 **Think this tool is useful?** Please let me know!
- 😅 **Think this is a stupid tool with no real need?** Tell me that too!
- 🚀 **Have ideas for improvements?** I'm all ears!
Thank you for taking the time to look at my work! 🙏
---
**Forge better Rust, automatically.** 🔨
[⭐ Star us on GitHub](https://github.com/kryptobaseddev/ferrous-forge) • [📦 Install from Crates.io](https://crates.io/crates/ferrous-forge) • [📖 Read the Docs](https://docs.rs/ferrous-forge)

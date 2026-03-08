# 🔨 Ferrous Forge

**The Type-Safe Rust Development Standards Enforcer**

[![Crates.io](https://img.shields.io/crates/v/ferrous-forge)](https://crates.io/crates/ferrous-forge)
[![Documentation](https://docs.rs/ferrous-forge/badge.svg)](https://docs.rs/ferrous-forge)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![CI](https://github.com/kryptobaseddev/ferrous-forge/workflows/CI/badge.svg)](https://github.com/kryptobaseddev/ferrous-forge/actions)

> *"Like a blacksmith forges iron into steel, Ferrous Forge shapes your Rust code into perfection."*

<!-- cargo-rdme start -->

<!-- cargo-rdme end -->

Ferrous Forge is a **system-wide Rust development standards enforcer** that automatically applies
professional-grade coding standards to your Rust projects. It integrates with cargo commands,
installs git hooks for pre-commit and pre-push validation, and provides a CLI for managing
Rust versions, editions, safety checks, and more.

**Version:** 1.7.2 | **MSRV:** 1.88 (Edition 2024) | **License:** MIT OR Apache-2.0

## ✨ Features

### 🦀 Rust Version Management
- **Version Checking** — Check your Rust installation against latest GitHub releases
- **Update Recommendations** — Intelligent update suggestions (security/major/minor)
- **Release Tracking** — List recent Rust releases with dates and release notes

### 📚 Edition Management
- **Compliance Checking** — Verify your project uses the latest Rust edition
- **Migration Assistant** — Guided migration from older editions to Edition 2024
- **Compatibility Analysis** — Pre-migration compatibility checking
- **Automated Fixes** — Integration with `cargo fix --edition`

### 🛡️ Safety Pipeline
- **Pre-Commit Protection** — Block commits that would fail CI
- **Pre-Push Validation** — Comprehensive checks before pushing to remote
- **Publish Safety** — Prevent broken crates from reaching crates.io
- **Emergency Bypass** — Hotfix capability with audit logging

### 🔧 Standards Enforcement
- 🚫 **Zero Underscore Bandaid** — Eliminates `_parameter` lazy patterns
- 📐 **Edition 2024 Enforcement** — Enforces latest Rust edition
- 🎯 **Comprehensive Clippy** — Enforces `clippy::all`, `clippy::pedantic`, and `clippy::nursery` lint groups, plus specific denies for `unwrap_used` and `expect_used`
- 📏 **Size Limits** — Default limits of 300-line files and 50-line functions (configurable)
- 📚 **Documentation Coverage** — Enforces RustDoc for public APIs with configurable thresholds
- 🔧 **System-Wide Integration** — Intercepts `cargo` and `rustc` commands via `ferrous-forge init`
- 🎯 **Zero Configuration** — Works immediately after installation
- 🔄 **Auto-Updates** — Keeps standards current with the Rust ecosystem
- 🧪 **Testing Enforced** — Requires comprehensive test coverage
- 🛡️ **Security First** — Automatic vulnerability scanning via `cargo audit`

## 🚀 Quick Start

```bash
# Install from crates.io
cargo install ferrous-forge

# Initialize system-wide standards (one-time setup)
ferrous-forge init

# Check your Rust version and get update recommendations
ferrous-forge rust check

# Check if your project uses the latest edition
ferrous-forge edition check

# That's it! All your Rust development now follows professional standards
cargo new my-project  # Automatically uses Edition 2024 + standards
```

## 📦 What Gets Enforced

### ✅ Every `cargo new`:
- Edition 2024 configuration
- Comprehensive clippy lint groups enabled
- Structured Cargo.toml with metadata
- Pre-configured GitHub Actions CI/CD
- Git hooks for validation
- Professional project structure

### 🔧 Every `cargo build/test/run`:
- Clippy validation (zero warnings policy)
- Format checking and auto-correction
- Security audit scanning
- Documentation completeness verification

### 🚫 Banned Patterns (produce compilation errors):
```rust
// ❌ These will cause compilation to fail:
fn bad_function(_unused: String) {}  // Underscore bandaid
let _ = some_result;                  // Ignored results
some_value.unwrap();                 // Unwrap in production
edition = "2021"                     // Wrong edition
```

## 🎯 CLI Reference

### Core Commands

| Command | Description |
|---------|-------------|
| `ferrous-forge init` | Initialize system-wide standards (use `--project` for project-level setup) |
| `ferrous-forge status` | Show installation status and current configuration |
| `ferrous-forge update` | Update to latest version (supports `--channel`, `--rules-only`, `--dry-run`) |
| `ferrous-forge validate [path]` | Validate a project against standards |
| `ferrous-forge rollback <version>` | Rollback to a previous version |
| `ferrous-forge uninstall` | Remove Ferrous Forge from the system |
| `ferrous-forge config` | Manage configuration (`--set`, `--get`, `--list`, `--reset`) |
| `ferrous-forge fix [path]` | Auto-fix code violations (`--dry-run`, `--only`, `--skip`) |

### 🦀 Rust Version Management

| Command | Description |
|---------|-------------|
| `ferrous-forge rust check` | Check current Rust version vs latest |
| `ferrous-forge rust recommend` | Get intelligent update recommendations |
| `ferrous-forge rust list` | List recent Rust releases |

### 📚 Edition Management

| Command | Description |
|---------|-------------|
| `ferrous-forge edition check` | Check edition compliance |
| `ferrous-forge edition analyze` | Analyze compatibility before migrating |
| `ferrous-forge edition migrate [edition]` | Migrate to a new edition (default: 2024) |

### 🛡️ Safety Pipeline

| Command | Description |
|---------|-------------|
| `ferrous-forge safety check` | Run safety checks (`--stage=pre-commit\|pre-push`) |
| `ferrous-forge safety test` | Test all safety checks |
| `ferrous-forge safety status` | View safety pipeline status |
| `ferrous-forge safety install` | Install git hooks for the safety pipeline |

### 📋 Template Management

| Command | Description |
|---------|-------------|
| `ferrous-forge template` | Manage project templates |

## 🔧 CI/CD Setup

### GitHub Actions
The repository includes comprehensive CI/CD workflows. To use them in your fork:

1. **Codecov Integration** (optional):
   - Sign up at [codecov.io](https://codecov.io)
   - Add your repository
   - Add `CODECOV_TOKEN` to your repository secrets

2. **GitHub Pages** (for documentation):
   - Go to Settings → Pages
   - Enable GitHub Pages
   - Set source to "GitHub Actions"

### Known CI Limitations
- **cargo-nextest**: Pinned to v0.9.72 for Rust 1.82 compatibility
- **Code coverage**: Requires Codecov token to avoid rate limits
- **Benchmarks**: Run in non-failing mode for stability

## ⚙️ Configuration

Ferrous Forge supports hierarchical configuration (system, user, project levels). Key configurable values:

| Setting | Default | Description |
|---------|---------|-------------|
| `max_file_lines` | 300 | Maximum lines per file |
| `max_function_lines` | 50 | Maximum lines per function |
| `required_edition` | 2024 | Required Rust edition |
| `ban_underscore_bandaid` | true | Ban `_parameter` patterns |
| `require_documentation` | true | Require RustDoc on public APIs |
| `auto_update` | true | Automatically check for updates |
| `update_channel` | stable | Release channel (stable/beta/nightly) |

```bash
# View all settings
ferrous-forge config --list

# Change a setting
ferrous-forge config --set max_file_lines=400

# Reset to defaults
ferrous-forge config --reset
```

## 📖 Documentation

- [**🦀 Rust Ecosystem Guide**](docs/rust-ecosystem-guide.md) — **New to Rust? Start here!**
- [**Installation Guide**](docs/installation.md) — Detailed setup instructions
- [**Configuration**](docs/configuration.md) — Customizing rules and settings
- [**Standards Reference**](docs/standards.md) — Complete list of enforced rules
- [**Integration Guide**](docs/integration.md) — IDE and tool integration
- [**Troubleshooting**](docs/troubleshooting.md) — Common issues and solutions
- [**Migration Guide**](docs/migration.md) — Upgrading existing projects

## 🛠️ Development & Contributing

We welcome contributions! Ferrous Forge is built with modern Rust practices:

- **Rust Edition 2024** (of course!)
- **100% Safe Rust** (no unsafe code)
- **Comprehensive Testing** (unit, integration, property-based)
- **CI/CD Pipeline** (automated testing, releases, security)
- **Documentation First** (every public API documented)

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Development Setup
```bash
git clone https://github.com/kryptobaseddev/ferrous-forge
cd ferrous-forge
cargo install --path .
cargo test --all-features
cargo doc --open
```

## 🏗️ Architecture

Ferrous Forge operates at multiple levels:

```
┌─────────────────────────────────────────┐
│           System Level                  │
├─────────────────────────────────────────┤
│ • Global Cargo configuration            │
│ • Cargo/rustc command interception      │
│ • Rust toolchain management             │
└─────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│          Project Level                  │
├─────────────────────────────────────────┤
│ • Template injection                    │
│ • Git hook installation                 │
│ • CI/CD configuration                   │
└─────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│          Runtime Level                  │
├─────────────────────────────────────────┤
│ • Clippy rule enforcement               │
│ • Real-time validation                  │
│ • Documentation coverage checks         │
└─────────────────────────────────────────┘
```

## 🔒 Security

- **Sandboxed execution** — No elevated privileges required
- **Dependency scanning** — Automatic vulnerability detection via `cargo audit`
- **Supply chain awareness** — Integrates with Rust security advisories

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🤝 Community

- **Issues**: [Bug Reports & Feature Requests](https://github.com/kryptobaseddev/ferrous-forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/kryptobaseddev/ferrous-forge/discussions)

## 🎉 **My First Rust Crate!**

Hey there, fellow Rustaceans! 👋 This is my **very first Rust crate**, and I am absolutely **thrilled** to share it with the community!

After months of teaching myself Rust (what an incredible journey it's been!), and with the amazing power of **Claude Code** accelerating my learning exponentially, I've built something I hope you'll find useful. This tool was born from my own frustration with inconsistent code standards across projects, and my desire to enforce professional-grade Rust practices automatically.

**I'm super excited to learn and grow with the Rust community!** 🦀✨

### 💬 **Please, I Need Your Feedback!**

- 🤔 **Think this tool is useful?** Please let me know!
- 😅 **Think this is a stupid tool with no real need?** Tell me that too - you can't hurt my feelings! (LOL)
- 🚀 **Have ideas for improvements?** I'm all ears!
- 🤝 **Want to contribute?** I'd be honored to have your help!
- 📚 **Spot any Rust anti-patterns?** Please teach me - I'm here to learn!

This is as much about my **Rust learning journey** as it is about building something useful. Thank you for taking the time to look at my work and for joining me on this adventure! 🙏

---

**Forge better Rust, automatically.** 🔨

[⭐ Star us on GitHub](https://github.com/kryptobaseddev/ferrous-forge) • [📦 Install from Crates.io](https://crates.io/crates/ferrous-forge) • [📖 Read the Docs](https://docs.rs/ferrous-forge)

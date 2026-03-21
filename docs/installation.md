# Installation Guide

This guide covers installing Ferrous Forge on your system.

## 🚀 Quick Installation

### Cargo Install (Recommended)

```bash
# Install the latest version from crates.io
cargo install ferrous-forge

# Verify installation
ferrous-forge --version
```

## 📋 Prerequisites

### Rust Toolchain

Ferrous Forge requires Rust 1.88 or newer (Edition 2024):

```bash
# Install or update Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Verify version
rustc --version  # Must be 1.88+
```

### Git

Git is required for project initialization:

```bash
# Verify git is installed
git --version
```

## 🔧 System Initialization

After installation, initialize Ferrous Forge:

```bash
# Initialize system-wide
ferrous-forge init

# Or initialize a specific project
cd my-project
ferrous-forge init --project
```

### What `ferrous-forge init` Does

1. Creates configuration directory (`~/.config/ferrous-forge/`)
2. Installs shell integration scripts
3. Sets up cargo wrapper for validation
4. Installs clippy configuration

### What `ferrous-forge init --project` Does

1. Creates `.rustfmt.toml` with project settings
2. Creates `.clippy.toml` with lint rules
3. Adds lints to `Cargo.toml`
4. Creates `.vscode/settings.json` (VS Code integration)
5. Creates `.github/workflows/ci.yml` template
6. Installs git hooks for pre-commit validation

## 🔍 Verification

Check that installation worked:

```bash
ferrous-forge status
```

Expected output shows:
- Ferrous Forge version
- Rust toolchain version
- Configuration status
- Git hooks status

## 🐧 Platform Support

Ferrous Forge works on:
- **Linux** (most distributions)
- **macOS** (Intel and Apple Silicon)
- **Windows** (with WSL2 recommended)

### Shell Support

- Bash 4.0+
- Zsh 5.0+
- Fish 3.0+
- PowerShell 7.0+ (Windows)

## 🔧 Troubleshooting

### "Command not found: ferrous-forge"

Ensure `~/.cargo/bin` is in your PATH:

```bash
# For bash/zsh
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# For fish
set -Ua fish_user_paths $HOME/.cargo/bin
```

### "Permission denied during init"

```bash
# Check ownership
ls -la ~/.cargo/

# Fix permissions (if needed)
sudo chown -R $(whoami) ~/.cargo

# Or force reinitialize
ferrous-forge init --force
```

### "Rust version too old"

```bash
# Update Rust
rustup update stable
rustup default stable

# Verify
rustc --version
```

### Reset Installation

If you need to start fresh:

```bash
# Uninstall
ferrous-forge uninstall --confirm

# Clean up configuration
rm -rf ~/.config/ferrous-forge

# Reinstall
cargo install ferrous-forge
ferrous-forge init
```

## 📞 Getting Help

If you encounter issues:

1. **Check status**: `ferrous-forge status`
2. **Read FEATURES.md**: See what's actually implemented
3. **Check GitHub Issues**: [github.com/kryptobaseddev/ferrous-forge/issues](https://github.com/kryptobaseddev/ferrous-forge/issues)

## 🎯 Next Steps

After installation:

1. Read [FEATURES.md](../FEATURES.md) to see what's available
2. Read [ROADMAP.md](../ROADMAP.md) to see what's planned
3. Try `ferrous-forge validate .` in a Rust project
4. Set up git hooks: `ferrous-forge safety install`

---

**Note:** Some features mentioned in older documentation may not be implemented yet. Always check [FEATURES.md](../FEATURES.md) for the current feature status.

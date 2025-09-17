# Installation Guide

This guide covers installing Ferrous Forge on your system and configuring it for automatic Rust standards enforcement.

## 🚀 Quick Installation

### Option 1: Cargo Install (Recommended)
```bash
# Install the latest stable version
cargo install ferrous-forge

# Initialize system-wide standards  
ferrous-forge init

# Verify installation
ferrous-forge status
```

### Option 2: Pre-built Binaries
```bash
# Download and install (Linux/macOS)
curl -sSL https://ferrous-forge.dev/install.sh | sh

# Or download manually from GitHub releases
wget https://github.com/yourusername/ferrous-forge/releases/latest/download/ferrous-forge-x86_64-unknown-linux-gnu.tar.gz
```

### Option 3: Package Managers

#### Homebrew (macOS/Linux)
```bash
brew install ferrous-forge
```

#### Arch Linux (AUR)
```bash
yay -S ferrous-forge
```

#### Nix/NixOS
```bash
nix-env -iA nixpkgs.ferrous-forge
```

## 📋 Prerequisites

### **Rust Toolchain**
Ferrous Forge requires Rust 1.82 or newer:

```bash
# Install or update Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Verify version (must be 1.82+)
rustc --version
```

### **Git**
Git is required for project initialization and hooks:

```bash
# Verify git is installed
git --version

# Configure git if not already done
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

### **Shell Support**
Ferrous Forge works with:
- **Bash** 4.0+
- **Zsh** 5.0+
- **Fish** 3.0+
- **PowerShell** 7.0+ (Windows)

## 🔧 System-Wide Initialization

After installation, run the initialization command:

```bash
ferrous-forge init
```

This command will:

1. **Install Required Tools**:
   - `clippy` - Rust linting
   - `rustfmt` - Code formatting
   - `cargo-audit` - Security scanning
   - `cargo-nextest` - Fast testing

2. **Configure Global Settings**:
   - Global Cargo configuration in `~/.cargo/config.toml`
   - Global Clippy rules in `~/.clippy.toml`
   - Shell integration scripts

3. **Set Up Command Hijacking**:
   - Wraps `cargo` commands with validation
   - Wraps `rustc` with Edition 2024 enforcement
   - Adds Ferrous Forge to your PATH

4. **Create Project Templates**:
   - Edition 2024 Cargo.toml template
   - Professional project structure
   - CI/CD configuration templates

## 🔍 Verification

Verify your installation works correctly:

### **Check Status**
```bash
ferrous-forge status
```

Expected output:
```
🔨 Ferrous Forge v0.1.0
✅ Rust toolchain: 1.82.0 (Edition 2024 ready)
✅ Global configuration: Installed
✅ Command hijacking: Active
✅ Project templates: Ready
✅ IDE integration: Available

🎯 System ready for professional Rust development!
```

### **Test Project Creation**
```bash
# Create a test project
cargo new test-ferrous-forge
cd test-ferrous-forge

# Verify it follows standards
cat Cargo.toml  # Should show edition = "2024"
ls -la          # Should include .clippy.toml

# Test validation
cargo build     # Should run validation first
```

### **Verify Command Hijacking**
```bash
# These commands should show validation output:
cargo build     # "🔨 Building with Ferrous Forge standards..."
cargo test      # "🧪 Testing with Ferrous Forge standards..."
cargo run       # "🚀 Running with Ferrous Forge standards..."
```

## 🎛️ Configuration Options

### **Release Channels**
Choose your update channel:

```bash
# Stable (default) - Thoroughly tested releases
ferrous-forge config set channel stable

# Beta - Preview upcoming features  
ferrous-forge config set channel beta

# Nightly - Latest development builds
ferrous-forge config set channel nightly
```

### **Validation Strictness**
Adjust validation levels:

```bash
# Strict (default) - Zero tolerance
ferrous-forge config set strictness strict

# Moderate - Warnings allowed
ferrous-forge config set strictness moderate  

# Permissive - Minimal enforcement
ferrous-forge config set strictness permissive
```

### **IDE Integration**
Enable automatic IDE configuration:

```bash
# Configure VS Code
ferrous-forge config set vscode.enabled true
ferrous-forge config set vscode.format-on-save true

# Configure rust-analyzer
ferrous-forge config set rust-analyzer.clippy true
ferrous-forge config set rust-analyzer.edition-2024 true
```

## 🔄 Updates

Ferrous Forge includes automatic update checking:

### **Manual Updates**
```bash
# Update to latest version
ferrous-forge update

# Update only standards rules
ferrous-forge update --rules-only

# Check for updates without installing
ferrous-forge update --dry-run
```

### **Automatic Updates**
```bash
# Enable automatic updates (default: weekly)
ferrous-forge config set auto-update.enabled true
ferrous-forge config set auto-update.frequency weekly

# Disable automatic updates
ferrous-forge config set auto-update.enabled false
```

## 🐧 Platform-Specific Setup

### **Linux**

#### Shell Integration
Add to your `~/.bashrc` or `~/.zshrc`:
```bash
# Ferrous Forge is automatically configured during init
# No manual additions needed
```

#### Desktop Integration
```bash
# Install desktop files (optional)
ferrous-forge install --desktop-files
```

### **macOS**

#### Homebrew Installation
```bash
# Install via Homebrew
brew install ferrous-forge

# Initialize
ferrous-forge init
```

#### PATH Configuration
Ferrous Forge automatically configures your PATH during `init`.

### **Windows**

#### PowerShell Setup
```powershell
# Install via cargo
cargo install ferrous-forge

# Initialize (requires admin for global config)
ferrous-forge init

# Manual PATH setup if needed
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
```

#### WSL2 Support
Ferrous Forge works seamlessly in WSL2:
```bash
# Install in WSL2
cargo install ferrous-forge
ferrous-forge init

# Works with Windows IDE integration
ferrous-forge config set wsl.windows-ide true
```

## 🔧 Troubleshooting

### **Common Issues**

#### "Command not found: ferrous-forge"
```bash
# Ensure ~/.cargo/bin is in PATH
echo $PATH | grep -q "$HOME/.cargo/bin" || echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### "Permission denied during init"
```bash
# Check file permissions
ls -la ~/.cargo/
chmod +w ~/.cargo/config.toml

# Or reinitialize with force
ferrous-forge init --force
```

#### "Rust version too old"
```bash
# Update Rust toolchain
rustup update stable
rustup default stable

# Verify version
rustc --version  # Should be 1.82+
```

#### "clippy not found"
```bash
# Install clippy component
rustup component add clippy rustfmt

# Verify installation
cargo clippy --version
```

### **Reset Installation**
If you encounter persistent issues:

```bash
# Complete reset (removes all configuration)
ferrous-forge uninstall --confirm
cargo uninstall ferrous-forge

# Clean reinstall
cargo install ferrous-forge
ferrous-forge init
```

## 📞 Getting Help

If you encounter issues not covered here:

1. **Check logs**: `ferrous-forge status --verbose`
2. **Search issues**: [GitHub Issues](https://github.com/yourusername/ferrous-forge/issues)
3. **Ask for help**: [GitHub Discussions](https://github.com/yourusername/ferrous-forge/discussions)
4. **Report bugs**: Use the issue template

## 🎯 Next Steps

After successful installation:

1. **Read the [Configuration Guide](configuration.md)** to customize behavior
2. **Review [Standards Reference](standards.md)** to understand enforced rules
3. **Set up [IDE Integration](integration.md)** for your editor
4. **Create your first project** with `cargo new my-project`

Welcome to professional Rust development with Ferrous Forge! 🔨✨
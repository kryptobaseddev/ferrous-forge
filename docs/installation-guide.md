# @task T023
# @epic T014
# Installation Guide

This guide covers installing Ferrous Forge on all supported platforms.

## Quick Install

### macOS / Linux (Homebrew)

```bash
brew tap kryptobaseddev/tap
brew install ferrous-forge
```

Or install directly from the formula:

```bash
brew install https://raw.githubusercontent.com/kryptobaseddev/ferrous-forge/main/packaging/homebrew/ferrous-forge.rb
```

### Arch Linux (AUR)

Using `yay`:

```bash
yay -S ferrous-forge
```

Using `paru`:

```bash
paru -S ferrous-forge
```

Manual installation:

```bash
git clone https://aur.archlinux.org/ferrous-forge.git
cd ferrous-forge
makepkg -si
```

### Nix / NixOS

```bash
nix-env -iA nixpkgs.ferrous-forge
```

Using flakes:

```bash
nix profile install github:kryptobaseddev/ferrous-forge
```

Add to your `flake.nix`:

```nix
{
  inputs.ferrous-forge.url = "github:kryptobaseddev/ferrous-forge";
  
  outputs = { self, nixpkgs, ferrous-forge }: {
    # ...
  };
}
```

### Windows (Chocolatey)

```powershell
choco install ferrous-forge
```

### Cargo (All Platforms)

```bash
cargo install ferrous-forge
```

## Post-Installation

After installation, initialize Ferrous Forge:

```bash
ferrous-forge init
```

This sets up:
- System-wide Cargo configuration
- Git hooks (optional)
- Shell integration

## Verify Installation

```bash
ferrous-forge --version
ferrous-forge --help
```

## Uninstallation

### Homebrew

```bash
brew uninstall ferrous-forge
```

### AUR

```bash
yay -R ferrous-forge
```

### Nix

```bash
nix-env -e ferrous-forge
```

### Chocolatey

```powershell
choco uninstall ferrous-forge
```

### Cargo

```bash
cargo uninstall ferrous-forge
```

## Updating

### Automatic Updates

Ferrous Forge can update itself:

```bash
ferrous-forge update
```

### Package Manager Updates

```bash
# Homebrew
brew update && brew upgrade ferrous-forge

# AUR
yay -Syu ferrous-forge

# Nix
nix-env -uA nixpkgs.ferrous-forge

# Chocolatey
choco upgrade ferrous-forge

# Cargo
cargo install ferrous-forge --force
```

## Troubleshooting

### Permission Denied

Ensure the binary is in your PATH:

```bash
which ferrous-forge
```

If not found, restart your terminal or reload your shell configuration:

```bash
source ~/.bashrc  # or ~/.zshrc, etc.
```

### Missing Dependencies

Ferrous Forge requires Rust 1.88 or later. Install from [rustup.rs](https://rustup.rs).

### Shell Completion Not Working

Shell completions are not yet included in packages. Run:

```bash
ferrous-forge completions bash > ~/.local/share/bash-completion/completions/ferrous-forge
# or
ferrous-forge completions zsh > ~/.zsh/completions/_ferrous-forge
# or
ferrous-forge completions fish > ~/.config/fish/completions/ferrous-forge.fish
```

## Platform-Specific Notes

### macOS

- **Apple Silicon (M1/M2/M3)**: Native aarch64 binary available
- **Intel**: x86_64 binary available
- Both architectures are supported via Homebrew

### Linux

- **glibc**: Standard binary for most distributions
- **musl**: Statically linked binary available for minimal systems

### Windows

- Run PowerShell as Administrator for system-wide installation
- Windows Defender may flag the binary; add an exclusion if needed

## Building from Source

See [CONTRIBUTING.md](../CONTRIBUTING.md) for build instructions.

## Support

- **Issues**: [GitHub Issues](https://github.com/kryptobaseddev/ferrous-forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/kryptobaseddev/ferrous-forge/discussions)

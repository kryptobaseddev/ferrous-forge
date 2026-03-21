# AUR Package for Ferrous Forge

This directory contains the Arch Linux User Repository (AUR) package files.

## Files

- `PKGBUILD` — Package build script
- `.SRCINFO` — Package metadata for AUR

## Installation

### Using an AUR Helper

```bash
yay -S ferrous-forge
# or
paru -S ferrous-forge
```

### Manual Installation

```bash
git clone https://aur.archlinux.org/ferrous-forge.git
cd ferrous-forge
makepkg -si
```

## Updating .SRCINFO

After modifying `PKGBUILD`, update `.SRCINFO`:

```bash
makepkg --printsrcinfo > .SRCINFO
```

## Package Details

- **Name**: `ferrous-forge`
- **Version**: 1.7.6
- **Architectures**: x86_64, aarch64
- **License**: MIT, Apache
- **Dependencies**: rust, cargo
- **Optional Dependencies**: git (for git hook integration)

## Submitting to AUR

1. Create an account on [aur.archlinux.org](https://aur.archlinux.org)
2. Generate SSH keys and add to your account
3. Clone the AUR repository:
   ```bash
   git clone ssh://aur@aur.archlinux.org/ferrous-forge.git
   ```
4. Copy files from this directory:
   ```bash
   cp packaging/aur/PKGBUILD ferrous-forge/
   cp packaging/aur/.SRCINFO ferrous-forge/
   ```
5. Commit and push:
   ```bash
   cd ferrous-forge
   git add PKGBUILD .SRCINFO
   git commit -m "Update to version X.Y.Z"
   git push
   ```

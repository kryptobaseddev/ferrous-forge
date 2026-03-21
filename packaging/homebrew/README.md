# Homebrew Formula for Ferrous Forge

This directory contains the Homebrew formula for Ferrous Forge.

## Installation

```bash
brew tap kryptobaseddev/tap
brew install ferrous-forge
```

Or install directly:

```bash
brew install https://raw.githubusercontent.com/kryptobaseddev/ferrous-forge/main/packaging/homebrew/ferrous-forge.rb
```

## Formula Details

- **Name**: `ferrous-forge`
- **Version**: 1.7.6
- **Platforms**: macOS (x86_64, aarch64), Linux (x86_64)
- **Dependencies**: Rust (optional, for building from source)

## Updating the Formula

The formula is automatically updated by the `update-packages.yml` workflow when a new release is published.

To update manually:

1. Download the release artifacts
2. Calculate SHA256 checksums:
   ```bash
   sha256sum ferrous-forge-linux-x86_64.tar.gz
   sha256sum ferrous-forge-macos-x86_64.tar.gz
   sha256sum ferrous-forge-macos-aarch64.tar.gz
   ```
3. Update the version and SHA256 values in `ferrous-forge.rb`

## Testing

```bash
brew install --build-from-source ./ferrous-forge.rb
brew test ferrous-forge
```

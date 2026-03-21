# Chocolatey Package for Ferrous Forge

This directory contains the Chocolatey package for Windows.

## Files

- `ferrous-forge.nuspec` — Package specification
- `tools/chocolateyinstall.ps1` — Installation script
- `tools/chocolateyuninstall.ps1` — Uninstallation script

## Installation

```powershell
choco install ferrous-forge
```

## Package Details

- **Id**: `ferrous-forge`
- **Version**: 1.7.6
- **Dependencies**: rust (≥1.88.0)

## Building the Package

### Prerequisites

- Chocolatey installed
- Windows PowerShell

### Build Steps

1. Navigate to this directory:
   ```powershell
   cd packaging\chocolatey
   ```

2. Pack the package:
   ```powershell
   choco pack
   ```

3. Test locally:
   ```powershell
   choco install ferrous-forge -s .
   ```

## Publishing to Chocolatey

1. Create an account on [chocolatey.org](https://chocolatey.org)
2. Get API key from your profile
3. Push the package:
   ```powershell
   choco push ferrous-forge.1.7.6.nupkg --source https://push.chocolatey.org/ --api-key YOUR_API_KEY
   ```

## Updating the Package

1. Update version in `ferrous-forge.nuspec`
2. Update version and checksum in `tools/chocolateyinstall.ps1`
3. Get new checksum:
   ```powershell
   Get-FileHash ferrous-forge-windows-x86_64.zip
   ```
4. Pack and push

## Testing

```powershell
# Install
choco install ferrous-forge -s . -y

# Verify
ferrous-forge --version

# Uninstall
choco uninstall ferrous-forge
```

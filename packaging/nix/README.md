# Nix Package for Ferrous Forge

This directory contains Nix expressions for Ferrous Forge.

## Files

- `default.nix` — Main package derivation
- `shell.nix` — Development shell
- `flake.nix` — Flake for modern Nix

## Installation

### Using nix-env

```bash
nix-env -iA nixpkgs.ferrous-forge
```

### Using nix profile (flakes)

```bash
nix profile install github:kryptobaseddev/ferrous-forge
```

### Using flakes in your project

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    ferrous-forge.url = "github:kryptobaseddev/ferrous-forge";
  };

  outputs = { self, nixpkgs, ferrous-forge }: {
    devShells.default = nixpkgs.legacyPackages.x86_64-linux.mkShell {
      buildInputs = [ ferrous-forge.packages.x86_64-linux.default ];
    };
  };
}
```

## Development Shell

```bash
nix-shell
# or with flakes
nix develop
```

## Building

```bash
nix-build
# or with flakes
nix build
```

## Updating

The Nix package requires manual hash updates:

1. Update the version in `default.nix` and `flake.nix`
2. Get the new source hash:
   ```bash
   nix-prefetch-url --unpack https://github.com/kryptobaseddev/ferrous-forge/archive/vX.Y.Z.tar.gz
   ```
3. Update the hash in `default.nix`
4. Build and get the cargo hash (first build will fail with correct hash):
   ```bash
   nix-build 2>&1 | grep "got:"
   ```
5. Update the `cargoHash` in `default.nix`

## Package Details

- **Name**: `ferrous-forge`
- **Version**: 1.7.6
- **Platforms**: Linux, macOS
- **Features**: update-system, telemetry

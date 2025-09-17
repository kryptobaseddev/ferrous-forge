# Configuration Guide

## Overview

Ferrous Forge uses a TOML-based configuration system that allows you to customize standards enforcement for your development environment.

## Configuration File Location

The main configuration file is located at:
- **Linux/macOS**: `~/.config/ferrous-forge/config.toml`
- **Windows**: `%APPDATA%\ferrous-forge\config.toml`

## Configuration Management

### View Current Configuration

```bash
# List all configuration values
ferrous-forge config --list

# Get a specific configuration value
ferrous-forge config --get key.name
```

### Modify Configuration

```bash
# Set a configuration value
ferrous-forge config --set key.name=value

# Reset to defaults
ferrous-forge config --reset
```

## Configuration Options

### Standards Settings

```toml
[standards]
# Edition enforcement
edition = "2024"
auto_upgrade = true

# Size limits
max_file_lines = 300
max_function_lines = 50
max_line_length = 100

# Pattern detection
ban_underscore_params = true
ban_underscore_let = true
ban_unwrap = true
ban_expect = true
```

### Validation Settings

```toml
[validation]
# Enable/disable specific checks
check_documentation = true
check_tests = false  # Currently in development
check_security = false  # Currently in development

# Validation behavior
fail_fast = false
show_suggestions = true
```

### Update Settings

```toml
[updates]
# Update channel
channel = "stable"  # stable, beta, nightly

# Auto-update behavior
check_updates = true
auto_update = false
```

## Project-Level Configuration

Projects can have their own `.ferrous-forge.toml` file that overrides global settings:

```toml
# .ferrous-forge.toml in project root
[project]
# Project-specific overrides
max_file_lines = 500  # Allow larger files for this project

[ignore]
# Files/patterns to ignore
patterns = [
    "tests/fixtures/**",
    "benches/**",
    "target/**"
]
```

## Environment Variables

Ferrous Forge respects the following environment variables:

- `FERROUS_FORGE_ENABLED` - Set to `0` to temporarily disable Ferrous Forge
- `FERROUS_FORGE_CONFIG` - Override config file location
- `FERROUS_FORGE_LOG_LEVEL` - Set logging verbosity (trace, debug, info, warn, error)

## Clippy Configuration

Ferrous Forge installs a global `.clippy.toml` file with strict linting rules. This file is located at `~/.clippy.toml` and contains 280+ configured lints.

To view or modify clippy settings:

```bash
# View current clippy configuration
cat ~/.clippy.toml

# Edit clippy configuration
$EDITOR ~/.clippy.toml
```

## Troubleshooting Configuration

### Configuration Not Loading

1. Check file exists: `ls -la ~/.config/ferrous-forge/`
2. Validate TOML syntax: `ferrous-forge config --list`
3. Check permissions: `chmod 644 ~/.config/ferrous-forge/config.toml`

### Resetting Configuration

If configuration becomes corrupted:

```bash
# Backup current config
cp ~/.config/ferrous-forge/config.toml ~/.config/ferrous-forge/config.toml.bak

# Reset to defaults
ferrous-forge config --reset

# Or manually delete and reinitialize
rm -rf ~/.config/ferrous-forge
ferrous-forge init
```

## Default Configuration

Here's the complete default configuration that Ferrous Forge uses:

```toml
[general]
version = "0.1.0"
initialized = true

[standards]
edition = "2024"
rust_version = "1.85"
auto_upgrade = true

[standards.file_limits]
max_lines = 300
max_line_length = 100

[standards.function_limits]
max_lines = 50
max_parameters = 7
max_complexity = 10

[standards.patterns]
ban_underscore_params = true
ban_underscore_let = true
ban_unwrap = true
ban_expect = true
ban_panic = true
ban_todo = true
ban_unimplemented = true

[validation]
check_documentation = true
fail_fast = false
show_suggestions = true

[updates]
channel = "stable"
check_updates = true
auto_update = false
```
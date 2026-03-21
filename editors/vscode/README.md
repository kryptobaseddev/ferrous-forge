# Ferrous Forge VS Code Extension

Real-time validation, inline diagnostics, and quick fixes for [Ferrous Forge](https://github.com/kryptobaseddev/ferrous-forge) - the type-safe Rust development standards enforcer.

## Features

### Real-time Validation
- **As-you-type validation** with configurable debounce delay
- **On-save validation** for comprehensive checks
- **On-open validation** for immediate feedback
- Shows violations inline with squiggly lines

### Inline Diagnostics
- **Error-level violations** shown in red
- **Warning-level violations** shown in yellow
- Click diagnostics to navigate to the exact line
- Hover for detailed violation information

### Quick Fixes (Code Actions)
- **Edition mismatches** - Update Cargo.toml or run `cargo fix`
- **Rust version issues** - Update configuration or run `rustup update`
- **Unwrap/Expect usage** - Replace with proper error handling
- **Missing documentation** - Add doc comments with one click
- **Line length violations** - Format with rustfmt
- **Fix all violations** with a single action

### Status Bar Indicator
- Shows validation status at a glance
- **Green**: All checks passed
- **Yellow with count**: Violations found
- **Red**: Validation error
- Click to show detailed output

### Configuration Panel
Access settings via VS Code preferences (`Ctrl+,`):

| Setting | Default | Description |
|---------|---------|-------------|
| `ferrousForge.enable` | `true` | Enable/disable Ferrous Forge validation |
| `ferrousForge.validateOnType` | `true` | Validate while typing (debounced) |
| `ferrousForge.validateOnSave` | `true` | Validate on file save |
| `ferrousForge.validateOnOpen` | `true` | Validate when files are opened |
| `ferrousForge.debounceDelay` | `500` | Delay (ms) before validating while typing |
| `ferrousForge.executablePath` | `ferrous-forge` | Path to ferrous-forge binary |
| `ferrousForge.showStatusBar` | `true` | Show status bar indicator |
| `ferrousForge.diagnosticSeverity` | `error` | Default severity for violations |
| `ferrousForge.enableQuickFixes` | `true` | Enable quick fix code actions |

## Commands

Access commands via Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`):

| Command | Keybinding | Description |
|---------|------------|-------------|
| **Ferrous Forge: Validate Project** | `Ctrl+Shift+V` / `Cmd+Shift+V` | Run full project validation |
| **Ferrous Forge: Validate Current File** | `Ctrl+Shift+Alt+V` / `Cmd+Shift+Alt+V` | Validate current Rust file |
| **Ferrous Forge: Fix All Violations** | - | Run `ferrous-forge fix` |
| **Ferrous Forge: Show Output Channel** | - | Open the output panel |
| **Ferrous Forge: Reload Configuration** | - | Reload settings from `.forge/config.toml` |

## Project Configuration

The extension automatically reads your project's Ferrous Forge configuration from `.forge/config.toml`:

```toml
# .forge/config.toml
edition = "2024"
rust_version = "1.88"

[limits]
max_file_lines = 500
max_function_lines = 100
max_line_length = 100
```

Changes to this file are automatically detected and applied.

## Requirements

- [Ferrous Forge](https://github.com/kryptobaseddev/ferrous-forge) installed and in your PATH
- VS Code 1.85.0 or higher
- Rust extension (recommended for syntax highlighting)

### Installing Ferrous Forge

```bash
cargo install ferrous-forge
```

Or install from source:

```bash
git clone https://github.com/kryptobaseddev/ferrous-forge.git
cd ferrous-forge
cargo install --path .
```

## Usage

1. Open a Rust project with Ferrous Forge configured
2. The extension activates automatically for `.rs` files
3. Edit your code - violations appear inline
4. Click the lightbulb (💡) or press `Ctrl+.` for quick fixes
5. View status in the status bar at the bottom right

## Keyboard Shortcuts

- **Validate Project**: `Ctrl+Shift+V` / `Cmd+Shift+V`
- **Validate Current File**: `Ctrl+Shift+Alt+V` / `Cmd+Shift+Alt+V`
- **Quick Fix**: `Ctrl+.` / `Cmd+.` (when cursor is on a violation)

## Offline Support

The extension works entirely offline by using your local Ferrous Forge installation. No network calls are made for validation.

## Supported Violations

The following violations trigger diagnostics and may have quick fixes:

| Violation | Severity | Quick Fix |
|-----------|----------|-----------|
| WrongEdition | Error | Update Cargo.toml, run cargo fix |
| OldRustVersion | Error | Update configuration, rustup update |
| UnwrapInProduction | Error | Replace with ?, match, or expect |
| ExpectInProduction | Error | Replace with proper error handling |
| MissingDocs | Warning | Add /// doc comment |
| MissingModuleDoc | Warning | Add //! module documentation |
| LineTooLong | Warning | Format with rustfmt |
| FileTooLarge | Warning | Refactor suggestion |
| FunctionTooLarge | Warning | Refactor suggestion |
| UnderscoreBandaid | Warning | Remove underscore prefix |
| LockedSetting | Error | Requires manual configuration update |

## Troubleshooting

### Extension not activating
- Ensure the project has a `Cargo.toml` file
- Check that `.rs` files are recognized as Rust

### Validation not running
1. Check Ferrous Forge is installed: `ferrous-forge --version`
2. Verify the path in settings: `ferrousForge.executablePath`
3. Check the output channel: **Ferrous Forge: Show Output Channel**

### High CPU usage
- Increase the debounce delay: `ferrousForge.debounceDelay` to 1000ms or higher
- Disable `ferrousForge.validateOnType` if needed

### Quick fixes not appearing
- Ensure `ferrousForge.enableQuickFixes` is `true`
- Some violations don't have automatic fixes and require manual changes

## Development

### Building from source

```bash
cd editors/vscode
npm install
npm run compile
```

### Running in development

1. Open this folder in VS Code
2. Press `F5` to open a new Extension Development Host window
3. Open a Rust project to test the extension

### Running tests

```bash
npm test
```

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/kryptobaseddev/ferrous-forge) for contribution guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Links

- [Ferrous Forge Repository](https://github.com/kryptobaseddev/ferrous-forge)
- [Issue Tracker](https://github.com/kryptobaseddev/ferrous-forge/issues)
- [Documentation](https://ferrous-forge.dev/docs)

---

**Task**: T022  
**Epic**: T014 - Aggressive Enforcement System  
**Part of**: Ferrous Forge - The Type-Safe Rust Development Standards Enforcer

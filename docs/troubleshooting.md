# Troubleshooting Guide

Common issues and solutions for Ferrous Forge.

## Installation Issues

### "Command not found: ferrous-forge"

**Problem**: After installation, the `ferrous-forge` command is not recognized.

**Solutions**:

1. Ensure `~/.cargo/bin` is in your PATH:
   ```bash
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

2. For fish shell:
   ```fish
   set -Ua fish_user_paths $HOME/.cargo/bin
   ```

3. Verify installation:
   ```bash
   ls -la ~/.cargo/bin/ferrous-forge
   cargo install --list | grep ferrous-forge
   ```

### "Permission denied" during installation

**Problem**: Can't install due to permission issues.

**Solutions**:
1. Never use `sudo` with cargo install
2. Ensure proper ownership:
   ```bash
   sudo chown -R $(whoami) ~/.cargo
   ```

3. Clear cargo cache if corrupted:
   ```bash
   rm -rf ~/.cargo/registry/cache
   cargo clean
   ```

## Initialization Problems

### "Failed to initialize: Config already exists"

**Problem**: Initialization fails because of existing configuration.

**Solution**:
```bash
# Force reinitialize
ferrous-forge init --force

# Or manually clean up
rm -rf ~/.config/ferrous-forge
ferrous-forge init
```

### Cargo wrapper not working

**Problem**: `cargo` commands don't trigger Ferrous Forge validation.

**Solutions**:
1. Check if wrapper is installed:
   ```bash
   ls -la ~/.local/bin/cargo
   which -a cargo
   ```

2. Verify PATH order:
   ```bash
   echo $PATH | tr ':' '\n'
   ```

3. Source shell configuration:
   ```bash
   source ~/.bashrc  # or ~/.zshrc
   ```

## Validation Issues

### "Validation fails but code compiles fine"

**Explanation**: Ferrous Forge enforces stricter standards than the compiler. This is intentional.

**Solutions**:
1. Review specific violations
2. Adjust code to meet standards
3. Configure exceptions in `.ferrous-forge.toml`:
   ```toml
   [ignore]
   patterns = ["tests/**", "benches/**"]
   ```

### "FileTooLarge" errors on generated files

**Solution**: Exclude generated files:
```toml
# .ferrous-forge.toml
[ignore]
files = [
    "src/generated.rs",
    "vendor/**",
    "target/**"
]
```

### False positives in macro-generated code

**Solutions**:
1. Use `#[allow()]` attributes:
   ```rust
   #[allow(clippy::unwrap_used)]
   macro_rules! my_macro {
       // macro implementation
   }
   ```

## Performance Issues

### Validation is slow

**Solutions**:
1. Exclude unnecessary directories:
   ```toml
   [validation]
   skip_dirs = ["target", "node_modules", ".git"]
   ```

2. Use quiet mode:
   ```bash
   ferrous-forge validate --quiet
   ```

## Configuration Issues

### Configuration not loading

**Debug steps**:
1. Check configuration:
   ```bash
   ferrous-forge config --list
   ```

2. Check file permissions
3. Verify working directory:
   ```bash
   pwd
   ferrous-forge config --sources
   ```

### Project-specific config not working

**Solutions**:
1. Ensure file is named `.ferrous-forge.toml`
2. Check file permissions
3. Verify location (must be in project root)

## Update Issues

### "Update available but can't install"

**Solutions**:
1. Manual update:
   ```bash
   cargo install ferrous-forge --force
   ```

2. Clear cargo cache:
   ```bash
   cargo cache -a
   cargo install ferrous-forge
   ```

### Rollback not working

**Solution**: Manually install specific version:
```bash
cargo install ferrous-forge --version 1.7.0 --force
```

## IDE Integration Issues

### VS Code not showing errors

**Solutions**:
1. Check rust-analyzer is running
2. Restart language server:
   - Press `Ctrl+Shift+P`
   - Run "Rust Analyzer: Restart server"

### IntelliJ/CLion integration not working

**Solutions**:
1. Check tool path is absolute
2. Enable in File Watchers
3. Check IDE event log

## Git Hook Issues

### Pre-commit hook not running

**Solutions**:
1. Check hook is executable:
   ```bash
   ls -la .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

2. Verify hook content:
   ```bash
   cat .git/hooks/pre-commit
   ```

3. Test hook manually:
   ```bash
   .git/hooks/pre-commit
   ```

### Hook causing commits to fail

**Solutions**:
1. Temporary bypass:
   ```bash
   git commit --no-verify
   ```

2. Fix issues then commit:
   ```bash
   ferrous-forge fix
   git add -A
   git commit
   ```

## Platform-Specific Issues

### Windows: "Cannot create symbolic link"

**Solutions**:
1. Run as Administrator
2. Enable Developer Mode in Windows Settings

### macOS: "Operation not permitted"

**Solutions**:
1. Grant Terminal full disk access:
   - System Preferences → Security & Privacy → Full Disk Access
   - Add Terminal/iTerm

### Linux: SELinux preventing execution

**Solutions**:
1. Check SELinux context:
   ```bash
   ls -Z ~/.cargo/bin/ferrous-forge
   ```

2. Fix context:
   ```bash
   restorecon -v ~/.cargo/bin/ferrous-forge
   ```

## Getting Help

If your issue isn't covered:

1. **Check FEATURES.md**: Verify the feature is implemented
2. **Check ROADMAP.md**: See if it's planned
3. **Search issues**: [GitHub Issues](https://github.com/kryptobaseddev/ferrous-forge/issues)
4. **Report bug** with:
   - Ferrous Forge version: `ferrous-forge --version`
   - Rust version: `rustc --version`
   - Operating system
   - Complete error message
   - Steps to reproduce

## Debug Mode

Enable debug output:

```bash
# Set environment variable
export RUST_LOG=debug
ferrous-forge validate

# Maximum verbosity
export RUST_LOG=trace
ferrous-forge validate -v
```

## Recovery Commands

If Ferrous Forge causes issues:

```bash
# Disable Ferrous Forge temporarily
export FERROUS_FORGE_ENABLED=0

# Remove all Ferrous Forge files
rm -rf ~/.config/ferrous-forge
rm -f ~/.local/bin/cargo
rm -f ~/.clippy.toml

# Uninstall completely
cargo uninstall ferrous-forge

# Reset shell configuration
grep -v "Ferrous Forge" ~/.bashrc > ~/.bashrc.new
mv ~/.bashrc.new ~/.bashrc
source ~/.bashrc
```

---

**Note:** Some solutions in older documentation may reference features not yet implemented. Always check [FEATURES.md](../FEATURES.md) for current capabilities.

# Troubleshooting Guide

## Common Issues and Solutions

### Installation Issues

#### "Command not found: ferrous-forge"

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

#### "Permission denied" during installation

**Problem**: Can't install due to permission issues.

**Solutions**:
1. Never use `sudo` with cargo install
2. Ensure proper ownership of cargo directory:
   ```bash
   sudo chown -R $(whoami) ~/.cargo
   ```

3. Clear cargo cache if corrupted:
   ```bash
   rm -rf ~/.cargo/registry/cache
   cargo clean
   ```

### Initialization Problems

#### "Failed to initialize: Config already exists"

**Problem**: Initialization fails because of existing configuration.

**Solution**: Use force flag or clean up first:
```bash
# Force reinitialize
ferrous-forge init --force

# Or manually clean up
rm -rf ~/.config/ferrous-forge
ferrous-forge init
```

#### Cargo wrapper not working

**Problem**: `cargo` commands don't trigger Ferrous Forge validation.

**Solutions**:
1. Check if wrapper is installed:
   ```bash
   ls -la ~/.local/bin/cargo
   which -a cargo
   ```

2. Verify PATH order (local bin should come first):
   ```bash
   echo $PATH | tr ':' '\n'
   ```

3. Manually source shell configuration:
   ```bash
   source ~/.bashrc  # or ~/.zshrc
   ```

### Validation Errors

#### "Validation fails but code compiles fine"

**Problem**: Ferrous Forge reports errors but Rust compiler accepts the code.

**Explanation**: Ferrous Forge enforces stricter standards than the compiler. This is intentional to maintain code quality.

**Solutions**:
1. Review the specific violations
2. Adjust code to meet standards
3. Or configure exceptions in `.ferrous-forge.toml`:
   ```toml
   [ignore]
   patterns = ["tests/**", "benches/**"]
   
   [standards]
   ban_unwrap = false  # Temporarily disable specific check
   ```

#### "FileTooLarge" errors on generated files

**Problem**: Generated or vendored files triggering size violations.

**Solution**: Exclude them from validation:
```toml
# .ferrous-forge.toml
[ignore]
files = [
    "src/generated.rs",
    "vendor/**",
    "target/**"
]
```

#### False positives in macro-generated code

**Problem**: Macros generating code that violates standards.

**Solutions**:
1. Use `#[allow()]` attributes locally:
   ```rust
   #[allow(clippy::unwrap_used)]
   macro_rules! my_macro {
       // macro implementation
   }
   ```

2. Configure macro exemptions:
   ```toml
   [standards]
   exempt_macros = true
   ```

### Performance Issues

#### Validation is very slow

**Problem**: Validation takes too long on large projects.

**Solutions**:
1. Use incremental validation:
   ```bash
   ferrous-forge validate --incremental
   ```

2. Exclude unnecessary directories:
   ```toml
   [validation]
   skip_dirs = ["target", "node_modules", ".git"]
   ```

3. Use quiet mode for CI/CD:
   ```bash
   ferrous-forge validate --quiet
   ```

4. Enable parallel validation:
   ```toml
   [performance]
   parallel = true
   threads = 4
   ```

### Configuration Issues

#### Configuration not loading

**Problem**: Custom configuration seems to be ignored.

**Debug steps**:
1. Check configuration location:
   ```bash
   ferrous-forge config --list
   ```

2. Validate TOML syntax:
   ```bash
   cat ~/.config/ferrous-forge/config.toml | toml-test
   ```

3. Check for typos in configuration:
   ```bash
   ferrous-forge config --validate
   ```

#### Project-specific config not working

**Problem**: `.ferrous-forge.toml` in project root is ignored.

**Solutions**:
1. Ensure file is named correctly (not `.ferrous_forge.toml`)
2. Check file permissions
3. Verify working directory:
   ```bash
   pwd
   ferrous-forge validate --show-config
   ```

### Update Issues

#### "Update available but can't install"

**Problem**: Update notification appears but update fails.

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

3. Check network/proxy settings:
   ```bash
   export CARGO_HTTP_PROXY=your_proxy
   cargo install ferrous-forge
   ```

#### Rollback not working

**Problem**: Can't rollback to previous version.

**Solution**: Manually install specific version:
```bash
cargo install ferrous-forge --version 0.1.0 --force
```

### IDE Integration Issues

#### VS Code not showing Ferrous Forge errors

**Problem**: Validation runs but errors don't appear in VS Code.

**Solutions**:
1. Check output format is compatible:
   ```json
   "rust-analyzer.checkOnSave.extraArgs": ["--format", "json"]
   ```

2. Verify rust-analyzer is running:
   ```bash
   # Check VS Code Output panel for "Rust Analyzer Language Server"
   ```

3. Restart language server:
   - Press `Ctrl+Shift+P`
   - Run "Rust Analyzer: Restart server"

#### IntelliJ/CLion integration not working

**Problem**: External tool configured but not triggering.

**Solutions**:
1. Check tool path is absolute:
   ```
   Program: /home/user/.cargo/bin/ferrous-forge
   ```

2. Enable in File Watchers
3. Check IDE event log for errors

### Git Hook Issues

#### Pre-commit hook not running

**Problem**: Git commits succeed without validation.

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

#### Hook causing commits to fail

**Problem**: All commits blocked by Ferrous Forge.

**Solutions**:
1. Temporary bypass:
   ```bash
   git commit --no-verify
   ```

2. Fix issues then commit:
   ```bash
   ferrous-forge validate --fix
   git add -A
   git commit
   ```

3. Adjust hook strictness in `.ferrous-forge.toml`:
   ```toml
   [hooks]
   strict = false
   allow_warnings = true
   ```

### Platform-Specific Issues

#### Windows: "Cannot create symbolic link"

**Problem**: Symbolic link creation fails on Windows.

**Solutions**:
1. Run as Administrator
2. Enable Developer Mode in Windows Settings
3. Use copy instead of symlink:
   ```toml
   [windows]
   use_symlinks = false
   ```

#### macOS: "Operation not permitted"

**Problem**: Security restrictions prevent file modifications.

**Solutions**:
1. Grant Terminal full disk access:
   - System Preferences → Security & Privacy → Full Disk Access
   - Add Terminal/iTerm

2. Use homebrew installation:
   ```bash
   brew install ferrous-forge
   ```

#### Linux: SELinux preventing execution

**Problem**: SELinux blocks Ferrous Forge execution.

**Solutions**:
1. Check SELinux context:
   ```bash
   ls -Z ~/.cargo/bin/ferrous-forge
   ```

2. Fix context:
   ```bash
   restorecon -v ~/.cargo/bin/ferrous-forge
   ```

3. Or create policy exception:
   ```bash
   sudo setsebool -P domain_can_mmap_files 1
   ```

## Getting Help

If your issue isn't covered here:

1. **Check the docs**: Review other documentation files
2. **Search issues**: [GitHub Issues](https://github.com/yourusername/ferrous-forge/issues)
3. **Ask the community**: [Discussions](https://github.com/yourusername/ferrous-forge/discussions)
4. **Report a bug**: Create a new issue with:
   - Ferrous Forge version: `ferrous-forge --version`
   - Rust version: `rustc --version`
   - Operating system
   - Complete error message
   - Steps to reproduce

## Debug Mode

Enable debug output for troubleshooting:

```bash
# Set environment variable
export FERROUS_FORGE_LOG=debug
ferrous-forge validate

# Or use flag
ferrous-forge validate --debug

# Maximum verbosity
RUST_LOG=trace ferrous-forge validate -vvv
```

## Recovery Commands

If Ferrous Forge causes issues, here are recovery commands:

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
# Contributing to Ferrous Forge

Thank you for your interest in contributing to Ferrous Forge! This document provides guidelines and information for contributors.

## 🤝 Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you agree to uphold this code.

## 🚀 Quick Start for Contributors

```bash
# 1. Fork and clone
git clone https://github.com/yourusername/ferrous-forge
cd ferrous-forge

# 2. Set up development environment
rustup update stable
rustup component add clippy rustfmt rust-analyzer

# 3. Install development dependencies
cargo install cargo-nextest cargo-audit cargo-outdated

# 4. Run tests to ensure everything works
cargo nextest run --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --check

# 5. Build documentation
cargo doc --open
```

## 📋 Development Standards

Ferrous Forge is built to the highest standards (naturally!):

### **Rust Requirements**
- **Edition 2024** (mandatory)
- **Rust 1.82+** minimum supported version
- **Zero unsafe code** (`#![forbid(unsafe_code)]`)
- **100% documented public APIs**
- **Zero clippy warnings** (--deny warnings)

### **Testing Requirements**
- **Unit tests** for all core logic
- **Integration tests** for CLI commands  
- **Property-based tests** for complex algorithms
- **Snapshot testing** for output validation
- **90%+ code coverage** target

### **Performance Requirements**
- **< 100ms startup time** for CLI
- **< 10MB memory usage** for typical operations
- **No unnecessary allocations** in hot paths
- **Benchmark all performance-critical code**

## 🏗️ Architecture Overview

```
ferrous-forge/
├── src/
│   ├── cli.rs              # Command line interface
│   ├── commands/           # Command implementations
│   │   ├── init.rs         # System initialization
│   │   ├── validate.rs     # Code validation
│   │   └── update.rs       # Self-update system
│   ├── config.rs          # Configuration management
│   ├── standards/         # Standards definitions
│   │   ├── clippy.rs      # Clippy rule sets
│   │   ├── edition.rs     # Edition enforcement
│   │   └── templates.rs   # Project templates
│   ├── validation/        # Code analysis
│   └── updater.rs         # Version management
├── tests/                 # Integration tests
├── benches/              # Performance benchmarks
└── docs/                 # Documentation
```

## 🛠️ Development Workflow

### **1. Branch Naming**
- `feature/description` - New features
- `fix/description` - Bug fixes  
- `docs/description` - Documentation
- `perf/description` - Performance improvements

### **2. Commit Messages**
Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new validation rule for underscore parameters
fix: resolve panic when Cargo.toml is malformed
docs: update installation instructions
perf: optimize template rendering performance
test: add property tests for config parsing
```

### **3. Pull Request Process**

1. **Create feature branch** from `main`
2. **Implement changes** following our standards
3. **Add/update tests** for your changes
4. **Update documentation** if needed
5. **Run full test suite**:
   ```bash
   cargo nextest run --all-features
   cargo clippy --all-features -- -D warnings
   cargo fmt --check
   cargo doc --all-features
   ```
6. **Create pull request** with clear description
7. **Address review feedback**
8. **Squash commits** before merge

### **4. Review Criteria**

Pull requests are reviewed for:
- ✅ **Correctness** - Does it work as intended?
- ✅ **Testing** - Are there adequate tests?
- ✅ **Documentation** - Is it properly documented?
- ✅ **Performance** - Does it meet performance requirements?
- ✅ **Standards** - Does it follow our coding standards?
- ✅ **Security** - Are there any security implications?

## 📝 Adding New Standards

When adding new Rust standards:

### **1. Create Standard Definition**
```rust
// src/standards/new_standard.rs
use crate::validation::Rule;

pub struct NewStandard;

impl Rule for NewStandard {
    fn name(&self) -> &'static str {
        "new_standard"
    }
    
    fn description(&self) -> &'static str {
        "Description of what this standard enforces"
    }
    
    fn validate(&self, project: &Project) -> Result<Vec<Violation>> {
        // Implementation
    }
}
```

### **2. Add Tests**
```rust
// tests/standards/test_new_standard.rs
use ferrous_forge::standards::NewStandard;

#[test]
fn test_new_standard_detection() {
    // Test cases for the new standard
}
```

### **3. Update Documentation**
- Add to `docs/standards.md`
- Update examples in README
- Add migration notes if breaking

### **4. Consider Backwards Compatibility**
- Can existing projects still build?
- Should this be opt-in initially?
- What's the migration path?

## 🧪 Testing Guidelines

### **Unit Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_specific_functionality() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

### **Integration Tests**
```rust
// tests/cli_integration.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_init_command() {
    let mut cmd = Command::cargo_bin("ferrous-forge").unwrap();
    cmd.arg("init")
       .assert()
       .success()
       .stdout(predicate::str::contains("Ferrous Forge initialized"));
}
```

### **Property Tests**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_config_parsing_roundtrip(config in any::<Config>()) {
        let serialized = config.to_string();
        let deserialized = Config::from_str(&serialized)?;
        prop_assert_eq!(config, deserialized);
    }
}
```

## 📚 Documentation Guidelines

### **Code Documentation**
- **All public APIs** must have doc comments
- **Include examples** for complex functions
- **Document errors** that can be returned
- **Link to related functions/types**

```rust
/// Validates a Rust project against Ferrous Forge standards.
///
/// This function performs comprehensive validation including:
/// - Edition 2024 compliance
/// - Clippy rule adherence  
/// - File size limitations
/// - Documentation coverage
///
/// # Arguments
///
/// * `project_path` - Path to the Rust project root
/// * `config` - Validation configuration
///
/// # Returns
///
/// Returns `Ok(Report)` with validation results, or `Err` if
/// the project cannot be analyzed.
///
/// # Examples
///
/// ```rust
/// use ferrous_forge::{validate_project, Config};
/// 
/// let config = Config::default();
/// let report = validate_project("./my-project", &config)?;
/// println!("Found {} violations", report.violations.len());
/// ```
///
/// # Errors
///
/// This function returns an error if:
/// - The project path doesn't exist
/// - Cargo.toml is malformed
/// - File system permissions deny access
pub fn validate_project(project_path: &Path, config: &Config) -> Result<Report> {
    // Implementation
}
```

### **User Documentation**
- Write for **beginners to Rust**
- Include **complete examples**
- Provide **troubleshooting sections**
- Keep **up to date** with code changes

## 🔄 Release Process

### **Version Numbers**
Follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- `MAJOR` - Breaking changes
- `MINOR` - New features (backwards compatible)
- `PATCH` - Bug fixes

### **Release Checklist**
- [ ] Update `Cargo.toml` version
- [ ] Update `CHANGELOG.md`
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Create git tag
- [ ] Publish to crates.io
- [ ] Create GitHub release
- [ ] Update homebrew formula (if applicable)

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment Information**:
   - Ferrous Forge version (`ferrous-forge --version`)
   - Rust version (`rustc --version`)
   - Operating system
   - Shell (bash/zsh/etc.)

2. **Reproduction Steps**:
   - Minimal example that reproduces the issue
   - Exact commands run
   - Expected vs actual behavior

3. **Additional Context**:
   - Error messages (full output)
   - Relevant configuration files
   - Screenshots if applicable

## 💡 Feature Requests

For feature requests:

1. **Check existing issues** first
2. **Describe the problem** you're trying to solve
3. **Propose a solution** if you have ideas
4. **Consider backwards compatibility**
5. **Discuss implementation approach**

## 🏷️ Issue Labels

We use these labels to organize issues:

- `bug` - Something isn't working
- `enhancement` - New feature or improvement  
- `documentation` - Improvements to docs
- `good first issue` - Good for newcomers
- `help wanted` - Community help needed
- `performance` - Performance improvements
- `security` - Security-related issues
- `breaking change` - Would require major version bump

## 🎯 Areas Needing Help

We especially welcome contributions in:

- **Windows support** - Testing and platform-specific features
- **IDE integrations** - VS Code, IntelliJ, Vim plugins
- **Performance optimization** - Making validation faster
- **New standards** - Additional Rust best practices
- **Documentation** - Examples, tutorials, guides
- **Internationalization** - Multi-language support

## 📞 Getting Help

- **GitHub Discussions** - General questions and ideas
- **Discord** - Real-time chat with maintainers
- **Issues** - Bug reports and feature requests
- **Email** - maintainers@ferrous-forge.dev for private matters

## 🙏 Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Project documentation
- Annual contributor reports

Thank you for helping make Rust development better for everyone! 🦀✨
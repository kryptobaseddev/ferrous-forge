# Contributing to Ferrous Forge

Thank you for your interest in contributing to Ferrous Forge! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our code of conduct:
- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Accept feedback gracefully

## How to Contribute

### Reporting Issues

Before reporting an issue:
1. Check existing issues to avoid duplicates
2. Try to reproduce the issue with the latest version
3. Collect relevant information (OS, Rust version, error messages)

When reporting:
- Use a clear, descriptive title
- Provide steps to reproduce
- Include expected vs actual behavior
- Add relevant logs or screenshots

### Suggesting Features

Feature requests are welcome! Please:
- Check if the feature has already been requested
- Explain the use case and benefits
- Consider if it aligns with project goals
- Be open to discussion and alternatives

### Contributing Code

#### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/ferrous-forge
cd ferrous-forge

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run tests
cargo test

# Run with verbose output for development
cargo run -- --verbose
```

#### Development Workflow

1. **Fork the repository** on GitHub
2. **Create a feature branch**: `git checkout -b feature-name`
3. **Make your changes** following our coding standards
4. **Test your changes**: `cargo test`
5. **Check formatting**: `cargo fmt`
6. **Run clippy**: `cargo clippy`
7. **Commit with clear message**: `git commit -m "feat: add new feature"`
8. **Push to your fork**: `git push origin feature-name`
9. **Create a Pull Request** on GitHub

#### Coding Standards

This project uses Ferrous Forge's own standards:
- Rust Edition 2024
- No underscore parameters
- No unwrap() in production code
- All public APIs must be documented
- Maximum 300 lines per file
- Maximum 50 lines per function

Run validation before committing:
```bash
cargo run -- validate .
```

#### Commit Message Format

Follow conventional commits:
```
type: brief description

Longer explanation if needed.

Fixes #issue-number
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions or changes
- `chore`: Maintenance tasks

### Testing

#### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

#### Writing Tests

- Write unit tests for new functions
- Add integration tests for new features
- Use property-based testing where appropriate
- Ensure tests are deterministic

Example test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = "test";
        
        // Act
        let result = process(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Documentation

#### Code Documentation

All public items must be documented:
```rust
/// Processes input according to standards.
///
/// # Arguments
/// * `input` - The data to process
///
/// # Returns
/// Processed result or error
///
/// # Examples
/// ```
/// let result = process("data")?;
/// ```
pub fn process(input: &str) -> Result<String> {
    // Implementation
}
```

#### Documentation Updates

When adding features:
1. Update relevant .md files in docs/
2. Add examples to documentation
3. Update README if needed
4. Ensure `cargo doc` builds without warnings

### Pull Request Process

1. **Ensure CI passes**: All checks must be green
2. **Update documentation**: If behavior changes
3. **Add tests**: For new functionality
4. **Request review**: From maintainers
5. **Address feedback**: Make requested changes
6. **Squash commits**: If requested by maintainers

### Release Process

Maintainers handle releases:
1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Create git tag
4. Publish to crates.io
5. Create GitHub release

## Getting Help

- Check documentation in docs/
- Look at existing issues
- Ask in discussions
- Reach out to maintainers

## Recognition

Contributors are recognized in:
- GitHub contributors page
- Release notes for significant contributions
- CONTRIBUTORS.md file (for regular contributors)

Thank you for contributing to Ferrous Forge!

# Standards Reference

## Overview

This document details all the coding standards that Ferrous Forge enforces. Standards are categorized by type and severity.

## Edition Requirements

### Rust Edition
- **Required**: Edition 2024
- **Minimum Rust Version**: 1.85.0
- **Action**: Projects using older editions will receive validation warnings

## Code Quality Standards

### File Size Limits
- **Maximum lines per file**: 300 lines
- **Maximum line length**: 100 characters
- **Rationale**: Smaller files are easier to understand and maintain

### Function Size Limits
- **Maximum lines per function**: 50 lines
- **Maximum parameters**: 7 parameters
- **Maximum cyclomatic complexity**: 10
- **Rationale**: Smaller functions are easier to test and reason about

## Banned Patterns

### Underscore Bandaid Patterns

#### Underscore Parameters
```rust
// ❌ Banned
fn process(_unused: String) {
    // ...
}

// ✅ Preferred
fn process(#[allow(unused)] data: String) {
    // ...
}
// Or remove the parameter if truly unused
```

#### Underscore Let Bindings
```rust
// ❌ Banned
let _ = important_result();

// ✅ Preferred
let _result = important_result(); // If you need to ignore
// Or handle the result properly
if let Err(e) = important_result() {
    eprintln!("Error: {}", e);
}
```

### Error Handling Patterns

#### Unwrap Usage
```rust
// ❌ Banned in production code
let value = some_option.unwrap();
let result = some_result.unwrap();

// ✅ Preferred
let value = some_option.expect("Clear error message");
// Or better:
let value = some_option.ok_or_else(|| CustomError::Missing)?;
```

#### Expect Usage
```rust
// ⚠️ Discouraged (warning)
let value = some_option.expect("message");

// ✅ Preferred
let value = some_option.ok_or_else(|| CustomError::new("message"))?;
```

### Panic Patterns

```rust
// ❌ Banned in production
panic!("Something went wrong");
todo!("Implement this");
unimplemented!("Not yet done");

// ✅ Preferred
return Err(Error::new("Something went wrong"));
// Or for truly unrecoverable errors with good context:
panic!("FATAL: Database connection lost after {} retries", MAX_RETRIES);
```

## Documentation Standards

### Public API Documentation
- **Requirement**: All public items must have documentation
- **Minimum**: One-line summary
- **Preferred**: Summary + detailed description + examples

```rust
// ❌ Missing documentation
pub fn process(data: &str) -> Result<String> {
    // ...
}

// ✅ Properly documented
/// Processes the input data and returns the transformed result.
///
/// # Arguments
/// * `data` - The input string to process
///
/// # Returns
/// The processed string or an error if processing fails
///
/// # Examples
/// ```
/// let result = process("hello")?;
/// assert_eq!(result, "HELLO");
/// ```
pub fn process(data: &str) -> Result<String> {
    // ...
}
```

## Clippy Lints

Ferrous Forge enforces 280+ clippy lints organized into categories:

### Denied Lints (Compilation Errors)
- `clippy::unwrap_used`
- `clippy::expect_used`
- `clippy::panic`
- `clippy::unimplemented`
- `clippy::todo`

### Warned Lints (Must Fix)
- All `clippy::pedantic` lints
- All `clippy::nursery` lints
- All `clippy::cargo` lints

### Allowed Exceptions
- `clippy::module_name_repetitions`
- `clippy::missing_errors_doc`

## Project Structure Standards

### Required Files
Every Rust project should have:
- `Cargo.toml` with complete metadata
- `README.md` with project description
- `LICENSE` file (MIT or Apache-2.0)
- `.gitignore` with proper exclusions

### Cargo.toml Requirements
```toml
[package]
name = "project-name"
version = "0.1.0"
edition = "2024"        # Required
rust-version = "1.85"   # Required

# Complete metadata required
authors = ["Your Name <email@example.com>"]
description = "Brief description"
repository = "https://github.com/..."
license = "MIT OR Apache-2.0"
keywords = ["rust", "..."]
categories = ["..."]

[lints.rust]
unsafe_code = "forbid"  # Recommended
missing_docs = "warn"    # Recommended

[lints.clippy]
unwrap_used = "deny"    # Required
expect_used = "deny"    # Required
```

## Testing Standards

### Test Coverage Requirements (Planned)
- **Minimum coverage**: 80% for libraries
- **Minimum coverage**: 60% for binaries
- **Required**: Unit tests for all public functions
- **Required**: Integration tests for main functionality

### Test Organization
```
tests/
├── unit/           # Unit tests
├── integration/    # Integration tests
└── fixtures/       # Test data
```

## Security Standards

### Dependency Security (Planned)
- Automatic scanning with `cargo audit`
- No known vulnerabilities in dependencies
- Regular dependency updates

### Safe Rust Requirement
```rust
// Required in lib.rs or main.rs
#![forbid(unsafe_code)]
```

## Performance Standards

### Compilation Settings
Release builds should use:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

## Version Control Standards

### Git Hooks (Planned)
Pre-commit hooks will validate:
1. Code formatting (`cargo fmt`)
2. Linting (`cargo clippy`)
3. Tests passing (`cargo test`)
4. Documentation building (`cargo doc`)

### Commit Message Format
Recommended format:
```
type: Brief description

Longer explanation if needed

Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Severity Levels

### Error (Must Fix)
- Wrong Rust edition
- Underscore bandaid patterns
- Use of `unwrap()` in production

### Warning (Should Fix)
- Missing documentation
- Function too large
- High cyclomatic complexity

### Info (Good to Know)
- Suggestions for better patterns
- Performance improvement opportunities
- Style recommendations
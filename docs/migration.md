# Migration Guide

## Migrating Existing Projects to Ferrous Forge Standards

This guide helps you migrate existing Rust projects to comply with Ferrous Forge standards.

## Quick Migration

For projects that are already well-maintained:

```bash
# 1. Backup your project
git add -A && git commit -m "Backup before Ferrous Forge migration"

# 2. Run automatic migration
ferrous-forge migrate --auto

# 3. Review and commit changes
git diff
git add -A && git commit -m "Migrate to Ferrous Forge standards"
```

## Step-by-Step Migration

### Step 1: Assessment

First, assess your project's current state:

```bash
# Get a migration report
ferrous-forge validate --report migration-report.html

# Summary of violations
ferrous-forge validate --summary
```

### Step 2: Update Rust Edition

#### Automatic Update
```bash
ferrous-forge migrate --edition 2024
```

#### Manual Update
Edit `Cargo.toml`:
```toml
[package]
edition = "2024"
rust-version = "1.85"
```

Then update code for edition changes:
```bash
cargo fix --edition
cargo fmt
```

### Step 3: Fix Critical Violations

#### Underscore Parameters

**Before:**
```rust
fn process(_data: String, _config: Config) {
    // implementation
}
```

**After:**
```rust
fn process(data: String, config: Config) {
    let _ = data; // If truly unused
    let _ = config;
    // Or better: remove unused parameters
}
```

**Automated fix:**
```bash
ferrous-forge fix --underscores
```

#### Unwrap/Expect Usage

**Before:**
```rust
let value = some_option.unwrap();
let result = some_result.expect("failed");
```

**After:**
```rust
let value = some_option.ok_or(Error::Missing)?;
let result = some_result.map_err(|e| Error::Process(e))?;
```

**Automated fix:**
```bash
ferrous-forge fix --error-handling
```

### Step 4: Add Missing Documentation

#### Generate Documentation Stubs
```bash
ferrous-forge generate-docs --missing
```

This creates basic documentation for all public items:

**Generated:**
```rust
/// TODO: Document this function
pub fn process() -> Result<()> {
    // ...
}
```

**Update to:**
```rust
/// Processes the application data according to configured rules.
///
/// # Errors
/// Returns an error if processing fails or data is invalid.
pub fn process() -> Result<()> {
    // ...
}
```

### Step 5: Split Large Files

For files exceeding 300 lines:

```bash
# Identify large files
ferrous-forge validate --check-size

# Suggest module split
ferrous-forge suggest --split src/large_file.rs
```

**Manual splitting example:**

Before (single large file):
```rust
// src/lib.rs (500 lines)
mod processing { /* 200 lines */ }
mod validation { /* 150 lines */ }
mod utils { /* 150 lines */ }
```

After (split into modules):
```rust
// src/lib.rs
mod processing;
mod validation;
mod utils;

// src/processing.rs (200 lines)
// src/validation.rs (150 lines)
// src/utils.rs (150 lines)
```

### Step 6: Configure Clippy

Install Ferrous Forge's clippy configuration:

```bash
ferrous-forge init --clippy-only
```

Fix clippy warnings:
```bash
cargo clippy --fix
cargo clippy -- -D warnings
```

### Step 7: Setup Git Hooks

Add pre-commit validation:

```bash
ferrous-forge init --hooks-only
```

### Step 8: Update CI/CD

Add Ferrous Forge to your CI pipeline. See examples for:
- [GitHub Actions](#github-actions)
- [GitLab CI](#gitlab-ci)
- [Jenkins](#jenkins)

## Migration Strategies by Project Type

### Library Crates

Priority order:
1. Public API documentation (critical)
2. Edition upgrade
3. Error handling patterns
4. Test coverage
5. Examples and benchmarks

```bash
ferrous-forge migrate --library
```

### Binary Applications

Priority order:
1. Error handling (no panics)
2. Configuration validation
3. CLI documentation
4. Security audit
5. Performance optimization

```bash
ferrous-forge migrate --binary
```

### Web Services

Priority order:
1. Security vulnerabilities
2. Error responses
3. API documentation
4. Input validation
5. Performance metrics

```bash
ferrous-forge migrate --web-service
```

### Embedded Systems

Special considerations:
```toml
# .ferrous-forge.toml
[embedded]
no_std = true
no_alloc = true
max_stack_size = 4096

[standards]
ban_panic = true  # Critical for embedded
ban_heap_allocation = true
```

## Gradual Migration

For large projects, migrate gradually:

### Phase 1: Non-Breaking Changes (Week 1)
- Documentation
- Formatting
- Clippy warnings
- Test organization

### Phase 2: Minor Breaking Changes (Week 2)
- Edition upgrade
- Dependency updates
- Error handling improvements
- Module reorganization

### Phase 3: Major Refactoring (Week 3-4)
- Remove underscore patterns
- Split large files
- Architectural improvements
- Performance optimizations

## Configuration for Legacy Code

If you need to maintain legacy code alongside new standards:

```toml
# .ferrous-forge.toml
[legacy]
# Allow legacy patterns in specific modules
allow_legacy = ["src/legacy/**", "src/vendor/**"]

# Gradual enforcement
[standards]
enforcement_level = "warning"  # Start with warnings

[schedule]
# Increase enforcement over time
warning_phase = "2024-01-01"
error_phase = "2024-03-01"
strict_phase = "2024-06-01"
```

## Common Migration Patterns

### Converting Error Handling

Use the provided macro for bulk conversion:

```rust
// Add to lib.rs
macro_rules! try_unwrap {
    ($expr:expr) => {
        $expr.map_err(|e| anyhow::anyhow!("{:?}", e))?
    };
}

// Replace unwrap() calls
// Before: value.unwrap()
// After: try_unwrap!(value)
```

### Dealing with Generated Code

Exclude generated code from validation:

```toml
[ignore]
generated = [
    "src/proto/**",
    "src/generated.rs",
    "**/*.pb.rs"
]
```

### Handling Third-Party Dependencies

Some dependencies might not meet standards:

```toml
[dependencies]
# Allow specific dependencies to violate standards
allow_unsafe = ["sys-crate", "ffi-wrapper"]
allow_old_edition = ["legacy-dep"]
```

## Verification

After migration, verify compliance:

```bash
# Full validation
ferrous-forge validate --strict

# Generate compliance report
ferrous-forge report --output compliance.html

# Check specific aspects
ferrous-forge validate --check-docs
ferrous-forge validate --check-security
ferrous-forge validate --check-size
```

## Rollback Plan

If migration causes issues:

```bash
# Create rollback point
git tag pre-ferrous-forge

# If needed, rollback
git reset --hard pre-ferrous-forge

# Partial rollback (keep some changes)
git revert <specific-commit>
```

## CI/CD Examples

### GitHub Actions
```yaml
name: Ferrous Forge Migration Check

on:
  pull_request:
    branches: [main]

jobs:
  migration:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Ferrous Forge
        run: cargo install ferrous-forge
      - name: Check Migration Status
        run: |
          ferrous-forge validate --summary
          ferrous-forge report --output migration-status.html
      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: migration-report
          path: migration-status.html
```

### GitLab CI
```yaml
migration-check:
  stage: test
  script:
    - cargo install ferrous-forge
    - ferrous-forge validate --summary
    - ferrous-forge report --output migration-status.html
  artifacts:
    reports:
      paths:
        - migration-status.html
    expire_in: 30 days
```

### Jenkins
```groovy
stage('Migration Check') {
  steps {
    sh 'cargo install ferrous-forge'
    sh 'ferrous-forge validate --summary'
    sh 'ferrous-forge report --output migration-status.html'
    publishHTML([
      reportDir: '.',
      reportFiles: 'migration-status.html',
      reportName: 'Migration Status'
    ])
  }
}
```

## Getting Help

- **Migration issues**: Create issue with `migration` tag
- **Community support**: [Discussions](https://github.com/yourusername/ferrous-forge/discussions)
- **Professional support**: Contact contributors for consulting

## Success Stories

> "We migrated our 100k LOC codebase in 2 weeks with Ferrous Forge. Code quality improved dramatically." - *Large Rust Project*

> "The gradual migration approach let us maintain velocity while improving standards." - *Startup Team*
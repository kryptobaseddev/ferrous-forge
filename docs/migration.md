# Migration Guide

Migrating existing Rust projects to comply with Ferrous Forge standards.

## Quick Start

For well-maintained projects:

```bash
# 1. Check current state
ferrous-forge validate .

# 2. Fix what can be auto-fixed
ferrous-forge fix

# 3. Check edition compliance
ferrous-forge edition check

# 4. Migrate edition if needed
ferrous-forge edition migrate  # Defaults to 2024

# 5. Re-validate
ferrous-forge validate .
```

## Step-by-Step Migration

### Step 1: Assessment

Check your project's current state:

```bash
# Run full validation
ferrous-forge validate .

# Generate AI report for detailed analysis
ferrous-forge validate . --ai-report
```

### Step 2: Auto-Fix

Apply safe automatic fixes:

```bash
# Preview fixes without applying
ferrous-forge fix --dry-run

# Apply fixes
ferrous-forge fix

# With AI analysis for complex violations
ferrous-forge fix --ai-analysis
```

### Step 3: Edition Migration

Check and migrate Rust edition:

```bash
# Check current edition
ferrous-forge edition check

# Analyze compatibility before migrating
ferrous-forge edition analyze

# Migrate to Edition 2024
ferrous-forge edition migrate

# Options:
ferrous-forge edition migrate 2024 --test --idioms
```

**Note:** Migration runs `cargo fix --edition` and makes backups automatically.

### Step 4: Fix Critical Violations Manually

#### Underscore Parameters

Change this:
```rust
fn process(_data: String, _config: Config) {
    // ...
}
```

To this:
```rust
fn process(data: String, config: Config) {
    let _ = data; // If truly unused
    let _ = config;
    // Or better: remove unused parameters
}
```

#### Error Handling

Change this:
```rust
let value = some_option.unwrap();
let result = some_result.expect("failed");
```

To this:
```rust
let value = some_option.ok_or(Error::Missing)?;
let result = some_result.map_err(|e| Error::Process(e))?;
```

### Step 5: Split Large Files

For files over 300 lines, split into modules:

```rust
// Before: src/lib.rs (500 lines)
mod processing { /* 200 lines */ }
mod validation { /* 150 lines */ }
mod utils { /* 150 lines */ }

// After: Split into files
// src/lib.rs
mod processing;
mod validation;
mod utils;

// src/processing.rs (200 lines)
// src/validation.rs (150 lines)
// src/utils.rs (150 lines)
```

### Step 6: Setup Project Tooling

Initialize Ferrous Forge for the project:

```bash
ferrous-forge init --project
```

This creates:
- `.rustfmt.toml` - Formatting rules
- `.clippy.toml` - Lint configuration
- Updates `Cargo.toml` with `[lints]` section
- `.vscode/settings.json` - VS Code integration
- `.github/workflows/ci.yml` - CI template
- Git hooks for pre-commit validation

### Step 7: Install Git Hooks

Set up automatic validation on commits:

```bash
ferrous-forge safety install
```

## Migration by Project Type

### Library Crates

Priority order:
1. Public API documentation
2. Edition upgrade
3. Error handling patterns
4. Test coverage
5. Examples and benchmarks

### Binary Applications

Priority order:
1. Error handling (no panics)
2. CLI documentation
3. Security audit
4. Performance optimization

### Web Services

Priority order:
1. Security vulnerabilities
2. Error responses
3. API documentation
4. Input validation

### Embedded Systems

Configure for `no_std`:

```toml
# .ferrous-forge.toml
[standards]
ban_panic = true
max_file_lines = 400  # May need larger files for embedded
```

## Gradual Migration

For large codebases, migrate in phases:

**Week 1:** Non-breaking changes
- Documentation
- Formatting (`cargo fmt`)
- Clippy warnings

**Week 2:** Minor changes
- Edition upgrade
- Dependency updates
- Error handling improvements

**Week 3-4:** Major refactoring
- Remove underscore patterns
- Split large files
- Architectural improvements

## Ignoring Files

Exclude generated or legacy code:

```toml
# .ferrous-forge.toml
[ignore]
patterns = [
    "tests/fixtures/**",
    "src/generated/**",
    "vendor/**"
]
```

## Verification

After migration:

```bash
# Full validation
ferrous-forge validate .

# Check specific areas
ferrous-forge edition check
ferrous-forge rust check

# Run safety checks
ferrous-forge safety check
```

## Rollback

If migration causes issues:

```bash
# Create rollback point before starting
git add -A && git commit -m "Pre-migration backup"
git tag pre-ferrous-forge

# If needed, rollback
git reset --hard pre-ferrous-forge

# Or revert specific commits
git revert <commit-hash>
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install Ferrous Forge
        run: cargo install ferrous-forge
      
      - name: Validate
        run: ferrous-forge validate .
```

### GitLab CI

```yaml
validate:
  stage: test
  image: rust:latest
  before_script:
    - cargo install ferrous-forge
  script:
    - ferrous-forge validate .
```

## Getting Help

- Create issue: [github.com/kryptobaseddev/ferrous-forge/issues](https://github.com/kryptobaseddev/ferrous-forge/issues)
- Discussions: [github.com/kryptobaseddev/ferrous-forge/discussions](https://github.com/kryptobaseddev/ferrous-forge/discussions)

---

**Coming in v1.8:**
- `ferrous-forge validate --report` - Generate HTML reports
- `ferrous-forge validate --summary` - Summary view
- `ferrous-forge migrate --auto` - Automatic migration
- `ferrous-forge fix --underscores` - Fix specific violation types

# Ferrous Forge Enhanced Configuration System
## Technical Specification v2.0

---

## Overview

The Ferrous Forge configuration system provides a hierarchical, shareable configuration mechanism that extends and enhances Cargo.toml capabilities. It enables project templates, dependency management, and Rust ecosystem configuration through a central, community-shareable format.

---

## Configuration Hierarchy

### 1. System Level Configuration
**Location**: `~/.config/ferrous-forge/config.toml`
**Priority**: Lowest
**Scope**: Global defaults for all projects

### 2. User Level Configuration  
**Location**: `~/.ferrous-forge/config.toml`
**Priority**: Medium
**Scope**: User-specific overrides

### 3. Project Level Configuration
**Location**: `<project_root>/ferrous-forge.toml`
**Priority**: Highest
**Scope**: Project-specific settings

### 4. Workspace Level Configuration
**Location**: `<workspace_root>/ferrous-forge.toml`
**Priority**: High (lower than project)
**Scope**: Workspace-wide settings

---

## Complete Configuration Schema

```toml
# Ferrous Forge Configuration Schema v2.0
# This file can exist at system, user, project, or workspace level

[metadata]
version = "2.0.0"                    # Config schema version
name = "my-config"                   # Optional config name
description = "My shared config"     # Optional description
author = "dev@example.com"          # Optional author
license = "MIT OR Apache-2.0"       # Optional license for shared configs
repository = "https://github.com/..." # Optional source repository

[rust]
# Rust toolchain configuration
preferred_channel = "stable"         # stable/beta/nightly
minimum_version = "1.85.0"          # Minimum supported Rust version
maximum_version = "1.95.0"          # Maximum tested version (optional)
default_edition = "2024"            # Default edition for new projects
toolchain_components = [            # Required rustup components
  "rustfmt",
  "clippy",
  "rust-analyzer"
]

[rust.channels]
# Channel-specific settings
stable = { version = "1.90.0", date = "2025-08-07" }
beta = { version = "1.91.0-beta.1" }
nightly = { version = "nightly-2025-09-19", features = ["const_generics"] }

[version_management]
auto_check = true                    # Auto-check for Rust updates
check_interval = "24h"              # How often to check (1h, 24h, 7d, 30d)
notify_security_updates = true      # Alert on security releases
notify_breaking_changes = true      # Alert on breaking changes
cache_ttl = "24h"                   # GitHub API cache duration

[edition]
enforce_latest = true               # Require latest stable edition
allow_migration_prompts = true      # Interactive migration prompts
backup_before_migration = true      # Create backup before migrating
migration_strategy = "incremental"  # all-at-once/incremental/manual
target_edition = "2024"             # Target edition for migrations

[edition.migration]
# Edition migration settings
fix_edition_idioms = true          # Apply edition idioms during migration
fix_all_targets = true             # Fix all targets (tests, benches, etc.)
allow_dirty = false                # Allow migration with uncommitted changes
allow_staged = false               # Allow migration with staged changes

[templates]
# Template system configuration
default_template = "standard"       # Default template for new projects
repository = "https://github.com/ferrous-forge/templates"
auto_fetch = true                  # Auto-fetch template updates
cache_dir = "~/.cache/ferrous-forge/templates"
enable_community = true            # Enable community templates

[templates.sources]
# Additional template sources
official = "https://github.com/ferrous-forge/templates"
community = "https://github.com/ferrous-forge/community-templates"
custom = ["https://github.com/myorg/templates"]

[templates.variables]
# Default template variables
author = "Your Name"
email = "you@example.com"
github_username = "yourname"
organization = "yourorg"

[project]
# Project creation defaults
vcs = "git"                        # Version control system
init_commit = true                 # Create initial commit
create_readme = true               # Generate README.md
create_changelog = true            # Generate CHANGELOG.md
create_license = true              # Add license file
license_type = "MIT OR Apache-2.0" # Default license

[project.structure]
# Project structure preferences
src_layout = "standard"            # standard/nested/flat
test_layout = "integrated"         # integrated/separate
bench_layout = "benches"          # benches/criterion
example_layout = "examples"        # examples/demos

[workspace]
# Workspace configuration
resolver = "2"                     # Workspace resolver version
default_members = []              # Default workspace members
exclude = ["target", "*.backup"]  # Exclude patterns

[dependencies]
# Default dependencies for new projects
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"], optional = true }
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"

[dependencies.groups]
# Dependency groups for templates
web = [
  "axum:0.7",
  "tower:0.4",
  "tower-http:0.5"
]
cli = [
  "clap:4.0",
  "colored:2.0",
  "indicatif:0.17"
]
async = [
  "tokio:1",
  "async-trait:0.1",
  "futures:0.3"
]

[dev-dependencies]
# Default dev dependencies
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.0"
quickcheck = "1.0"
mockall = "0.13"
pretty_assertions = "1.4"

[build-dependencies]
# Default build dependencies
cc = { version = "1.0", optional = true }
bindgen = { version = "0.70", optional = true }

[profile.dev]
# Development profile settings
opt-level = 0
debug = 2
debug-assertions = true
overflow-checks = true
lto = false
panic = 'unwind'
incremental = true
codegen-units = 256
rpath = false

[profile.release]
# Release profile settings
opt-level = 3
debug = 0
debug-assertions = false
overflow-checks = false
lto = true
panic = 'abort'
incremental = false
codegen-units = 1
rpath = false
strip = true

[profile.bench]
# Benchmark profile
inherits = "release"
debug = true

[lints]
# Linting configuration
enable_all = false                 # Enable all lints by default
pedantic = true                    # Enable pedantic lints
style = "aggressive"              # conservative/moderate/aggressive

[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
missing_debug_implementations = "warn"
unused_results = "warn"
rust_2024_compatibility = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "allow"
cargo = "warn"
perf = "warn"
style = "warn"
complexity = "warn"
correctness = "deny"
suspicious = "deny"

[lints.clippy.allow]
# Explicitly allowed clippy lints
module_name_repetitions = true
must_use_candidate = true

[lints.rustdoc]
broken_intra_doc_links = "deny"
missing_crate_level_docs = "warn"
missing_doc_code_examples = "warn"
private_doc_tests = "warn"

[formatting]
# Code formatting preferences
edition = "2024"
hard_tabs = false
tab_spaces = 4
max_width = 100
newline_style = "Unix"
use_field_init_shorthand = true
use_try_shorthand = true
format_strings = true
format_macro_matchers = true
format_macro_bodies = true

[testing]
# Testing configuration
default_test_threads = 0          # 0 = auto-detect
test_timeout = "60s"
enable_coverage = true
coverage_threshold = 80           # Minimum coverage percentage
enable_miri = false               # Enable Miri for unsafe code
enable_loom = false              # Enable Loom for concurrency testing

[documentation]
# Documentation settings
require_examples = true           # Require examples in docs
require_safety_docs = true        # Require # Safety sections
min_coverage = 80                # Minimum doc coverage percentage
build_docs_on_save = true        # Auto-build docs
open_docs_after_build = false    # Open in browser after building

[security]
# Security settings
audit_on_install = true          # Run cargo-audit on install
audit_on_build = true            # Run cargo-audit on build
deny_warnings = true             # Treat warnings as errors
forbid_unsafe = true             # Forbid unsafe code
check_dependencies = true        # Check dependency security
max_dependency_depth = 5        # Maximum transitive dependency depth

[security.allowed_licenses]
# Allowed dependency licenses
licenses = [
  "MIT",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-3-Clause",
  "ISC",
  "Unicode-DFS-2016"
]

[git]
# Git integration settings
auto_commit = false              # Auto-commit changes
commit_style = "conventional"   # conventional/simple/custom
sign_commits = false            # GPG sign commits
hooks_enabled = true            # Enable git hooks

[git.hooks]
pre_commit = ["fmt", "lint", "test"]
pre_push = ["fmt", "lint", "test", "audit"]
commit_msg = ["conventional"]

[ci]
# CI/CD configuration
provider = "github"             # github/gitlab/azure/jenkins
enable_caching = true          # Enable dependency caching
parallel_jobs = true           # Run jobs in parallel
fail_fast = false             # Stop on first failure

[ci.github]
# GitHub Actions specific
workflow_path = ".github/workflows/ci.yml"
runs_on = ["ubuntu-latest", "windows-latest", "macos-latest"]
rust_versions = ["stable", "beta", "nightly"]
allow_failures = ["nightly"]

[validation]
# Ferrous Forge validation rules
max_file_lines = 300
max_function_lines = 50
max_cyclomatic_complexity = 10
max_cognitive_complexity = 15
ban_todo_comments = false
ban_fixme_comments = false
ban_unwrap = true
ban_expect = true
ban_panic = true
ban_unimplemented = true
ban_underscore_params = true

[validation.custom_rules]
# Custom validation rules
[[validation.custom_rules]]
name = "no_println"
pattern = 'println!\('
message = "Use tracing or log crates instead of println!"
severity = "warning"

[[validation.custom_rules]]
name = "require_error_type"
pattern = 'Result<.*, Box<dyn'
message = "Use concrete error types instead of Box<dyn Error>"
severity = "error"

[benchmarks]
# Benchmark configuration
framework = "criterion"         # criterion/bencher/custom
baseline = "main"              # Baseline branch for comparison
regression_threshold = 5       # Percentage threshold for regression
improvement_threshold = 5      # Percentage threshold for improvement

[features]
# Feature flag configuration
default = []
all_features = false          # Test with --all-features
no_default_features = false    # Test with --no-default-features

[env]
# Environment variables
RUST_BACKTRACE = "1"
RUST_LOG = "debug"
RUSTFLAGS = "-D warnings"
CARGO_INCREMENTAL = "1"

[aliases]
# Command aliases
b = "build"
c = "check"
t = "test"
r = "run"
br = "build --release"
tr = "test --release"
rr = "run --release"

[sharing]
# Configuration sharing settings
allow_export = true           # Allow exporting config
allow_import = true          # Allow importing configs
sign_exports = false         # Cryptographically sign exports
verify_imports = true        # Verify imported configs

[sharing.registry]
# Config registry settings
url = "https://registry.ferrous-forge.io"
auth_required = false
auto_publish = false

[experimental]
# Experimental features (require nightly)
const_generics = false
async_closures = false
generators = false
```

---

## Configuration Inheritance & Merging

### Inheritance Rules

1. **Deep Merge**: Nested tables are merged recursively
2. **Array Concatenation**: Arrays are concatenated (with de-duplication)
3. **Value Override**: Higher priority values override lower
4. **Explicit Null**: Use `null` or `~` to explicitly unset a value

### Example Inheritance

```toml
# System config
[dependencies]
serde = "1.0"
tokio = "1.0"

# User config  
[dependencies]
tokio = { version = "1.0", features = ["full"] }  # Overrides system
anyhow = "1.0"  # Adds to system

# Result
[dependencies]
serde = "1.0"  # From system
tokio = { version = "1.0", features = ["full"] }  # From user (override)
anyhow = "1.0"  # From user (addition)
```

---

## Template Variables & Expansion

### Variable Syntax

```toml
[project]
name = "{{project_name}}"
author = "{{author_name}} <{{author_email}}>"
repository = "https://github.com/{{github_username}}/{{project_name}}"
```

### Built-in Variables

- `{{project_name}}` - Project name from CLI
- `{{author_name}}` - From git config or template vars
- `{{author_email}}` - From git config or template vars
- `{{year}}` - Current year
- `{{date}}` - Current date (ISO format)
- `{{rust_version}}` - Current Rust version
- `{{edition}}` - Target Rust edition

---

## Command Integration

### Config Management Commands

```bash
# View configuration
ferrous-forge config show               # Show merged config
ferrous-forge config show --level user  # Show only user config
ferrous-forge config show --json        # Output as JSON

# Edit configuration
ferrous-forge config set rust.minimum_version "1.90.0"
ferrous-forge config unset templates.default_template
ferrous-forge config add dependencies.new_crate "1.0"

# Import/Export
ferrous-forge config export > my-config.toml
ferrous-forge config import shared-config.toml
ferrous-forge config validate config.toml

# Share configurations
ferrous-forge config publish --name "web-service-defaults"
ferrous-forge config fetch "ferrous-forge/web-service-defaults"
```

### Project Creation with Config

```bash
# Create project with specific config
ferrous-forge new my-project --config=web-service.toml

# Create from template with config overlay
ferrous-forge new my-project --template=axum --config=prod-settings.toml

# Initialize existing project
cd existing-project
ferrous-forge init --config=~/.configs/rust-defaults.toml
```

---

## Cargo.toml Extension

The configuration system can generate and maintain Cargo.toml:

```rust
// Pseudo-code for Cargo.toml generation
fn generate_cargo_toml(config: Config, project: Project) -> String {
    let mut cargo_toml = CargoToml::new();
    
    // Apply base configuration
    cargo_toml.apply_config(&config);
    
    // Apply template if specified
    if let Some(template) = project.template {
        cargo_toml.apply_template(&template);
    }
    
    // Apply project-specific overrides
    cargo_toml.apply_overrides(&project.overrides);
    
    // Expand variables
    cargo_toml.expand_variables(&project.variables);
    
    cargo_toml.to_string()
}
```

---

## Community Sharing Protocol

### Publishing a Config

1. Create configuration with metadata
2. Validate configuration
3. Sign configuration (optional)
4. Upload to registry or GitHub

### Config Package Format

```
my-config.ferrous/
├── config.toml          # Main configuration
├── manifest.toml        # Metadata and requirements
├── templates/          # Associated templates
│   ├── Cargo.toml.hbs
│   └── src/
├── hooks/              # Setup scripts
│   ├── pre-install.sh
│   └── post-install.sh
└── README.md          # Documentation
```

### Registry Protocol

```json
{
  "name": "web-service-defaults",
  "version": "1.0.0",
  "author": "dev@example.com",
  "checksum": "sha256:...",
  "downloads": 1523,
  "rating": 4.8,
  "tags": ["web", "async", "production"],
  "requires": {
    "ferrous_forge": ">=2.0.0",
    "rust": ">=1.85.0"
  }
}
```

---

## Implementation Details

### Config Parser Implementation

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FerrousConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    
    pub rust: RustConfig,
    pub edition: EditionConfig,
    pub templates: TemplateConfig,
    pub dependencies: HashMap<String, Dependency>,
    
    #[serde(default)]
    pub lints: LintConfig,
    
    #[serde(flatten)]
    pub extensions: HashMap<String, toml::Value>,
}

impl FerrousConfig {
    /// Load and merge configurations from all levels
    pub fn load() -> Result<Self> {
        let mut config = Self::load_system()?;
        config.merge(Self::load_user()?);
        config.merge(Self::load_workspace()?);
        config.merge(Self::load_project()?);
        Ok(config)
    }
    
    /// Merge another config into this one
    pub fn merge(&mut self, other: Self) {
        // Deep merge implementation
    }
    
    /// Export configuration for sharing
    pub fn export(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(Into::into)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validation logic
        Ok(())
    }
}
```

---

## Migration from v1.0

### Automatic Migration

```bash
ferrous-forge config migrate
```

This command will:
1. Backup existing configuration
2. Convert v1.0 format to v2.0
3. Validate the migration
4. Offer to review changes

### Manual Migration Guide

Key changes from v1.0 to v2.0:
- `enforce_edition_2024` → `edition.target_edition`
- `max_file_lines` → `validation.max_file_lines`
- `clippy_rules` → `lints.clippy`
- New sections: `templates`, `sharing`, `ci`

---

## Security Considerations

1. **Config Validation**: All imported configs are validated
2. **Script Sandboxing**: Hook scripts run in restricted environment
3. **Dependency Verification**: Check dependency signatures
4. **Registry Authentication**: Optional auth for private registries
5. **Local Override**: Local configs always take precedence

---

## Future Enhancements

### Version 2.1
- YAML configuration support
- Config inheritance from URLs
- Dynamic variable resolution
- Config composition (include other configs)

### Version 2.2  
- Config versioning and rollback
- Team configuration management
- IDE integration for config editing
- AI-powered config recommendations

### Version 3.0
- Distributed config synchronization
- Blockchain-based config verification
- Cross-language project support
- Universal package manager integration

# Ferrous Forge Repository Structure

This document outlines the structure of the Ferrous Forge repository and explains where different types of files belong.

## Root Directory

```
├── Cargo.toml              # Rust package manifest
├── Cargo.lock              # Dependency lock file
├── justfile                # Task runner commands
├── README.md               # Main project documentation
├── VISION.md               # Project vision and philosophy
├── ROADMAP.md              # Development roadmap
├── FEATURES.md             # Feature status tracking
├── CHANGELOG.md            # Version history
├── CONTRIBUTING.md         # Contribution guidelines
├── CODE_OF_CONDUCT.md      # Community standards
├── SECURITY.md             # Security policy
├── CONTRIBUTORS.md         # List of contributors
├── LICENSE-MIT             # MIT license
├── LICENSE-APACHE          # Apache 2.0 license
├── .gitignore              # Git ignore rules
├── .gitattributes          # Git attributes
└── rust-toolchain.toml     # Rust toolchain specification
```

## Source Code

```
src/
├── main.rs                 # Binary entry point
├── lib.rs                  # Library root
├── cli.rs                  # CLI argument parsing
├── error.rs                # Error types
├── commands/               # Command implementations
│   ├── mod.rs              # Command module root
│   ├── init.rs             # Init command
│   ├── validate.rs         # Validate command
│   ├── fix.rs              # Fix command
│   ├── config.rs           # Config commands
│   ├── safety/             # Safety pipeline commands
│   ├── rust/               # Rust management commands
│   ├── edition/            # Edition management commands
│   └── template/           # Template commands
├── config/                 # Configuration system
│   ├── mod.rs
│   ├── hierarchy/          # Hierarchical config
│   └── locking/            # Config locking
├── validation/             # Validation engine
├── templates/              # Template system
├── git_hooks/              # Git hook scripts
├── rust_version/           # Rust version management
└── ...
```

## Documentation

```
docs/
├── dev/                    # Development documentation
│   ├── adr/                # Architecture Decision Records
│   └── specs/              # Technical specifications
├── rust-ecosystem-guide.md # New to Rust guide
├── installation.md         # Installation guide
├── configuration.md        # Configuration guide
├── standards.md            # Standards reference
├── integration.md          # Integration guide
├── troubleshooting.md      # Troubleshooting
├── migration.md            # Migration guide
├── USER_STORIES.json       # User stories
└── RUSTDOC-STANDARDS.md    # Documentation standards
```

## Tests

```
tests/
├── integration_tests.rs    # Integration tests
├── fixtures/               # Test fixtures
└── snapshots/              # Test snapshots
```

## GitHub Configuration

```
.github/
├── workflows/              # GitHub Actions
│   ├── ci.yml              # CI workflow
│   ├── release.yml         # Release workflow
│   └── update-packages.yml # Package manager updates
├── ISSUE_TEMPLATE/         # Issue templates
│   ├── config.yml          # Issue template config
│   ├── bug_report.md       # Bug report template
│   └── feature_request.md  # Feature request template
├── PULL_REQUEST_TEMPLATE.md # PR template
└── dependabot.yml          # Dependabot configuration
```

## Templates

```
templates/
├── cli-app/                # CLI application template
├── library/                # Library template
├── wasm/                   # WebAssembly template
├── embedded/               # Embedded template
├── web-service/            # Web service template
├── plugin/                 # Plugin template
└── workspace/              # Workspace template
```

## Packaging

```
packaging/
├── homebrew/               # Homebrew formula
├── aur/                    # Arch Linux PKGBUILD
├── nix/                    # Nix derivation
└── chocolatey/             # Chocolatey package
```

## Editor Support

```
editors/
└── vscode/                 # VS Code extension
    ├── package.json
    ├── src/
    └── README.md
```

## CI/CD Artifacts

Generated during CI/CD:

```
target/
├── debug/                  # Debug builds
├── release/                # Release builds
├── doc/                    # Generated documentation
└── ...

.cleo/                      # CLEO task management
.forge/                     # Forge configuration (test projects)
```

## Where to Add New Files

### New Command
1. Add to `src/commands/<name>.rs`
2. Register in `src/commands/mod.rs`
3. Add to CLI in `src/cli.rs`
4. Add tests in `tests/` or inline

### New Documentation
1. Architecture decisions → `docs/dev/adr/`
2. Technical specs → `docs/dev/specs/`
3. User guides → `docs/` root
4. API docs → `///` comments in source

### New Template
1. Create directory in `templates/<name>/`
2. Add `manifest.toml`
3. Add template files
4. Register in template registry

### New GitHub Workflow
1. Add to `.github/workflows/`
2. Follow naming convention: `<purpose>.yml`
3. Update this document

## File Naming Conventions

- **Rust files**: `snake_case.rs`
- **Markdown files**: `UPPERCASE.md` for top-level, `lowercase.md` for docs
- **Directories**: `lowercase` or `kebab-case`
- **Templates**: `lowercase` with descriptive names
- **Tests**: `test_<name>.rs` or `<module>_tests.rs`

## Important Notes

1. **Never edit `README.md` directly** — it's generated from `lib.rs` by `cargo-rdme`
2. **Never edit `Cargo.lock` manually** — managed by Cargo
3. **Always run `cargo fmt` before committing**
4. **Always run `cargo clippy` and fix warnings**
5. **Keep ADRs in `docs/dev/adr/` up to date**
6. **Update `CHANGELOG.md` for user-facing changes**

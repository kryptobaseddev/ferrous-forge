//! Workspace template file content generation

use super::templates::{cli_main_content, core_lib_content, utils_lib_content};
use std::collections::HashMap;

/// Create the files for workspace template
pub fn create_workspace_files() -> HashMap<String, String> {
    let mut files = HashMap::new();

    files.insert("Cargo.toml".to_string(), workspace_cargo_toml());
    files.insert("README.md".to_string(), readme_content());
    files.insert("core/Cargo.toml".to_string(), core_cargo_toml());
    files.insert("core/src/lib.rs".to_string(), core_lib_content());
    files.insert("cli/Cargo.toml".to_string(), cli_cargo_toml());
    files.insert("cli/src/main.rs".to_string(), cli_main_content());
    files.insert("utils/Cargo.toml".to_string(), utils_cargo_toml());
    files.insert("utils/src/lib.rs".to_string(), utils_lib_content());
    files.insert(
        ".ferrous-forge/config.toml".to_string(),
        config_toml_content(),
    );

    files
}

/// Workspace Cargo.toml content
fn workspace_cargo_toml() -> String {
    r#"[workspace]
resolver = "2"
members = ["core", "cli", "utils"]

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["{{author}}"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/{{author}}/{{workspace_name}}"

[workspace.dependencies]
# Shared dependencies across workspace members
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.40", features = ["full"] }
clap = { version = "4.5", features = ["derive"] }

# Dev dependencies
tempfile = "3.10"
assert_cmd = "2.0"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"
"#
    .to_string()
}

/// README.md content
fn readme_content() -> String {
    r#"# {{workspace_name}}

{{description}}

## Architecture

This workspace consists of the following crates:

- **core**: Core business logic and shared functionality
- **cli**: Command-line interface
- **utils**: Utility functions and helpers

## Building

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Check code quality
cargo clippy --workspace --all-features
```

## Usage

```bash
# Run the CLI application
cargo run --bin cli

# Run tests for specific crate
cargo test -p core
```

## License

This project is licensed under the MIT OR Apache-2.0 license.
"#
    .to_string()
}

/// Core crate Cargo.toml
fn core_cargo_toml() -> String {
    r#"[package]
name = "{{workspace_name}}-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Core functionality for {{workspace_name}}"

[dependencies]
anyhow.workspace = true
thiserror.workspace = true
serde.workspace = true

[dev-dependencies]
tempfile.workspace = true
tokio = { workspace = true, features = ["test-util"] }
"#
    .to_string()
}

/// CLI crate Cargo.toml
fn cli_cargo_toml() -> String {
    r#"[package]
name = "{{workspace_name}}-cli"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Command-line interface for {{workspace_name}}"

[[bin]]
name = "cli"
path = "src/main.rs"

[dependencies]
{{workspace_name}}-core = { path = "../core" }
{{workspace_name}}-utils = { path = "../utils" }
clap.workspace = true
anyhow.workspace = true
tokio.workspace = true

[dev-dependencies]
assert_cmd.workspace = true
tempfile.workspace = true
"#
    .to_string()
}

/// Utils crate Cargo.toml
fn utils_cargo_toml() -> String {
    r#"[package]
name = "{{workspace_name}}-utils"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Utility functions for {{workspace_name}}"

[dependencies]
anyhow.workspace = true
tokio.workspace = true

[dev-dependencies]
tempfile.workspace = true
"#
    .to_string()
}

/// .ferrous-forge/config.toml content
fn config_toml_content() -> String {
    r#"# Ferrous Forge configuration for workspace projects

[validation]
max_line_length = 100
max_file_length = 300
max_function_length = 50
allow_unwrap = false
allow_expect = false

[workspace]
# Workspace-specific configuration
check_all_members = true
unified_formatting = true
"#
    .to_string()
}

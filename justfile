[tools.just]
# Use just for running commands

# Default recipe - show help
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run all tests
test:
    cargo test --all-features

# Run tests with output
test-verbose:
    cargo test --all-features -- --nocapture

# Check code formatting
check-format:
    cargo fmt --all -- --check

# Format all code
format:
    cargo fmt --all

# Run clippy lints
clippy:
    cargo clippy --all-features --all-targets -- -D warnings

# Run all checks (format, clippy, test)
check-all: check-format clippy test
    @echo "✅ All checks passed!"

# Run Ferrous Forge validation on the project itself
validate:
    cargo run -- validate .

# Generate documentation
doc:
    cargo doc --no-deps --all-features

# Open documentation in browser
doc-open:
    cargo doc --no-deps --all-features --open

# Run benchmarks
bench:
    cargo bench

# Run security audit
audit:
    cargo audit

# Install locally for testing
install-local:
    cargo install --path .

# Clean build artifacts
clean:
    cargo clean

# Run the project in development mode
run *ARGS:
    cargo run -- {{ARGS}}

# Watch for changes and run tests (requires cargo-watch)
watch:
    cargo watch -x test

# Release a new version (requires version argument)
release version:
    @echo "Releasing version {{version}}..."
    @echo "1. Update version in Cargo.toml"
    @echo "2. Update CHANGELOG.md"
    @echo "3. Run tests: just check-all"
    @echo "4. Create git tag: git tag v{{version}}"
    @echo "5. Push tag: git push origin v{{version}}"
    @echo "6. GitHub Actions will build and publish"

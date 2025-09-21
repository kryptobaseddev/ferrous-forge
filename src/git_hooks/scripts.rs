//! Git hook scripts content

/// Pre-commit hook script content
pub const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# Ferrous Forge pre-commit hook
# Automatically validates code before allowing commits

set -e

echo "🔨 Running Ferrous Forge pre-commit validation..."

# Check if ferrous-forge is installed
if ! command -v ferrous-forge >/dev/null 2>&1; then
    echo "⚠️  Ferrous Forge not found in PATH"
    echo "   Install with: cargo install ferrous-forge"
    echo "   Skipping validation..."
    exit 0
fi

# Run formatting check
echo "📝 Checking code formatting..."
if ! cargo fmt -- --check >/dev/null 2>&1; then
    echo "❌ Code is not formatted properly"
    echo "   Run 'cargo fmt' to fix formatting"
    exit 1
fi

# Run Ferrous Forge validation
echo "🔍 Running standards validation..."
if ! ferrous-forge validate --quiet; then
    echo "❌ Ferrous Forge validation failed"
    echo "   Run 'ferrous-forge validate' to see detailed errors"
    echo "   Fix all violations before committing"
    exit 1
fi

# Run clippy
echo "📎 Running clippy checks..."
if ! cargo clippy -- -D warnings 2>/dev/null; then
    echo "❌ Clippy found issues"
    echo "   Run 'cargo clippy' to see warnings"
    exit 1
fi

echo "✅ All pre-commit checks passed!"
"#;

/// Pre-push hook script content
pub const PRE_PUSH_HOOK: &str = r#"#!/bin/sh
# Ferrous Forge pre-push hook
# Runs comprehensive validation before allowing pushes

set -e

echo "🚀 Running Ferrous Forge pre-push validation..."

# Check if ferrous-forge is installed
if ! command -v ferrous-forge >/dev/null 2>&1; then
    echo "⚠️  Ferrous Forge not found in PATH"
    echo "   Install with: cargo install ferrous-forge"
    echo "   Skipping validation..."
    exit 0
fi

# Run tests
echo "🧪 Running tests..."
if ! cargo test --quiet; then
    echo "❌ Tests failed"
    echo "   Fix failing tests before pushing"
    exit 1
fi

# Run full validation
echo "🔍 Running full standards validation..."
if ! ferrous-forge validate; then
    echo "❌ Ferrous Forge validation failed"
    echo "   Fix all violations before pushing"
    exit 1
fi

echo "✅ All pre-push checks passed!"
"#;

/// Commit-msg hook script content
pub const COMMIT_MSG_HOOK: &str = r#"#!/bin/sh
# Ferrous Forge commit-msg hook
# Validates commit message format

commit_msg_file=$1
commit_msg=$(cat "$commit_msg_file")

# Check for conventional commit format
if ! echo "$commit_msg" | \
    grep -qE "^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .{1,50}"; then
    echo "❌ Invalid commit message format"
    echo "   Use conventional commit format: type(scope): description"
    echo "   Examples:"
    echo "   - feat: add new validation rule"
    echo "   - fix: resolve unwrap detection bug"
    echo "   - docs: update README"
    exit 1
fi

echo "✅ Commit message format valid"
"#;

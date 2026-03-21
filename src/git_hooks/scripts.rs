//! Git hook scripts content for mandatory blocking hooks
//!
//! @task T017
//! @epic T014

/// Pre-commit hook script content - BLOCKING by default
pub const PRE_COMMIT_HOOK: &str = r#"#!/bin/bash
# Ferrous Forge Mandatory Safety Pipeline - Pre-Commit Hook
# @task T017 @epic T014
# 
# This hook BLOCKS commits if safety checks fail.
# Use 'ferrous-forge safety bypass --stage=pre-commit --reason="..."' to bypass.

set -e

echo ""
echo "🛡️  Ferrous Forge Safety Pipeline - Pre-Commit"
echo "═══════════════════════════════════════════════════"

# Check if ferrous-forge is installed
if ! command -v ferrous-forge >/dev/null 2>&1; then
    echo ""
    echo "⚠️  Ferrous Forge not found in PATH"
    echo "   Install with: cargo install ferrous-forge"
    echo "   Skipping safety checks..."
    exit 0
fi

# Check for active bypass
BYPASS_CHECK=$(ferrous-forge safety check-bypass --stage=pre-commit 2>/dev/null || echo "none")
if [ "$BYPASS_CHECK" = "active" ]; then
    echo ""
    echo "⚠️  Safety checks bypassed (active bypass found)"
    exit 0
fi

# Run formatting check
echo ""
echo "📝 Checking code formatting..."
if ! cargo fmt -- --check >/dev/null 2>&1; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED COMMIT"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Code is not properly formatted."
    echo ""
    echo "How to fix:"
    echo "  Run 'cargo fmt' to format your code"
    echo ""
    echo "To bypass (requires reason):"
    echo "  ferrous-forge safety bypass --stage=pre-commit --reason=\"WIP\""
    echo ""
    echo "To bypass for this commit only:"
    echo "  git commit --no-verify"
    echo ""
    exit 1
fi

# Run Ferrous Forge validation
echo "🔍 Running Ferrous Forge validation..."
if ! ferrous-forge validate 2>&1; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED COMMIT"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Validation failed. Fix the issues before committing."
    echo ""
    echo "How to fix:"
    echo "  1. Run 'ferrous-forge validate' to see all errors"
    echo "  2. Fix the reported violations"
    echo "  3. Try committing again"
    echo ""
    echo "To bypass (requires reason):"
    echo "  ferrous-forge safety bypass --stage=pre-commit --reason=\"WIP commit\""
    echo ""
    echo "To bypass for this commit only:"
    echo "  git commit --no-verify"
    echo ""
    exit 1
fi

# Run clippy
echo "📎 Running clippy checks..."
if ! cargo clippy -- -D warnings 2>/dev/null; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED COMMIT"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Clippy found issues. Fix the warnings before committing."
    echo ""
    echo "How to fix:"
    echo "  Run 'cargo clippy -- -D warnings' to see all issues"
    echo ""
    exit 1
fi

echo ""
echo "✅ All pre-commit checks passed! Commit allowed."
echo ""
exit 0
"#;

/// Pre-push hook script content - BLOCKING by default
pub const PRE_PUSH_HOOK: &str = r#"#!/bin/bash
# Ferrous Forge Mandatory Safety Pipeline - Pre-Push Hook
# @task T017 @epic T014
#
# This hook BLOCKS pushes if safety checks fail.
# Use 'ferrous-forge safety bypass --stage=pre-push --reason="..."' to bypass.

set -e

echo ""
echo "🛡️  Ferrous Forge Safety Pipeline - Pre-Push"
echo "═══════════════════════════════════════════════════"

# Check if ferrous-forge is installed
if ! command -v ferrous-forge >/dev/null 2>&1; then
    echo ""
    echo "⚠️  Ferrous Forge not found in PATH"
    echo "   Install with: cargo install ferrous-forge"
    echo "   Skipping safety checks..."
    exit 0
fi

# Check for active bypass
BYPASS_CHECK=$(ferrous-forge safety check-bypass --stage=pre-push 2>/dev/null || echo "none")
if [ "$BYPASS_CHECK" = "active" ]; then
    echo ""
    echo "⚠️  Safety checks bypassed (active bypass found)"
    exit 0
fi

# Run tests
echo ""
echo "🧪 Running tests..."
if ! cargo test --quiet 2>&1; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED PUSH"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Tests failed. Fix failing tests before pushing."
    echo ""
    echo "How to fix:"
    echo "  Run 'cargo test' to see detailed test failures"
    echo ""
    echo "To bypass (requires reason):"
    echo "  ferrous-forge safety bypass --stage=pre-push --reason=\"Emergency fix\""
    echo ""
    exit 1
fi

# Run full validation
echo "🔍 Running full standards validation..."
if ! ferrous-forge validate 2>&1; then
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "🛡️  FERROUS FORGE BLOCKED PUSH"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "Validation failed. Fix the issues before pushing."
    echo ""
    echo "How to fix:"
    echo "  1. Run 'ferrous-forge validate' to see all errors"
    echo "  2. Fix the reported violations"
    echo "  3. Try pushing again"
    echo ""
    echo "To bypass (requires reason):"
    echo "  ferrous-forge safety bypass --stage=pre-push --reason=\"Emergency fix\""
    echo ""
    exit 1
fi

# Run security audit
echo "🔒 Running security audit..."
if command -v cargo-audit >/dev/null 2>&1; then
    if ! cargo audit 2>/dev/null; then
        echo ""
        echo "⚠️  Security audit found vulnerabilities"
        echo "   Run 'cargo audit' for details"
        echo ""
    fi
else
    echo "   (cargo-audit not installed, skipping)"
fi

echo ""
echo "✅ All pre-push checks passed! Push allowed."
echo ""
exit 0
"#;

/// Commit-msg hook script content
pub const COMMIT_MSG_HOOK: &str = r#"#!/bin/bash
# Ferrous Forge commit-msg hook
# Validates commit message format

commit_msg_file=$1
commit_msg=$(cat "$commit_msg_file")

# Check for conventional commit format
if ! echo "$commit_msg" | \
    grep -qE "^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .{1,50}"; then
    echo ""
    echo "❌ Invalid commit message format"
    echo ""
    echo "Use conventional commit format: type(scope): description"
    echo ""
    echo "Examples:"
    echo "  feat: add new validation rule"
    echo "  fix: resolve unwrap detection bug"
    echo "  docs: update README"
    echo ""
    exit 1
fi

echo "✅ Commit message format valid"
"#;

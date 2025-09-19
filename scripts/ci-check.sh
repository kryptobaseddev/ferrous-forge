#!/bin/bash
# CI/CD compatibility check script
# Run this before every commit to ensure CI will pass

set -e

echo "🔍 Running CI/CD compatibility checks..."

echo "📝 1. Format Check..."
cargo fmt --check
echo "✅ Format check passed"

echo "🔍 2. Clippy Check..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy check passed"

echo "🧪 3. Test Check..."
cargo test --all-targets --all-features
echo "✅ Test check passed"

echo "🏗️ 4. Build Check..."
cargo build --release
echo "✅ Build check passed"

echo "📦 5. Publish Dry Run..."
cargo publish --dry-run
echo "✅ Publish dry run passed"

echo ""
echo "🎉 All CI/CD checks passed! Safe to commit and push."

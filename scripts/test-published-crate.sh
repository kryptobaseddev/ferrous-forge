#!/bin/bash
# Test script for verifying the published ferrous-forge crate

set -e

VERSION="1.4.0"
TEST_DIR="/tmp/ferrous-forge-test-$$"

echo "🧪 Testing Ferrous Forge v$VERSION from crates.io"
echo "=================================================="

# Create clean test environment
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "📁 Working in: $TEST_DIR"

# Install from crates.io
echo "📦 Installing ferrous-forge@$VERSION from crates.io..."
cargo install ferrous-forge@$VERSION --force

# Verify installation
echo "✅ Verifying installation..."
ferrous-forge --version
ferrous-forge --help | head -5

# Test core functionality
echo "🔍 Testing core functionality..."

# Test template system
echo "📋 Testing template system..."
ferrous-forge template list

# Create a test project from template
echo "🏗️ Creating test project from template..."
ferrous-forge template create cli-app test-cli-project --var project_name=test-cli

# Validate the generated project
echo "🔍 Validating generated project..."
cd test-cli-project
ferrous-forge validate .

# Test rust version checking
echo "🦀 Testing rust version functionality..."
ferrous-forge rust check

# Test edition checking
echo "📚 Testing edition functionality..."
ferrous-forge edition check

echo ""
echo "🎉 SUCCESS: All tests passed!"
echo "✅ ferrous-forge@$VERSION is working correctly from crates.io"
echo "📁 Test artifacts in: $TEST_DIR"

# Cleanup option
read -p "🗑️ Delete test directory? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cd /
    rm -rf "$TEST_DIR"
    echo "🗑️ Test directory cleaned up"
else
    echo "📁 Test directory preserved at: $TEST_DIR"
fi
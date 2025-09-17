#!/bin/bash
# Ferrous Forge - Cargo Command Wrapper
# This script intercepts cargo commands and applies Ferrous Forge standards

# Find the real cargo binary
REAL_CARGO=$(which -a cargo | grep -v "$HOME/.local/bin/cargo" | head -1)

if [ -z "$REAL_CARGO" ]; then
    echo "Error: Could not find real cargo binary"
    exit 1
fi

# Check if Ferrous Forge is enabled
if [ "$FERROUS_FORGE_ENABLED" != "1" ]; then
    exec "$REAL_CARGO" "$@"
    exit $?
fi

# Function to apply Ferrous Forge standards to new projects
apply_ferrous_forge_standards() {
    local project_name="$1"
    local project_path="$2"
    
    echo "🔨 Applying Ferrous Forge standards to '$project_name'..."
    
    # Check if ferrous-forge binary is available
    if command -v ferrous-forge >/dev/null 2>&1; then
        # Use the ferrous-forge binary to apply templates
        ferrous-forge apply-templates "$project_path" 2>/dev/null || {
            echo "⚠️  Could not apply Ferrous Forge templates automatically"
            echo "   Run 'ferrous-forge validate $project_path' to check compliance"
        }
    else
        echo "⚠️  Ferrous Forge binary not found in PATH"
        echo "   Install with: cargo install ferrous-forge"
    fi
}

# Handle different cargo commands
case "$1" in
    "new")
        # Run the original cargo new command
        "$REAL_CARGO" "$@"
        exit_code=$?
        
        if [ $exit_code -eq 0 ] && [ -n "$2" ]; then
            # Extract project name (last argument that doesn't start with -)
            project_name=""
            for arg in "$@"; do
                if [[ ! "$arg" =~ ^- ]] && [ "$arg" != "new" ]; then
                    project_name="$arg"
                fi
            done
            
            if [ -n "$project_name" ] && [ -d "$project_name" ]; then
                apply_ferrous_forge_standards "$project_name" "$(pwd)/$project_name"
            fi
        fi
        
        exit $exit_code
        ;;
        
    "build"|"test"|"run"|"check")
        # Validate with Ferrous Forge before building/testing/running
        if command -v ferrous-forge >/dev/null 2>&1; then
            echo "🦀 Running Ferrous Forge validation..."
            ferrous-forge validate . --quiet || {
                echo "❌ Ferrous Forge validation failed"
                echo "   Fix the violations above before building"
                exit 1
            }
        fi
        
        # Run the original command
        exec "$REAL_CARGO" "$@"
        ;;
        
    *)
        # For all other commands, just pass through
        exec "$REAL_CARGO" "$@"
        ;;
esac
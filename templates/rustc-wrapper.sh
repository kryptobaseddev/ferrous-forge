#!/usr/bin/env bash
# Ferrous Forge - RustC Command Wrapper
# This script intercepts rustc commands and applies Ferrous Forge standards

# Check if Ferrous Forge is enabled
if [ "$FERROUS_FORGE_ENABLED" = "0" ]; then
    # Disabled, pass through to real rustc
    exec rustc "$@"
fi

# Find the real rustc binary
# First try rustc.real (if we've already moved it)
if command -v rustc.real >/dev/null 2>&1; then
    REAL_RUSTC="rustc.real"
# Otherwise find the system rustc
elif [ -x "/usr/bin/rustc" ] && [ "$0" != "/usr/bin/rustc" ]; then
    REAL_RUSTC="/usr/bin/rustc"
elif [ -x "/usr/local/bin/rustc" ] && [ "$0" != "/usr/local/bin/rustc" ]; then
    REAL_RUSTC="/usr/local/bin/rustc"
else
    # Try to find it in PATH, excluding our wrapper
    REAL_RUSTC=$(which -a rustc | grep -v "$0" | head -1)
fi

if [ -z "$REAL_RUSTC" ]; then
    echo "Error: Could not find real rustc binary" >&2
    exit 1
fi

# Check if this is just a query command (no compilation)
for arg in "$@"; do
    case "$arg" in
        --version|--help|-V|-h|--print*|--emit=dep-info|--emit=metadata)
            # Just pass through for informational commands
            exec "$REAL_RUSTC" "$@"
            ;;
    esac
done

# Check if ferrous-forge binary is available
if command -v ferrous-forge >/dev/null 2>&1; then
    # For actual compilation, run validation first
    echo "🔨 Running Ferrous Forge validation..." >&2
    
    # Run validation quietly
    ferrous-forge validate --quiet 2>/dev/null
    VALIDATION_RESULT=$?
    
    if [ $VALIDATION_RESULT -ne 0 ]; then
        echo "❌ Ferrous Forge validation failed!" >&2
        echo "   Fix the violations above before compiling." >&2
        echo "   Run 'ferrous-forge validate' for details." >&2
        
        # Check if strict mode is enabled
        if [ "$FERROUS_FORGE_STRICT" = "1" ]; then
            echo "   Compilation blocked due to FERROUS_FORGE_STRICT=1" >&2
            exit 1
        else
            echo "   ⚠️  Continuing with compilation (strict mode disabled)" >&2
        fi
    else
        echo "✅ Ferrous Forge validation passed" >&2
    fi
else
    echo "⚠️  Ferrous Forge binary not found in PATH" >&2
    echo "   Install with: cargo install ferrous-forge" >&2
fi

# Execute the real rustc with all original arguments
exec "$REAL_RUSTC" "$@"
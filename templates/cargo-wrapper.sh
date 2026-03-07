#!/bin/bash
# Ferrous Forge - Cargo Command Wrapper
# Intercepts cargo commands and applies Ferrous Forge standards
#
# Tiered blocking:
#   Edition/version violations (LOCKED SETTINGS) → block ALL cargo commands
#   Style violations (file size, function size)   → warn during dev, block at publish
#
# Environment variables:
#   FERROUS_FORGE_ENABLED=1         — must be set to activate interception
#   FERROUS_FORGE_BYPASS=true       — skip style checks; locked settings still enforced
#   FERROUS_FORGE_FORCE_BYPASS=true — skip ALL checks (use only in emergencies)

# Find the real cargo binary
REAL_CARGO=$(which -a cargo | grep -v "$HOME/.local/bin/cargo" | head -1)

if [ -z "$REAL_CARGO" ]; then
    echo "Error: Could not find real cargo binary"
    exit 1
fi

# Check if Ferrous Forge is enabled
if [ "$FERROUS_FORGE_ENABLED" != "1" ]; then
    exec "$REAL_CARGO" "$@"
fi

# Force bypass — skip everything
if [ "$FERROUS_FORGE_FORCE_BYPASS" = "true" ]; then
    echo "⚠️  FERROUS FORGE FORCE BYPASSED (FERROUS_FORGE_FORCE_BYPASS=true)" >&2
    echo "   All validation skipped. This should NEVER happen in production." >&2
    exec "$REAL_CARGO" "$@"
fi

# Function to apply Ferrous Forge standards to new projects
apply_ferrous_forge_standards() {
    local project_name="$1"
    local project_path="$2"

    echo "🔨 Applying Ferrous Forge standards to '$project_name'..."

    if command -v ferrous-forge >/dev/null 2>&1; then
        ferrous-forge apply-templates "$project_path" 2>/dev/null || {
            echo "⚠️  Could not apply Ferrous Forge templates automatically"
            echo "   Run 'ferrous-forge validate $project_path' to check compliance"
        }
    else
        echo "⚠️  Ferrous Forge binary not found in PATH"
        echo "   Install with: cargo install ferrous-forge"
    fi
}

# Check locked settings (edition/version) — always blocks when violated
check_locked_settings() {
    if command -v ferrous-forge >/dev/null 2>&1; then
        if ! ferrous-forge validate . --locked-only >/dev/null 2>&1; then
            echo "" >&2
            echo "❌ FERROUS FORGE — Locked Setting Violation" >&2
            echo "   Run: ferrous-forge validate . --locked-only" >&2
            echo "   for the full violation message." >&2
            ferrous-forge validate . --locked-only >&2 || true
            echo "" >&2
            return 1
        fi
    fi
    return 0
}

# Check style violations (file size, function size) — warn only during dev
check_style_warnings() {
    if [ "$FERROUS_FORGE_BYPASS" = "true" ]; then
        echo "⚠️  Ferrous Forge style checks bypassed (FERROUS_FORGE_BYPASS=true)" >&2
        return 0
    fi

    if command -v ferrous-forge >/dev/null 2>&1; then
        # Run full validate but don't block on style — just surface warnings
        ferrous-forge validate . 2>&1 | grep -v "LOCKED SETTING" | \
            grep -E "(FileTooLarge|FunctionTooLarge|UnderscoreBandaid|MissingModuleDoc)" | \
            head -5 | while read -r line; do
                echo "  ⚠️  $line" >&2
            done
    fi
    return 0
}

# Handle different cargo commands
case "$1" in
    "new")
        "$REAL_CARGO" "$@"
        exit_code=$?

        if [ $exit_code -eq 0 ] && [ -n "$2" ]; then
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
        echo "🦀 Ferrous Forge: Checking locked settings..."

        # Locked settings always block dev commands
        if ! check_locked_settings; then
            exit 1
        fi

        # Style: warn but don't block
        check_style_warnings

        exec "$REAL_CARGO" "$@"
        ;;

    "publish")
        echo "🦀 Ferrous Forge: Running full pre-publish validation..."

        # Full validation blocks publish
        if command -v ferrous-forge >/dev/null 2>&1; then
            if ! ferrous-forge validate .; then
                echo "❌ Ferrous Forge validation failed — fix all violations before publishing" >&2
                exit 1
            fi
        fi

        exec "$REAL_CARGO" "$@"
        ;;

    *)
        exec "$REAL_CARGO" "$@"
        ;;
esac

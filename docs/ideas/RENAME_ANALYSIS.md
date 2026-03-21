# Project Rename Analysis: Ferrous Forge → Forge

**Status:** Future Idea / Analysis  
**Not Current Plan**

## Overview

Analysis of what it would take to rename "Ferrous Forge" to just "Forge".

## Current State

- **Name:** Ferrous Forge
- **Binary:** `ferrous-forge`
- **Crate:** `ferrous-forge`
- **Config:** `~/.config/ferrous-forge/`

## Proposed Change

- **Name:** Forge
- **Binary:** `forge`
- **Crate:** `forge`
- **Config:** `~/.config/forge/`

## Impact Assessment

### Files to Update: 1,397+ references

1. **Code:** 500+ references in .rs files
2. **Documentation:** 16 markdown files
3. **Config paths:** Hardcoded in source
4. **GitHub:** Repository name, URLs
5. **Package managers:** All formulas
6. **CI/CD:** Workflow files

### Breaking Changes

- All existing installations broken
- User configs orphaned
- Git hooks fail
- Package manager installs fail
- External links break

### Migration Cost

**Time:** 4-6 weeks  
**Effort:** High  
**User disruption:** High  

## Better Alternative

Add binary alias instead:

```toml
[[bin]]
name = "ferrous-forge"
path = "src/main.rs"

[[bin]]
name = "forge"
path = "src/main.rs"
```

**Result:**
- `ferrous-forge validate` (backwards compatible)
- `forge validate` (shorthand)

**Effort:** 1 line in Cargo.toml

## Recommendation

**DO NOT RENAME** at this stage.

**Reasons:**
1. Too much breakage for minimal benefit
2. "Ferrous Forge" is good branding
3. Early stage — better to stabilize than rebrand
4. Can add `forge` alias for convenience

**If you MUST rename**, do it before wider adoption with:
- Clear migration guide
- Automated migration script
- Communication plan
- Long deprecation period

## Related

See also: [Multi-Language Vision](./MULTI_LANGUAGE_VISION.md) for related naming discussions.

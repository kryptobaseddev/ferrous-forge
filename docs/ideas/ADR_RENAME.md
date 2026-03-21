# Project Rename Analysis: Ferrous Forge → Forge-RS

## Overview

**Current Name:** Ferrous Forge  
**Proposed Name:** Forge-RS  
**Binary Name Change:** `ferrous-forge` → `forge`  
**Crate Name Change:** `ferrous-forge` → `forge-rs`  

## Impact Assessment: 🔴 **HIGH EFFORT**

This would be a **significant undertaking** affecting every aspect of the project.

---

## What Would Need to Change

### 1. Code Changes (Medium Effort)

**Cargo.toml**
```toml
[package]
name = "forge-rs"          # Changed from "ferrous-forge"
```

**Source Files**
- Update all `crate::` references
- Update documentation strings mentioning "Ferrous Forge"
- Update error messages and CLI output

**Binary Name**
- Users would run `forge` instead of `ferrous-forge`
- Much shorter and easier to type ✅

### 2. Configuration Paths (Breaking Change!)n

**Current:**
- `~/.config/ferrous-forge/`
- `/etc/ferrous-forge/`
- `./.ferrous-forge/`

**New:**
- `~/.config/forge-rs/`
- `/etc/forge-rs/`
- `./.forge-rs/`

**Impact:** All existing user configs would be orphaned. Migration script needed.

### 3. Documentation (High Effort)

Files requiring updates:
- `README.md` — Every command example
- `VISION.md` — All philosophy statements
- `ROADMAP.md` — References to current name
- `FEATURES.md` — CLI examples
- `CONTRIBUTING.md` — Development instructions
- All `docs/**/*.md` files
- Code comments in all `.rs` files

**Count:** ~500+ references to update

### 4. GitHub Repository (High Impact)

- **Repository name change** — Breaks all existing links
- **GitHub Pages** — URL changes
- **GitHub Actions** — Workflow references
- **Issue/PR history** — References become stale

**Impact:** External links (blogs, tutorials, bookmarks) all break.

### 5. Package Manager Formulas (Medium Effort)

Update all packaging files:
- `packaging/homebrew/ferrous-forge.rb` → `forge-rs.rb`
- `packaging/aur/PKGBUILD` — Package name
- `packaging/nix/default.nix` — Package name
- `packaging/chocolatey/` — All files

### 6. VS Code Extension (Medium Effort)

- `editors/vscode/package.json` — Extension name
- `editors/vscode/README.md` — All documentation
- Marketplace listing would need update

### 7. Published Crates (Irreversible)

- **crates.io** — `ferrous-forge` stays published forever
- New crate `forge-rs` would be separate
- Users need to manually migrate
- No automatic redirect possible

### 8. IDE and Editor Integrations

- **VS Code settings** — `.vscode/settings.json` references
- **Rust-analyzer** — Config paths
- All IDE configs use the old paths

### 9. Git Hooks (Breaking!)

Existing projects with hooks installed:
```bash
# Current hooks call:
ferrous-forge validate

# Would break after rename unless:
forge validate
```

**All existing installations would break immediately.**

### 10. Safety Audit Logs (Data Migration)

Existing audit logs reference old binary:
```json
{
  "command": "ferrous-forge safety bypass",
  // ...
}
```

Would show inconsistent history.

---

## Migration Strategy (If You Proceed)

### Phase 1: Preparation (1-2 weeks)
1. Create migration guide
2. Write migration script for configs
3. Update all documentation
4. Create compatibility layer (optional)

### Phase 2: Code Changes (1 week)
1. Rename crate in Cargo.toml
2. Update all internal references
3. Change config path constants
4. Update binary name

### Phase 3: Documentation (1 week)
1. Update all markdown files
2. Update code comments
3. Update examples
4. Create rename announcement

### Phase 4: Release (1 week)
1. Publish new crate `forge-rs`
2. Deprecate old crate `ferrous-forge`
3. Update GitHub repo (if changing)
4. Update package managers

### Phase 5: Support (Ongoing)
1. Support users migrating
2. Answer "why did you rename?" questions
3. Maintain compatibility notes

**Total Time:** 4-6 weeks of focused effort

---

## Pros of Renaming

✅ **Shorter binary name** — `forge` vs `ferrous-forge`  
✅ **Clearer identity** — "Forge-RS" explicitly says "Rust"  
✅ **Easier to type** — Less typing for frequent commands  
✅ **Modern feel** — `-rs` suffix is trendy in Rust ecosystem  

## Cons of Renaming

🔴 **Breaking change for all users**  
🔴 **Orphans existing installations**  
🔴 **Loses SEO and name recognition**  
🔴 **Confusion during transition**  
🔴 **Maintenance burden of two crates**  
🔴 **All external links break**  

---

## Recommendation

### 🟡 **DO NOT RENAME** (at this stage)

**Rationale:**
1. **Too much breakage** — Every existing user would be affected
2. **Early stage** — Project is still establishing itself
3. **Name is good** — "Ferrous Forge" is memorable and thematic
4. **Future cost** — Harder to rename as adoption grows

### Alternative: Add Alias

Instead of full rename, provide a shorter alias:

```bash
# In Cargo.toml
[[bin]]
name = "ferrous-forge"
path = "src/main.rs"

[[bin]]
name = "forge"
path = "src/main.rs"
```

**Result:** Both work:
- `ferrous-forge validate` (explicit)
- `forge validate` (shorthand)

**Effort:** Minimal — just add one line to Cargo.toml

---

## Conclusion

Renaming to "Forge-RS" would be a **4-6 week project** affecting:
- 500+ documentation references
- All existing user installations
- Package managers
- GitHub presence
- VS Code extension

**The cost outweighs the benefit** at this stage. Consider adding `forge` as a binary alias instead for the best of both worlds.

**If you MUST rename**, do it **NOW** before wider adoption, with a clear migration plan and user communication strategy.

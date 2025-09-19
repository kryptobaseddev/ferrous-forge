# Ferrous Forge Standards Compliance Plan
## Fix Our Own 271 Violations

---

## 🚨 **Current Status: UNACCEPTABLE**

Ferrous Forge has **271 standards violations** in its own codebase while claiming to enforce standards. This is a **critical credibility issue** that must be fixed immediately.

---

## 📊 **Violation Breakdown**

### 🚨 **File Size Violations (10 files)**
| File | Lines | Violation | Action Required |
|------|-------|-----------|-----------------|
| `src/validation.rs` | 1,133 | 3.8x over limit | **Split into 4+ modules** |
| `src/standards.rs` | 752 | 2.5x over limit | **Split into 3+ modules** |
| `src/test_coverage.rs` | 528 | 1.8x over limit | **Split into 2 modules** |
| `src/updater.rs` | 427 | 1.4x over limit | **Split into 2 modules** |
| `src/config.rs` | 461 | 1.5x over limit | **Split into 2 modules** |
| `src/formatting.rs` | 340 | 1.1x over limit | **Minor split** |
| `src/edition/migrator.rs` | 338 | 1.1x over limit | **Minor split** |
| `src/commands/edition.rs` | 334 | 1.1x over limit | **Minor split** |
| `src/git_hooks.rs` | 313 | 1.0x over limit | **Minor split** |
| `src/safety/config.rs` | 308 | 1.0x over limit | **Minor split** |

### 🚨 **Function Size Violations (41 functions)**
- Functions ranging from 51-153 lines (max allowed: 50)
- Primarily in command handlers and validation logic

### 🚨 **Underscore Bandaid Violations (82 violations)**
- Unused parameters marked with `_` prefix
- Violates "fix the design instead of hiding warnings" principle

---

## 🎯 **Systematic Fix Strategy**

### **Phase 1: Critical File Splits (Priority 1)**

#### 1. **Split `src/validation.rs` (1,133 lines → 4 modules)**
```
src/validation/
├── mod.rs              # Main interface (100 lines)
├── rust_validator.rs   # Core validator (250 lines)
├── cargo_validator.rs  # Cargo.toml validation (250 lines)
├── violation.rs        # Violation types and reporting (250 lines)
└── standards.rs        # Standards definitions (250 lines)
```

#### 2. **Split `src/standards.rs` (752 lines → 3 modules)**
```
src/standards/
├── mod.rs              # Main interface (100 lines)
├── coding_standards.rs # Code quality standards (250 lines)
├── dependency.rs       # Dependency standards (200 lines)
└── documentation.rs    # Documentation standards (200 lines)
```

#### 3. **Split `src/test_coverage.rs` (528 lines → 2 modules)**
```
src/test_coverage/
├── mod.rs              # Main interface (150 lines)
├── analyzer.rs         # Coverage analysis (200 lines)
└── reporter.rs         # Coverage reporting (178 lines)
```

### **Phase 2: Function Refactoring (Priority 2)**

#### **Large Function Strategy:**
1. **Extract helper functions** from large functions
2. **Split command handlers** into smaller, focused functions
3. **Use builder patterns** for complex initialization
4. **Extract validation logic** into separate functions

#### **Example Refactoring:**
```rust
// BEFORE: 153-line function
pub async fn handle_migrate(/* large function */) -> Result<()> {
    // 153 lines of code
}

// AFTER: Split into focused functions
pub async fn handle_migrate(/* params */) -> Result<()> {
    validate_migration_params(/* params */)?;
    let options = build_migration_options(/* params */)?;
    execute_migration(options).await?;
    report_migration_results().await?;
    Ok(())
}

fn validate_migration_params(/* params */) -> Result<()> { /* 20 lines */ }
fn build_migration_options(/* params */) -> Result<MigrationOptions> { /* 25 lines */ }
async fn execute_migration(/* options */) -> Result<()> { /* 30 lines */ }
async fn report_migration_results() -> Result<()> { /* 25 lines */ }
```

### **Phase 3: Underscore Bandaid Fixes (Priority 3)**

#### **Strategy:**
1. **Remove unused parameters** where possible
2. **Use meaningful names** instead of `_param`
3. **Add `#[allow(unused_variables)]`** only where truly necessary
4. **Refactor APIs** to eliminate unused parameters

#### **Example Fixes:**
```rust
// BEFORE: Underscore bandaid
fn handle_command(_unused_param: String, data: Data) -> Result<()> {
    process_data(data)
}

// AFTER: Remove unused parameter
fn handle_command(data: Data) -> Result<()> {
    process_data(data)
}

// OR: Use meaningful name if needed for API compatibility
fn handle_command(#[allow(unused_variables)] legacy_param: String, data: Data) -> Result<()> {
    process_data(data)
}
```

---

## 🎯 **Template System 2.0 Integration**

While fixing standards, we'll **simultaneously implement** Template System 2.0:

### **Template System Architecture**
```
src/templates/
├── mod.rs              # Enhanced template system (150 lines max)
├── engine.rs           # Template processing engine (200 lines max)
├── manifest.rs         # Template manifest handling (150 lines max)
├── repository.rs       # Community template repository (200 lines max)
├── validator.rs        # Template validation (150 lines max)
└── standard_library.rs # Built-in templates (200 lines max)
```

### **Template Features**
- 📦 **Template Creation**: `ferrous-forge template create`
- 🌐 **Community Sharing**: GitHub-based template repository
- 📋 **Template Manifests**: Dependencies, features, hooks
- 🔧 **Project Generation**: `ferrous-forge new --template=web-service`
- 📚 **Standard Library**: Built-in templates for common project types

---

## 📋 **Implementation Plan**

### **Week 1: Standards Compliance (Oct 3-6)**
- **Day 1**: Split `validation.rs` into modules
- **Day 2**: Split `standards.rs` into modules  
- **Day 3**: Fix large functions in commands
- **Day 4**: Remove underscore bandaids

### **Week 2: Template System 2.0 (Oct 7-10)**
- **Day 5**: Implement template engine and manifest system
- **Day 6**: Add community repository integration
- **Day 7**: Create standard template library
- **Day 8**: Add CLI commands and testing

### **Success Criteria**
- ✅ **Zero standards violations**: `ferrous-forge validate .` shows 0 violations
- ✅ **Template system working**: Can create, share, and use templates
- ✅ **All tests passing**: No regressions in functionality
- ✅ **CI/CD clean**: All checks pass

---

## 🚨 **Immediate Actions**

Let me start with the **most critical** file: `src/validation.rs` (1,133 lines)

**Should I start splitting this massive file into proper modules?** This is **mandatory** for credibility - we cannot claim to enforce standards while violating them ourselves.

**Priority Order:**
1. 🔥 **Fix validation.rs** (1,133 lines → 4 modules)
2. 🔥 **Fix standards.rs** (752 lines → 3 modules)  
3. 🔧 **Template System 2.0** (implement while maintaining standards)
4. 🧹 **Clean up remaining violations**

**This is essential for Ferrous Forge's credibility and integrity.** ✅

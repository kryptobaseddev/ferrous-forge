# 🤖 AI-Friendly Compliance Report

**Generated**: 2025-09-19T19:08:20.444847003+00:00
**Project**: .
**Total Violations**: 293
**Compliance**: 0.0%

## 🎯 Fix Priority Order

1. **UnwrapInProduction** - Critical for safety
2. **UnderscoreBandaid** - Implement missing functionality
3. **FunctionTooLarge** - Refactor for maintainability
4. **FileTooLarge** - Split into modules

## 🔧 Automated Fix Commands

```bash
# Generate this report
ferrous-forge validate . --ai-report

# Use AI assistant with the JSON report to implement fixes
# The JSON contains structured data for automated processing
```

## 📊 Violation Summary

### FunctionTooLarge (42 violations)
**Strategy**: Review and fix manually

**Example**: 
```rust

```

### UnderscoreBandaid (88 violations)
**Strategy**: 1. Identify what functionality the parameter should provide
2. Either implement the functionality or remove the parameter
3. Update function signature and callers

**Example**: 
```rust
// Before: fn process(_unused: String, data: Data)
// After: fn process(data: Data) or implement the unused parameter
```

### LineTooLong (17 violations)
**Strategy**: Review and fix manually

**Example**: 
```rust

```

### UnwrapInProduction (135 violations)
**Strategy**: 1. Change function to return Result<T, Error>
2. Replace .unwrap() with ?
3. Handle errors at call sites

**Example**: 
```rust
// Before: value.unwrap()
// After: value?
```

### FileTooLarge (11 violations)
**Strategy**: 1. Identify logical boundaries in the file
2. Create new module directory
3. Split into focused modules
4. Update imports

**Example**: 
```rust
// Split validation.rs into validation/mod.rs, validation/core.rs, validation/types.rs
```


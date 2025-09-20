# Claude Orchestrator Fix Instructions

## Overview
Total violations to fix: 225
Analyzable violations: 100
Confidence level: High for 87, Medium for 13, Low for 0

## Fix Priority

### Simple Fixes (26 violations)
1. /mnt/projects/ferrous-forge/src/ai_analyzer.rs:304 - BANNED: .unwrap() in production code - use proper error handling with ?
2. /mnt/projects/ferrous-forge/src/ai_analyzer.rs:442 - BANNED: .unwrap() in production code - use proper error handling with ?
3. /mnt/projects/ferrous-forge/src/ai_analyzer.rs:442 - BANNED: .expect() in production code - use proper error handling with ?

### Moderate Fixes (44 violations)
1. /mnt/projects/ferrous-forge/src/commands/edition.rs:232 - BANNED: Underscore parameter (_param) - fix the design instead of hiding warnings
2. /mnt/projects/ferrous-forge/src/commands/rust.rs:143 - BANNED: Underscore parameter (_param) - fix the design instead of hiding warnings
3. /mnt/projects/ferrous-forge/src/commands/safety.rs:9 - BANNED: Underscore parameter (_param) - fix the design instead of hiding warnings

### Complex Fixes (2 violations)
1. /mnt/projects/ferrous-forge/src/edition/migrator.rs:331 - BANNED: .unwrap() in production code - use proper error handling with ?
2. /mnt/projects/ferrous-forge/src/rust_version/github.rs:206 - BANNED: .unwrap() in production code - use proper error handling with ?

### Architectural Fixes (28 violations)
1. /mnt/projects/ferrous-forge/src/ai_analyzer.rs:358 - BANNED: Underscore parameter (_param) - fix the design instead of hiding warnings
2. /mnt/projects/ferrous-forge/src/ai_analyzer.rs:374 - BANNED: Underscore parameter (_param) - fix the design instead of hiding warnings
3. /mnt/projects/ferrous-forge/src/ai_analyzer.rs:381 - BANNED: Underscore parameter (_param) - fix the design instead of hiding warnings

## Recommended Strategies

### Progressive Error Handling Migration
Gradually replace unwrap() with proper error handling
Estimated time: 15 minutes
Confidence: 80%

### Implement Missing Functionality
Either use parameters properly or remove them
Estimated time: 30 minutes
Confidence: 60%


## AI Agent Instructions

You are an expert Rust developer tasked with fixing code violations identified by Ferrous Forge.

Your goals:
1. Fix violations while maintaining code functionality
2. Improve error handling without breaking existing behavior
3. Follow Rust best practices and idioms
4. Ensure all changes compile and pass tests
5. Add appropriate documentation for complex changes

Key principles:
- Preserve existing behavior unless explicitly broken
- Prefer explicit error handling over panics
- Use type system to prevent errors at compile time
- Write self-documenting code
- Consider performance implications

### Validation Criteria
- Code must compile
- Tests must pass
- No new violations introduced
- Performance not degraded

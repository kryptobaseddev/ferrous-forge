# Multi-Language Forge Vision

**Status:** Future Idea / Hypothetical  
**Not Current Implementation**

## Overview

This document explores a potential future where Forge becomes a **multi-language quality enforcement platform** rather than just a Rust tool.

## The Premise

You already have:
- **Ferrous Forge** — Rust enforcement (current, complete)
- **Forge-ts** — TypeScript documentation/compiler (separate project)

**Idea:** Unify these into a single "Forge" platform with Rust core + language plugins.

## Architecture Concept

```
Forge Core (Rust)
├── Configuration System (language-agnostic)
├── Safety Pipeline (language-agnostic)
├── Git Hooks (language-agnostic)
└── Plugin API
    │
    ├── Rust Plugin (ferrous-forge-rust)
    ├── TypeScript Plugin (forge-ts)
    ├── Go Plugin (future)
    └── Python Plugin (future)
```

## Core Features (Language-Agnostic)

These would work for ANY language:

1. **Configuration System**
   - Hierarchical config (System → User → Project)
   - Config locking with audit trail
   - Export/import for team sharing

2. **Safety Pipeline**
   - Git hooks for pre-commit/pre-push
   - Bypass system with mandatory reasons
   - Audit logging

3. **LSP Server**
   - Real-time diagnostics
   - Quick fixes
   - Cross-language workspace symbols

## Language-Specific Features

Each plugin provides:

### Rust Plugin
- cargo, clippy, rustdoc, rustfmt
- Edition management
- Already implemented in Ferrous Forge

### TypeScript Plugin
- npm/pnpm/yarn integration
- ESLint/Biome, TSDoc
- OpenAPI generation
- Doctest execution
- From forge-ts project

### Go Plugin (Future)
- go modules, golangci-lint
- godoc generation
- gofmt

### Python Plugin (Future)
- pip/poetry/uv, ruff/pylint
- Sphinx/MkDocs
- black/isort

## Benefits

1. **One tool for entire stack** — Monorepos with multiple languages
2. **Unified config** — One file, all languages
3. **Consistent safety pipeline** — Same bypass/audit across languages
4. **AI-agent friendly** — Consistent guardrails

## Challenges

1. **Breaking changes** — Existing Ferrous Forge users affected
2. **Complexity** — More code to maintain
3. **Scope creep** — Risk of being "jack of all trades"
4. **Migration effort** — Moving forge-ts into plugin architecture

## Decision Needed

**Option A: Keep Separate**
- Ferrous Forge (Rust) continues as-is
- Forge-ts (TypeScript) continues as-is
- Pros: Simple, focused, no breaking changes
- Cons: Two tools, fragmented ecosystem

**Option B: Unified Platform**
- Create "Forge" as umbrella project
- Rust core with plugins
- Pros: One tool, unified experience
- Cons: Breaking changes, complex migration

**Recommendation:** Keep separate for now. Revisit when:
- Ferrous Forge 2.0 planning begins
- Strong demand for multi-language unification
- Resources available for major refactoring

## Related Documents

- Current Ferrous Forge: [VISION.md](/VISION.md)
- Current Ferrous Forge: [README.md](/README.md)
- TypeScript project: `/mnt/projects/forge-ts/`

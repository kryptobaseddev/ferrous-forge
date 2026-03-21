# Ferrous Forge Roadmap

> **CLEO is the Source of Truth** - This document references CLEO tasks. All work items exist in CLEO with full details, dependencies, and priorities.

## Quick Reference

| Epic | CLEO Task | Status | Priority |
|------|-----------|--------|----------|
| Aggressive Enforcement System | [T014](cleo://task/T014) | Pending | CRITICAL |
| v1.8.0 Core Completion | [T001](cleo://task/T001) | Pending | High |
| v2.0 IDE Integration | [T002](cleo://task/T002) | Pending | Medium |

---

## Vision: Aggressive Enforcement System

Ferrous Forge is an **opinionated, aggressive** Rust development standards enforcer. Unlike other tools that suggest fixes, Forge **prevents non-compliant code** from being committed, pushed, or published.

### Core Philosophy

1. **Preconfiguration**: Sets up clippy, rustfmt, and lints optimally from project creation
2. **Locking**: Critical settings (edition, rust-version) are immutable without explicit justification
3. **Enforcement**: Blocks git operations and cargo publish by default when checks fail
4. **Agent-Proof**: Designed specifically to prevent LLM agents from "workaround-ing" standards
5. **Escape Hatches**: Bypass available but requires explicit command with audit logging

### Target Behavior

```bash
# Agent tries to change edition to fix compile error
$ sed -i 's/edition = "2024"/edition = "2021"/' Cargo.toml
# Ferrous Forge detects this and blocks the change!

# Agent must explicitly unlock with justification
$ ferrous-forge config unlock edition --reason="Upgrading dependency that requires 2021"
# This is logged and requires human awareness

# Publishing blocked if validation fails
$ cargo publish
# 🛡️ Ferrous Forge validation failed - publish blocked!
# Run 'ferrous-forge validate' to see issues

# Must explicitly bypass with justification
$ ferrous-forge safety bypass --stage=publish --reason="Emergency security patch"
```

---

## Implementation Phases

### Phase 1: Foundation (Immediate)

**Goal**: Implement aggressive enforcement core

| Task | CLEO ID | Priority | Dependencies |
|------|---------|----------|--------------|
| Config Locking System | [T015](cleo://task/T015) | High | None - Foundation |
| Cargo Publish Blocking | [T016](cleo://task/T016) | High | None |
| Mandatory Safety Hooks | [T017](cleo://task/T017) | High | T015 |
| Complete Safety CLI | [T019](cleo://task/T019) | High | T016, T017 |

**Phase 1 Success Criteria**:
- [ ] Locked settings cannot be modified without `ferrous-forge config unlock`
- [ ] `cargo publish` runs validation and blocks on failure
- [ ] Git hooks block commits/pushes by default
- [ ] Bypass commands require justification and log to audit trail

### Phase 2: Configuration & Sharing (Short-term)

**Goal**: Enable team-wide standard enforcement

| Task | CLEO ID | Priority | Dependencies |
|------|---------|----------|--------------|
| Hierarchical Config | [T018](cleo://task/T018) | High | T015 |
| Rustup Integration | [T020](cleo://task/T020) | Medium | None |
| Template Repository | [T021](cleo://task/T021) | Medium | T018 |

**Phase 2 Success Criteria**:
- [ ] Configs merge: System → User → Project
- [ ] Team can share config via `ferrous-forge config export/import`
- [ ] Rust version enforcement works
- [ ] Templates can be fetched from GitHub

### Phase 3: Ecosystem Integration (Medium-term)

**Goal**: Integrate with broader Rust ecosystem

| Task | CLEO ID | Priority | Dependencies |
|------|---------|----------|--------------|
| GitHub API Integration | [T024](cleo://task/T024) | Medium | None |
| Package Manager Distribution | [T023](cleo://task/T023) | Medium | None |
| VS Code Extension | [T022](cleo://task/T022) | Low | Phase 1 complete |

**Phase 3 Success Criteria**:
- [ ] Automatic Rust release tracking
- [ ] Installable via Homebrew, AUR, etc.
- [ ] Real-time validation in VS Code

---

## CLEO Task Index

### Epic: Aggressive Enforcement (T014)
- **T015**: Config Locking System
- **T016**: Cargo Publish Interception & Blocking
- **T017**: Mandatory Safety Pipeline Hooks
- **T018**: Hierarchical Configuration with Sharing
- **T019**: Complete Safety Pipeline CLI
- **T020**: Rustup Integration & Toolchain Management
- **T021**: Template Repository System
- **T022**: VS Code Extension for Real-time Validation
- **T023**: Package Manager Distribution
- **T024**: GitHub API Integration for Release Tracking

### Epic: v1.8.0 Core Completion (T001)
Previously tracked features being consolidated into T014 epic above.

### Epic: v2.0 IDE Integration (T002)
Future IDE and ecosystem work.

---

## Historical Context

Previous planning documents in `docs/roadmap/archive/` describe the aggressive enforcement vision in detail. These have been superseded by CLEO tasks as the source of truth.

**Key insight from historical docs**: Ferrous Forge was always intended to be aggressive and opinionated. The current implementation is too permissive. Phase 1 above brings Forge to its intended aggressive enforcement state.

---

## Success Metrics

### Technical
- Zero code published to crates.io that fails validation
- Zero commits to main that bypass safety checks without audit trail
- 100% enforcement of locked configuration values

### Adoption
- Package manager availability (Homebrew, AUR, etc.)
- Team config sharing in production use
- LLM agents using Forge as guardrails

### Quality
- Consistent standards across all Forge-managed projects
- Reduced "creative workarounds" by AI agents
- Clear audit trail of all bypasses

---

**Last Updated**: 2025-03-20  
**Source of Truth**: CLEO (see tasks above)  
**Next Review**: After Phase 1 completion

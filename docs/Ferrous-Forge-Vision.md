# Ferrous Forge — Vision & Philosophy
> **Forge better Rust, automatically.**
## The Core Vision
Ferrous Forge is an **opinionated, aggressive Rust development standards enforcer**. Unlike other tools that suggest fixes or warn about issues, Forge **physically prevents non-compliant code** from being committed, pushed, or published.
### The Problem We Solve
Rust development has a consistency problem:
- **LLM agents** routinely "workaround" lint rules to fix compile errors
- **Teams** struggle to enforce standards across multiple projects
- **CI/CD** catches issues too late in the development cycle
- **Junior developers** don't know what "good" Rust looks like
- **Code reviews** become nit-picky debates about style instead of architecture
Traditional solutions (clippy, rustfmt, CI checks) are **permissive by default**. They warn, suggest, and allow overrides. This permissiveness creates drift—small compromises that accumulate into technical debt.
### The Forge Solution
Ferrous Forge takes the opposite approach: **aggressive enforcement with escape hatches**.
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
## The Five Pillars
### 1. Preconfiguration
Forge sets up clippy, rustfmt, and lints optimally from project creation. No configuration needed—professional-grade standards out of the box.
**Current State:** ✅ Implemented
- 7 built-in templates (CLI, library, WASM, embedded, web-service, plugin, workspace)
- Automatic injection of `[lints]` blocks into Cargo.toml
- Pre-configured rustfmt.toml and clippy.toml
- VS Code settings for optimal Rust development
### 2. Locking
Critical settings (edition, rust-version) are immutable without explicit justification. Prevents "temporary" downgrades that become permanent.
**Current State:** ✅ Implemented
- Hierarchical locking: System → User → Project levels
- `ferrous-forge config lock <key> --reason="..."`
- `ferrous-forge config unlock <key> --reason="..."`
- Full audit trail of all lock/unlock operations
- Lock validation before any config changes
### 3. Enforcement
Blocks git operations and cargo publish by default when checks fail. Safety is opt-out, not opt-in.
**Current State:** ✅ Implemented
- Pre-commit hooks block commits with violations
- Pre-push hooks block pushes with test failures
- Cargo publish interception with validation
- All hooks respect bypass status
### 4. Agent-Proof
Designed specifically to prevent LLM agents from "workaround-ing" standards. Every bypass requires explicit command with justification and audit logging.
**Current State:** ✅ Implemented
- `ferrous-forge safety bypass --stage=pre-commit --reason="..."`
- 24-hour bypass duration by default
- Complete audit log with user, timestamp, and reason
- Hooks check bypass status before allowing operations
### 5. Escape Hatches
Bypass available but requires explicit command with audit logging. No silent overrides, no environment variables that disable everything.
**Current State:** ✅ Implemented
- `ferrous-forge safety bypass` with mandatory `--reason`
- `ferrous-forge safety audit` to view all bypasses
- Configurable bypass duration
- Team-wide visibility of all bypasses
## Architecture
```
┌─────────────────────────────────────────────────────────────────────────┐
│                     CONFIGURATION HIERARCHY                             │
├─────────────────────────────────────────────────────────────────────────┤
│ System: /etc/ferrous-forge/config.toml          (Organization-wide)     │
│ User:   ~/.config/ferrous-forge/config.toml     (Personal defaults)     │
│ Project: ./.ferrous-forge/config.toml           (Team-agreed standards) │
│                                                                          │
│ Locks are inherited: Project locks override User locks                  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     SAFETY PIPELINE                                     │
├─────────────────────────────────────────────────────────────────────────┤
│ Pre-Commit:  Format → Clippy → Validation → BLOCK if failed             │
│ Pre-Push:    Tests → Security Audit → Full Validation → BLOCK if failed │
│ Publish:     Validation → Dry-run → BLOCK if failed                     │
│                                                                          │
│ Bypass: ferrous-forge safety bypass --stage=X --reason="..."            │
│ Audit:  ferrous-forge safety audit                                      │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     TOOLCHAIN MANAGEMENT                                │
├─────────────────────────────────────────────────────────────────────────┤
│ Rustup Integration: ferrous-forge rust update                           │
│ Version Enforcement: Locked rust-version compliance                     │
│ Edition Management: ferrous-forge edition migrate                       │
│ Release Tracking: GitHub API integration with security advisories       │
└─────────────────────────────────────────────────────────────────────────┘
```
## Success Metrics
### Technical
- ✅ Zero code published to crates.io that fails validation
- ✅ Zero commits to main that bypass safety checks without audit trail
- ✅ 100% enforcement of locked configuration values
### Adoption
- ✅ Package manager availability (Homebrew, AUR, Nix, Chocolatey)
- ✅ Team config sharing via export/import
- ✅ LLM agents using Forge as guardrails
### Quality
- ✅ Consistent standards across all Forge-managed projects
- ✅ Reduced "creative workarounds" by AI agents
- ✅ Clear audit trail of all bypasses
## Current Implementation Status
**Version:** 1.7.6 | **Status:** Aggressive Enforcement System Complete
### ✅ Phase 1: Foundation (COMPLETE)
- T015: Config Locking System
- T016: Cargo Publish Interception & Blocking
- T017: Mandatory Safety Pipeline Hooks
- T019: Complete Safety Pipeline CLI
### ✅ Phase 2: Configuration & Sharing (COMPLETE)
- T018: Hierarchical Configuration with Sharing
- T020: Rustup Integration & Toolchain Management
- T021: Template Repository System
### ✅ Phase 3: Ecosystem Integration (COMPLETE)
- T022: VS Code Extension for Real-time Validation
- T023: Package Manager Distribution
- T024: GitHub API Integration for Release Tracking
## The Forge Philosophy
### Strict > Permissive
We believe it's better to be overly strict and have developers explicitly opt-out than to be permissive and let quality slip. Every opt-out is logged and visible.
### Prevention > Detection
Catch issues at commit time, not CI time. The feedback loop is tighter, and the cost of fixing is lower.
### Explicit > Implicit
No magic environment variables. No hidden config files. Every bypass requires an explicit command with a reason.
### Team > Individual
Configuration is hierarchical so organizations can set standards. Individual preferences are fine, but team standards take precedence.
### Code is the Source of Truth
Documentation is auto-generated from code. README is synced from lib.rs. No drift between code and docs.
## Target Users
### Team Leads
- Enforce organization-wide standards
- Audit trail of all configuration changes
- Zero-configuration onboarding for new team members
### Individual Developers
- Never worry about style debates again
- Automatic setup for new projects
- Consistent tooling across all your Rust projects
### DevOps/Platform Engineers
- Hierarchical configuration for organization standards
- Git hooks prevent bad code from entering CI/CD
- Package manager distribution for easy adoption
### AI/LLM Agents
- Clear boundaries: "These settings are locked, find another way"
- Audit trail of all workarounds attempted
- Guided fixes instead of silent violations
## Future Vision
While the core aggressive enforcement system is complete, we continue to evolve:
- **Real-time IDE Integration:** VS Code extension with inline diagnostics
- **Custom Lint Rules:** Dylint integration for organization-specific rules
- **Metrics Dashboard:** Historical trend analysis and team analytics
- **Web Dashboard:** Multi-project overview and team collaboration
## Conclusion
Ferrous Forge is not just another linting tool. It's a **quality gatekeeper** that sits between your code and your repository, ensuring that only code meeting professional-grade standards makes it to production.
Like a blacksmith forges iron into steel, Ferrous Forge shapes your Rust code into perfection.
---
**Forge better Rust, automatically.** 🔨

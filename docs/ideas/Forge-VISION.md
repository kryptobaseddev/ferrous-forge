# Forge — Vision & Philosophy
> **Forge better code, automatically — in any language.**
## The Core Vision
**Forge** is an **opinionated, aggressive multi-language development standards enforcer** with a Rust-based core. Unlike language-specific tools that only work in isolation, Forge provides **unified quality enforcement across your entire stack** — from Rust backends to TypeScript frontends, Go microservices to Python data pipelines.
### The Problem We Solve
Modern development is multi-language:
- **Monorepos** contain Rust, TypeScript, Go, and Python code
- **AI agents** work across all languages but lack consistent guardrails
- **Each language** has its own linting, formatting, and documentation tools
- **No unified way** to enforce standards across the entire codebase
- **Documentation rot** happens in every language, differently
Traditional approaches:
- ESLint for TypeScript, Clippy for Rust, golangci-lint for Go, pylint for Python...
- Each tool has different configs, different behaviors, different escape hatches
- Teams need expertise in 4+ different quality ecosystems
- AI agents learn different "workaround patterns" for each language
### The Forge Solution
**One core. One config. One safety pipeline. Multiple languages.**
```bash
# Forge auto-detects languages in your project
$ forge init --project
Detected languages: Rust, TypeScript, Go
Installing safety pipeline for all detected languages...
# All languages use the same config hierarchy
$ forge config lock required_edition --reason="Team decision"
✅ Locked for Rust: required_edition
✅ Locked for TypeScript: target
✅ Locked for Go: go_version
# One command validates everything
$ forge validate
🦀 Rust: 0 violations
⚡ TypeScript: 1 violation (missing TSDoc on public export)
🐹 Go: 0 violations
🐍 Python: 2 violations (line too long)
# Safety pipeline blocks commits in ALL languages
$ git commit -m "feat: add feature"
🛡️ Forge blocked commit:
  - TypeScript: Missing TSDoc (see src/api.ts:42)
  - Python: Line too long (see src/utils.py:15)
# One bypass works across languages
$ forge safety bypass --stage=pre-commit --reason="WIP"
✅ Bypass active for: Rust, TypeScript, Go, Python
```
## The Five Pillars
### 1. Language Detection
Forge automatically detects which languages are present in your project and loads appropriate plugins:
```rust
pub trait LanguagePlugin: Send + Sync {
    fn name(&self) -> &str;                    // "rust", "typescript", "go", "python"
    fn detect(&self, path: &Path) -> bool;    // Check for Cargo.toml, package.json, etc.
    fn rules(&self) -> Vec<ValidationRule>;  // Language-specific rules
    async fn lint(&self) -> Result<LintResult>;      // Run linter
    async fn format(&self, check: bool) -> Result<()>; // Run formatter
    async fn doc_check(&self) -> Result<DocCoverage>; // Check documentation
}
```
**Supported Languages (v1.0):**
- **Rust** — cargo, clippy, rustdoc, edition management
- **TypeScript** — npm/pnpm/yarn, ESLint/Biome, TSDoc, OpenAPI generation
- **Go** — go modules, golangci-lint, godoc, gofmt
- **Python** — pip/poetry/uv, pylint/ruff, docstrings, black/isort
### 2. Unified Configuration
One hierarchical config system works across all languages:
```toml
# ~/.config/forge/config.toml (User defaults)
[validation]
max_file_lines = 500
require_documentation = true
# Language-specific sections
[language.rust]
edition = "2024"
unsafe_forbidden = true
[language.typescript]
target = "ES2022"
strict = true
[language.go]
go_version = "1.23"
[language.python]
python_version = "3.12"
```
**Locking works across languages:**
```bash
# Lock a concept that applies to multiple languages
$ forge config lock required_edition --reason="Team decision"
🔒 Locked:
  - Rust: edition = "2024"
  - TypeScript: target = "ES2022" 
  - Go: go_version = "1.23"
  - Python: python_version = "3.12"
```
### 3. Aggressive Enforcement
**Default behavior: BLOCK, not warn.**
Every language gets the same aggressive treatment:
- Git hooks block commits with violations
- Pre-push hooks block failing tests
- Config locking prevents "temporary" downgrades
- All bypasses require explicit justification and audit logging
### 4. AI-Agent-First Design
Built for the era of LLM-driven development:
- **Deterministic enforcement** — Agents can't bypass without explicit commands
- **Audit trail** — Every bypass logged with timestamp, user, and reason
- **AI context generation** — `llms.txt` for every language in your project
- **LSP integration** — Real-time feedback in IDE (VS Code, Cursor, Zed)
- **Machine-readable output** — `--json` flag on every command
### 5. Documentation as Code
Forge treats documentation as a first-class deliverable:
**Per-language documentation generation:**
- **Rust** — rustdoc + mdBook integration
- **TypeScript** — TSDoc → Markdown/OpenAPI + llms.txt
- **Go** — godoc → static site
- **Python** — docstrings → Sphinx/MkDocs
**Unified outputs:**
- `forge doc build` — Build docs for all languages
- `forge doc serve` — Unified documentation server
- `forge doc check` — Verify coverage across all languages
## Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                        FORGE CORE (Rust)                        │
├─────────────────────────────────────────────────────────────────┤
│  CLI • Config System • Safety Pipeline • Git Hooks • LSP Server │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Plugin API
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
    ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
    │ Rust Plugin │ │  TS Plugin  │ │  Go Plugin  │
    ├─────────────┤ ├─────────────┤ ├─────────────┤
    │ • cargo     │ │ • npm/pnpm  │ │ • go mod    │
    │ • clippy    │ │ • ESLint    │ │ • golangci  │
    │ • rustdoc   │ │ • TSDoc     │ │ • godoc     │
    │ • rustfmt   │ │ • Prettier  │ │ • gofmt     │
    └─────────────┘ └─────────────┘ └─────────────┘
```
## Core Components
### 1. Configuration System (Language-Agnostic)
- Hierarchical config (System → User → Project → Language)
- Locking with audit trail
- Export/import for team sharing
- Schema validation per language
### 2. Safety Pipeline (Language-Agnostic)
- Git hooks for all languages
- Bypass system with mandatory reasons
- Unified audit log
- Works with any language plugin
### 3. Validation Engine (Per-Language)
Each plugin implements:
- **Pattern detection** — Language-specific anti-patterns
- **AST analysis** — Parse and validate code structure
- **Documentation coverage** — Ensure public APIs are documented
- **Security scanning** — Language-specific vulnerability checks
### 4. Documentation Compiler (Per-Language)
Each plugin generates:
- **API documentation** — Language-native formats (rustdoc, TSDoc, godoc, Sphinx)
- **AI context** — `llms.txt` for LLM agents
- **SSG integration** — Markdown/MDX for Docusaurus, Mintlify, VitePress
- **README sync** — Keep GitHub front page up to date
### 5. LSP Server (Unified)
One Language Server Protocol implementation that aggregates all language plugins:
- Real-time diagnostics
- Quick fixes across languages
- Go-to-definition across language boundaries
- Unified workspace symbols
## Success Metrics
### Technical
- ✅ One command validates entire codebase
- ✅ Config locking works across all languages
- ✅ Safety pipeline blocks bad commits in any language
- ✅ AI context generated for multi-language projects
### Adoption
- ✅ Teams use Forge for polyglot projects
- ✅ CI/CD pipelines use single Forge step
- ✅ AI agents work reliably across languages
### Quality
- ✅ Consistent standards across Rust/TS/Go/Python
- ✅ Documentation coverage tracked per-language
- ✅ Audit trail shows cross-language bypasses
## Implementation Roadmap
### Phase 1: Core (Current)
**Status:** ✅ Complete
- Rust plugin with full feature set
- Configuration system with locking
- Safety pipeline with git hooks
- Package manager distribution
### Phase 2: TypeScript Plugin (v2.0)
**ETA:** Q2 2026
- Integrate forge-ts capabilities
- ESLint/Biome integration
- TSDoc → OpenAPI generation
- Doctest support
- AI context (llms.txt) generation
### Phase 3: Go Plugin (v2.1)
**ETA:** Q3 2026
- Go modules integration
- golangci-lint support
- godoc generation
- gofmt enforcement
### Phase 4: Python Plugin (v2.2)
**ETA:** Q4 2026
- pip/poetry/uv support
- ruff/pylint integration
- docstring coverage
- black/isort formatting
### Phase 5: Unified Features (v3.0)
**ETA:** 2027
- Cross-language LSP server
- Unified documentation site
- Cross-language go-to-definition
- Multi-language refactoring
## Target Users
### Polyglot Teams
- "Our backend is Rust, frontend is TypeScript, ML is Python — one tool for all"
- Unified CI/CD pipeline
- Shared configuration standards
### Platform Engineers
- Set organization-wide standards across languages
- One audit trail for all code
- Simplified developer onboarding
### AI-First Teams
- Deterministic guardrails for LLM agents
- Consistent bypass audit trail
- AI context for all languages
### Open Source Maintainers
- One tool for multi-language projects
- Documentation generation for all contributors
- Standards enforcement in CI
## Philosophy
### Multi-Language > Single Language
Modern development is polyglot. Embrace it with unified tooling.
### Prevention > Detection
Catch issues at commit time, not CI time, in every language.
### Explicit > Implicit
No magic. Every override requires explicit command with reason.
### Documentation == Code
Documentation is a first-class deliverable, not an afterthought.
### AI-First
Built for the era where AI writes code. Provide clear boundaries and audit trails.
## Conclusion
**Forge** is not just another linter. It's a **unified quality platform** for the polyglot era.
One core. One config. One safety pipeline. Any language.
---
**Forge better code, automatically.** 🔨
*Rust core complete. TypeScript, Go, and Python plugins in development.*

# 🔨 Forge
**The Universal Multi-Language Development Standards Enforcer**
[![Crates.io](https://img.shields.io/crates/v/forge)](https://crates.io/crates/forge)
[![npm](https://img.shields.io/npm/v/@forge/cli)](https://www.npmjs.com/package/@forge/cli)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
> *"Forge better code, automatically — in any language."*
Forge is a **multi-language quality enforcement platform** with a Rust-based core. It provides unified standards enforcement across your entire stack — Rust, TypeScript, Go, Python, and more.
**Version:** 2.0.0 | **License:** MIT OR Apache-2.0
## 🎯 The Problem
Modern development is multi-language:
- Backend in **Rust** or **Go**
- Frontend in **TypeScript**
- Data processing in **Python**
- All in the same monorepo
But quality tools are fragmented — ESLint for TypeScript, Clippy for Rust, golangci-lint for Go, pylint for Python... Each with different configs, different behaviors, different escape hatches.
**Forge unifies them into one tool.**
## 🚀 Quick Start
```bash
# Install Forge (Rust-based, works for all languages)
cargo install forge
# Initialize in your project
forge init --project
# Auto-detects: Rust, TypeScript, Go, Python
# Validate everything
forge validate
# 🦀 Rust: 0 violations
# ⚡ TypeScript: 0 violations
# 🐹 Go: 0 violations
# 🐍 Python: 0 violations
# That's it! Git commits now validated across all languages
git commit -m "feat: add feature"
```
## ✨ Features
### 🔍 Language Detection
Forge automatically detects which languages are present:
```bash
$ forge init --project
🔍 Detected languages:
   ✅ Rust (Cargo.toml found)
   ✅ TypeScript (package.json found)
   ❌ Go (go.mod not found)
   ✅ Python (pyproject.toml found)
🛡️ Installing safety pipeline for: Rust, TypeScript, Python...
```
### 🔒 Unified Configuration Locking
Lock standards across all languages:
```bash
# Lock "edition" concept (applies to all languages)
$ forge config lock required_edition --reason="Team decision"
🔒 Locked:
  ✅ Rust: edition = "2024"
  ✅ TypeScript: target = "ES2022"
  ✅ Python: python_version = "3.12"
# Try to change it (blocked!)
$ cat Cargo.toml | grep edition
edition = "2021"
$ forge validate .
❌ Violation: edition is locked to "2024"
   Fix: forge config unlock required_edition --reason="..."
```
### 🛡️ Multi-Language Safety Pipeline
One safety pipeline validates everything:
```bash
$ git commit -m "feat: add feature"
🛡️ Forge Safety Pipeline
   Rust:      ✅ Pass
   TypeScript: ❌ Fail (1 violation)
   Python:    ✅ Pass
❌ Commit blocked:
   TypeScript: Missing TSDoc on public export
     → src/api.ts:42
# Emergency bypass (applies to all languages)
$ forge safety bypass --stage=pre-commit --reason="WIP"
✅ Bypass active for: Rust, TypeScript, Python
```
### 📚 Documentation Generation
Generate docs for all languages:
```bash
$ forge doc build
🦀 Rust:      Generating rustdoc...
⚡ TypeScript: Generating TSDoc + OpenAPI...
🐹 Go:        Generating godoc...
🐍 Python:    Generating Sphinx docs...
✅ Generated:
  📄 target/doc/ (Rust API)
  📄 docs/api/openapi.json (TypeScript API)
  📄 docs/generated/llms.txt (AI context)
  🌐 http://localhost:3000 (unified docs)
```
## 🎯 CLI Reference
### Core Commands
| Command | Description |
|---------|-------------|
| `forge init [--project]` | Initialize system or project |
| `forge validate [path]` | Validate all detected languages |
| `forge fix [path]` | Auto-fix violations |
| `forge config` | Manage configuration |
| `forge safety` | Safety pipeline management |
| `forge doc` | Documentation commands |
### Per-Language Commands
| Command | Description |
|---------|-------------|
| `forge rust` | Rust toolchain management |
| `forge typescript` | TypeScript/Node.js management |
| `forge go` | Go toolchain management |
| `forge python` | Python toolchain management |
## ⚙️ Configuration
One config file for all languages:
```toml
# .forge/config.toml
[validation]
max_file_lines = 500
require_documentation = true
[language.rust]
edition = "2024"
lint_groups = ["all", "pedantic"]
[language.typescript]
target = "ES2022"
strict = true
linter = "biome"
[language.go]
go_version = "1.23"
[language.python]
python_version = "3.12"
linter = "ruff"
```
## 🛡️ What Gets Enforced
### By Language
**Rust:**
- Edition compliance
- Unsafe code prevention
- Clippy lint groups
- Documentation coverage
- Security audits
**TypeScript:**
- Strict mode enforcement
- TSDoc coverage
- OpenAPI spec generation
- Doctest execution
- Biome/ESLint rules
**Go:**
- Go version compliance
- golint rules
- Godoc coverage
- Module vendoring
**Python:**
- Python version compliance
- Type hint enforcement
- Docstring coverage
- ruff/pylint rules
### Git Hooks (Blocking)
```bash
# Pre-commit checks ALL languages
$ git commit -m "feat: add feature"
🛡️ Forge Safety Pipeline
   Running checks for: Rust, TypeScript, Python
   
   [####################] 100%
   
   Rust:       ✅ Pass (12 checks)
   TypeScript: ✅ Pass (8 checks)
   Python:     ❌ Fail (2 violations)
     - Line too long: src/utils.py:15
     - Missing docstring: src/models.py:42
❌ Commit blocked. Fix violations or bypass with: forge safety bypass
```
## 📦 Installation
### From Cargo (Rust)
```bash
cargo install forge
```
### From npm (TypeScript projects)
```bash
npm install -D @forge/cli
```
### Package Managers
```bash
# Homebrew
brew install forge
# Arch Linux
yay -S forge
# Nix
nix-env -iA forge
```
## 🚫 Blocked Patterns
**Rust:**
```rust
// ❌ Edition downgrade
edition = "2021"  // Locked to "2024"
// ❌ Unsafe code
unsafe fn dangerous() {}  // unsafe_forbidden = true
// ❌ Underscore bandaid
fn bad(_unused: String) {}
```
**TypeScript:**
```typescript
// ❌ Non-strict mode
// tsconfig.json: strict = false  // Locked to true
// ❌ Missing TSDoc
export function api() {}  // Public exports require documentation
// ❌ Any type
function process(data: any) {}  // avoid any when possible
```
**Go:**
```go
// ❌ Wrong Go version
// go.mod: go 1.22  // Locked to 1.23
// ❌ Missing godoc
func Process() {}  // Public functions require documentation
```
**Python:**
```python
# ❌ Missing type hints
def process(data):  # Type hints required
    pass
# ❌ Missing docstring
class Processor:  # Public classes require docstrings
    pass
```
## 🔧 Supported Languages
| Language | Status | Features |
|----------|--------|----------|
| **Rust** | ✅ Stable | Full enforcement, edition mgmt, rustdoc |
| **TypeScript** | 🚧 Beta | ESLint/Biome, TSDoc, OpenAPI, doctests |
| **Go** | 📋 Planned | golangci-lint, godoc, gofmt |
| **Python** | 📋 Planned | ruff, Sphinx, type hints |
## 🏗️ Architecture
```
┌─────────────────────────────────────┐
│         FORGE CORE (Rust)           │
├─────────────────────────────────────┤
│  Config • Safety • CLI • LSP        │
└──────────┬──────────────────────────┘
           │ Plugin API
   ┌───────┼───────┐
   ▼       ▼       ▼
┌──────┐ ┌──────┐ ┌──────┐
│ Rust │ │  TS  │ │  Go  │
│Plugin│ │Plugin│ │Plugin│
└──────┘ └──────┘ └──────┘
```
## 📖 Documentation
- [**VISION.md**](VISION.md) — Core vision and philosophy
- [**ROADMAP.md**](ROADMAP.md) — Development roadmap
- [**PLUGIN_GUIDE.md**](PLUGIN_GUIDE.md) — Building custom plugins
## 🤝 Contributing
We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md).
**Priority Areas:**
1. TypeScript plugin completion
2. Go plugin development
3. Python plugin development
4. Custom plugin API
## 📄 License
Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
## Migration from Ferrous Forge
If you're using Ferrous Forge 1.x:
```bash
# Install new Forge
cargo install forge
# Migrate config
mv ~/.config/ferrous-forge ~/.config/forge
# Update hooks (run in each project)
forge safety install --force
# Done! All existing functionality preserved
```
---
**Forge better code, automatically.** 🔨
*One core. One config. Any language.*

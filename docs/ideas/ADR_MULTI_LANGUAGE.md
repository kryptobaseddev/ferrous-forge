# ADR-004: Multi-Language Plugin Architecture

**Status:** Accepted  
**Date:** 2025-03-21  
**Deciders:** @kryptobaseddev  

## Context

Forge currently provides aggressive enforcement for Rust. However, modern development is inherently multi-language:

- Backend services in **Rust** or **Go**
- Frontend in **TypeScript** (React, Vue, etc.)
- Data processing in **Python**
- All living in the same monorepo

Teams currently juggle 4+ different quality toolchains (ESLint, Clippy, golangci-lint, pylint), each with different configs and behaviors.

**We will unify these into a single Forge platform with language plugins.**

## Decision

We will architect Forge as a **language-agnostic core with pluggable language support**.

**Core principle:** One binary, one config, one safety pipeline — multiple languages.

## Architecture

### Core (Rust-Based, Language-Agnostic)

The following components work for any language:

1. **Configuration System**
   - Hierarchical config (System → User → Project)
   - Locking with audit trail
   - Export/import for team sharing
   - Per-language config sections

2. **Safety Pipeline**
   - Git hook installation and management
   - Bypass system with mandatory reasons
   - Unified audit log across all languages
   - Stage-based execution (pre-commit, pre-push, pre-publish)

3. **CLI Interface**
   - Single binary: `forge`
   - Auto-detects project languages
   - Unified output format
   - `--json` for machine-readable output

4. **LSP Server**
   - Aggregates all language plugins
   - Cross-language workspace symbols
   - Unified diagnostics

### Language Plugin System

Each language provides a plugin implementing the `LanguagePlugin` trait:

```rust
#[async_trait]
pub trait LanguagePlugin: Send + Sync {
    /// Language identifier (e.g., "rust", "typescript", "go", "python")
    fn name(&self) -> &str;
    
    /// Detect if this plugin applies to the current project
    /// Checks for Cargo.toml, package.json, go.mod, etc.
    fn detect(&self, project_path: &Path) -> DetectionResult;
    
    /// Get available validation rules for this language
    fn rules(&self) -> Vec<Box<dyn ValidationRule>>;
    
    /// Run the language's linter
    async fn lint(&self, options: &LintOptions) -> Result<LintResult>;
    
    /// Run the language's formatter
    async fn format(&self, options: &FormatOptions) -> Result<FormatResult>;
    
    /// Check documentation coverage
    async fn doc_check(&self) -> Result<DocCoverage>;
    
    /// Generate documentation
    async fn doc_build(&self, options: &DocOptions) -> Result<DocOutput>;
    
    /// List available toolchains/versions
    async fn list_toolchains(&self) -> Result<Vec<Toolchain>>;
    
    /// Install a specific toolchain/version
    async fn install_toolchain(&self, version: &str) -> Result<()>;
    
    /// Get AI context (llms.txt content) for this language
    async fn ai_context(&self) -> Result<String>;
}

pub struct DetectionResult {
    pub detected: bool,
    pub confidence: f32,  // 0.0 to 1.0
    pub manifest_files: Vec<PathBuf>,  // Cargo.toml, package.json, etc.
}
```

### Plugin Registry

Plugins are loaded dynamically at runtime:

```rust
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn LanguagePlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            plugins: HashMap::new(),
        };
        
        // Register built-in plugins
        registry.register(Box::new(RustPlugin::new()));
        registry.register(Box::new(TypeScriptPlugin::new()));
        registry.register(Box::new(GoPlugin::new()));
        registry.register(Box::new(PythonPlugin::new()));
        
        registry
    }
    
    pub fn detect_languages(&self, project_path: &Path
    ) -> Vec<&dyn LanguagePlugin> {
        self.plugins.values()
            .filter(|p| p.detect(project_path).detected)
            .map(|p| p.as_ref())
            .collect()
    }
}
```

### Language-Specific Implementations

#### Rust Plugin (Built-in)

**Tools integrated:**
- cargo (build, test)
- clippy (linting)
- rustfmt (formatting)
- rustdoc (documentation)
- rustup (toolchain management)

**Capabilities:**
- Edition management (2021, 2024)
- Unsafe code detection
- Documentation coverage
- Security audit (cargo-audit)

**Status:** ✅ Complete

#### TypeScript Plugin (v2.0)

**Tools integrated:**
- npm/pnpm/yarn (package management)
- ESLint or Biome (linting)
- Prettier or Biome (formatting)
- TypeScript compiler API (AST)
- @microsoft/tsdoc (TSDoc parsing)

**Capabilities:**
- TypeScript target enforcement (ES2022, etc.)
- TSDoc coverage checking
- OpenAPI 3.2 spec generation
- Doctest execution (@example blocks)
- llms.txt generation
- README synchronization

**Integration with forge-ts:**
The existing forge-ts project will be refactored as the TypeScript plugin for Forge:
- Core AST traversal logic becomes `TypeScriptPlugin`
- CLI commands map to `forge` subcommands
- Config system unified with Forge's hierarchical config

**Status:** 🚧 In Development

#### Go Plugin (v2.1)

**Tools integrated:**
- go modules (dependency management)
- golangci-lint (linting)
- gofmt (formatting)
- godoc (documentation)

**Capabilities:**
- Go version enforcement
- Godoc coverage checking
- Module vendoring checks
- Security scanning

**Status:** 📋 Planned

#### Python Plugin (v2.2)

**Tools integrated:**
- pip/poetry/uv (package management)
- ruff or pylint (linting)
- black or ruff (formatting)
- Sphinx or MkDocs (documentation)

**Capabilities:**
- Python version enforcement
- Docstring coverage (Google/NumPy/reST style)
- Type hint checking
- Import sorting

**Status:** 📋 Planned

### Configuration Structure

```toml
# ~/.config/forge/config.toml (User defaults)
[validation]
max_file_lines = 500
require_documentation = true

# Language-specific sections
[language.rust]
edition = "2024"
allow_unsafe = false
lint_groups = ["all", "pedantic", "nursery"]

[language.typescript]
target = "ES2022"
strict = true
linter = "biome"  # or "eslint"
formatter = "biome"  # or "prettier"

[language.go]
go_version = "1.23"
linter = "golangci-lint"

[language.python]
python_version = "3.12"
linter = "ruff"
formatter = "black"
docstyle = "google"  # or "numpy", "reST"
```

### Cross-Language Locking

Locking a concept can apply to multiple languages:

```rust
// forge config lock required_edition
pub struct CrossLanguageLock {
    concept: String,  // "required_edition"
    applies_to: Vec<String>,  // ["rust", "typescript", "go"]
    locked_values: HashMap<String, String>,
    // "rust" → "2024"
    // "typescript" → "ES2022"
    // "go" → "1.23"
}
```

## Implementation Phases

### Phase 1: Core Refactoring (v2.0-alpha)
- Extract language-agnostic core
- Define `LanguagePlugin` trait
- Refactor Rust support as built-in plugin
- No breaking changes to existing Rust users

### Phase 2: TypeScript Integration (v2.0)
- Port forge-ts as TypeScript plugin
- Integrate TSDoc → OpenAPI generation
- Add doctest support for TypeScript
- Implement llms.txt generation

### Phase 3: Go Support (v2.1)
- Implement GoPlugin
- golangci-lint integration
- Godoc coverage checking

### Phase 4: Python Support (v2.2)
- Implement PythonPlugin
- ruff/pylint integration
- Docstring coverage

### Phase 5: Advanced Features (v3.0)
- Cross-language LSP server
- Unified documentation site
- Cross-language refactoring
- Custom plugin API for community languages

## Integration with Existing Projects

### Ferrous Forge (Rust)
Becomes the "Rust Plugin" within the unified Forge platform.

**Migration:**
- Binary name: `ferrous-forge` → `forge`
- Config path: `~/.config/ferrous-forge/` → `~/.config/forge/`
- All existing functionality preserved

### Forge-ts (TypeScript)
Refactored as the "TypeScript Plugin".

**Migration:**
- Commands: `forge-ts check` → `forge validate --language=typescript`
- Config: `forge-ts.config.ts` → `forge.toml` [language.typescript] section
- All existing functionality preserved

## Alternatives Considered

| Option | Pros | Cons |
|--------|------|------|
| **Unified Core + Plugins** (chosen) | One tool for entire stack, shared config, consistent UX | Complex core, maintenance burden |
| **Separate Tools** | Each tool optimized for language | Fragmented, no shared config |
| **Wrapper Scripts** | Simple implementation | Inconsistent behavior, limited integration |
| **Language-Specific Cores** | Native performance per language | Duplicated effort, divergent features |

## Consequences

**Positive:**
- Teams use one tool across entire stack
- Shared configuration and safety pipeline
- Consistent UX across languages
- Easier for platform teams to mandate
- AI agents get consistent guardrails

**Negative:**
- Increased core complexity
- Risk of "jack of all trades, master of none"
- Slower releases as we support more languages
- Breaking changes for existing Ferrous Forge users

**Mitigations:**
- Rust remains the primary focus
- Plugins can be developed independently
- Strict quality gates for new language support
- Clear migration path with backwards compatibility

## References

- See [VISION: Multi-Language Support](../VISION.md)
- Related: forge-ts project (`/mnt/projects/forge-ts/`)
- Plugin trait definition: `src/plugin.rs`

## Notes

This architecture enables Forge to become the "universal quality platform" rather than just a Rust tool. The core investment in Rust gives us performance and safety, while the plugin system allows us to support any language.

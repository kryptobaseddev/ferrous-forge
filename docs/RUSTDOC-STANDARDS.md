# Rustdoc Standards — LLM Agent Reference

> **Canonical guide** for documentation architecture in any Rust project.
> Use this as a bible: comment syntax, enforcement layers, doc tests, CI pipeline,
> and the two-layer documentation system (development docs → generated docs).

---

## The Two-Layer Documentation System

This enforces a strict separation between **development documentation** and **generated documentation**. They serve different audiences, live in different places, and are written at different times.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  LAYER 1: DEVELOPMENT DOCS  (docs/dev/)                                     │
│  Audience: Engineers, AI agents, architects                                  │
│  Written: BEFORE implementation — drives decisions and design                │
│  Authored: By humans. ADRs require human sign-off.                          │
│                                                                              │
│  docs/dev/                                                                   │
│  ├── adr/          Architecture Decision Records — why decisions were made   │
│  └── specs/        Technical specifications — what to build and how         │
└────────────────────────────┬────────────────────────────────────────────────┘
                             │  Specs define interfaces/behavior
                             │  ADRs define constraints and rationale
                             ▼
                      IMPLEMENTATION
                      src/**/*.rs
                      (code + /// inline docs)
                             │
                             │  cargo doc + cargo-rdme + mdBook
                             ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  LAYER 2: GENERATED DOCS  (book/ + target/doc/)                             │
│  Audience: End users, integrators, maintainers                              │
│  Written: NEVER by hand — auto-generated from code                          │
│  Source of truth: /// comments in src/                                      │
│                                                                              │
│  book/                        target/doc/                                    │
│  ├── guides/                  └── <crate>/         ← cargo doc output       │
│  └── API ref link ────────────────────────────────────────────────────────► │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Hard rules:**
1. `docs/dev/` is NEVER published to end users. It is the engineering source layer.
2. `book/` and `target/doc/` are NEVER hand-edited. They are generated outputs.
3. `README.md` is NEVER hand-edited. It is synced from `lib.rs` `//!` by `cargo-rdme`.
4. Every significant architectural decision MUST have an ADR before implementation begins.
5. Every non-trivial module or crate-level design MUST reference its ADR in the `//!` doc.

---

## Layer 1: Development Docs (`docs/dev/`)

### Why This Folder Exists

`docs/dev/` answers questions that code cannot: *Why was this decision made? What alternatives were rejected? What are the constraints?* These are the questions LLM agents and new engineers ask when encountering unfamiliar code. Without this layer, decisions are made and then lost.

ADRs and Specs are the **contract** between planning and implementation. Code that diverges from a Spec without a superseding ADR is a bug in the process.

### Folder Structure

```
docs/dev/
├── adr/
│   ├── README.md               ← ADR index (maintained or script-generated)
│   ├── 001-transport-protocol.md
│   ├── 002-database-backend.md
│   └── 003-auth-token-format.md
└── specs/
    ├── message-envelope.md
    ├── delivery-guarantees.md
    └── agent-identity.md
```

### ADR Format (Canonical)

Filename: `docs/dev/adr/NNN-short-kebab-title.md` — `NNN` is zero-padded and sequential.

```markdown
# ADR-NNN: Title

**Status:** Proposed | Accepted | Deprecated | Superseded by [ADR-NNN](./NNN-title.md)
**Date:** YYYY-MM-DD
**Deciders:** @handle, @handle

## Context

What situation or problem forced this decision.
What constraints existed (performance, compatibility, team skill, time).
What was the state of the system before this decision.

## Decision

What was decided. Be specific and definitive.

## Alternatives Considered

| Option | Reason Rejected |
|--------|----------------|
| Option A | ... |
| Option B | ... |

## Consequences

**Positive:** What becomes easier.
**Negative:** What becomes harder or what debt is accepted.
**Neutral:** What changes without clear value judgment.

## Implementation Notes

Which modules, crates, or interfaces are affected.
Link to the Spec if one exists.
```

### Spec Format (Canonical)

Filename: `docs/dev/specs/kebab-title.md`

```markdown
# Spec: Title

**Status:** Draft | Active | Superseded
**Related ADR:** [ADR-NNN](../adr/NNN-title.md)
**Last Updated:** YYYY-MM-DD

## Overview

One paragraph: what this spec defines and why it exists.

## Interface / Behavior

Precise definition of the contract: inputs, outputs, invariants, error conditions.
Use diagrams, tables, and code examples where needed.

## Edge Cases and Constraints

What is explicitly out of scope.
What boundary conditions must be handled.

## Acceptance Criteria

Testable conditions that confirm correct implementation.
These map directly to doc tests and integration tests.
```

### The ADR → Code Link (Non-Optional)

Every module or crate whose design was driven by an ADR MUST reference that ADR in its `//!` doc. Without it, the ADR and the code drift apart silently within months.

```rust
// src/transport/mod.rs
//! LAFS-compliant message transport layer.
//!
//! Supports SSE, webhook, and polling delivery modes with automatic fallback.
//!
//! # Design
//!
//! Architecture defined in [ADR-001: Transport Protocol](../../docs/dev/adr/001-transport-protocol.md).
//! Delivery guarantee semantics specified in [Spec: Delivery Guarantees](../../docs/dev/specs/delivery-guarantees.md).
```

---

## Layer 2: Generated Docs

### What Generates What

| Output | Source | Tool | Command |
|--------|--------|------|---------|
| `target/doc/` (API reference) | `///` and `//!` in `src/` | rustdoc | `cargo doc --no-deps` |
| `README.md` | `//!` in `lib.rs` | cargo-rdme | `cargo rdme` |
| `book/` (guides + API link) | `docs/src/` markdown | mdBook | `mdbook build` |

`docs/dev/` is deliberately NOT included in the mdBook `src/`. Development docs are for engineers, not end users.

---

## 1. Comment Syntax

| Syntax | Use | Location |
|--------|-----|----------|
| `//!` | Document the containing item | Top of `lib.rs`, `mod.rs`, inline modules |
| `///` | Document the item that follows | Functions, structs, enums, traits, fields, consts |

```rust
// lib.rs
//! Fast LAFS-compliant message delivery.
//!
//! Provides agents with SSE, webhook, and polling transport with automatic fallback.
//!
//! # Quick Start
//!
//! ```rust
//! let client = signaldock::Client::connect("ws://localhost:9000")?;
//! client.send("agent-b", "hello").await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Design
//!
//! See [ADR-001](../docs/dev/adr/001-transport-protocol.md).

/// Returns the decoded payload from a signed LAFS envelope.
///
/// Verifies HMAC-SHA256 before deserializing.
///
/// # Errors
///
/// - [`Error::InvalidSignature`] — HMAC verification failed
/// - [`Error::Malformed`] — deserialization failed
///
/// # Examples
///
/// ```rust
/// # use my_crate::decode_envelope;
/// let bytes = include_bytes!("../tests/fixtures/valid-envelope.bin");
/// let payload = decode_envelope(bytes)?;
/// assert_eq!(payload.version, 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decode_envelope(bytes: &[u8]) -> Result<Payload, Error> { ... }
```

**Formatting rules:**
- First line = one sentence, no period required, shown in search indexes.
- Blank `///` line between sections.
- All types, functions, fields in backticks or linked: `` [`MyType`] ``
- Intra-doc links (`[`Type`]`) always preferred over raw URLs for same-crate items.
- Hide example boilerplate from rendered output with `# ` prefix — still compiled and run.

---

## 2. Required Sections by Item

| Item | Always Required | Conditionally Required |
|------|----------------|----------------------|
| Every `pub` item | One-sentence summary | — |
| `pub fn` | `# Examples` | — |
| `pub fn -> Result<_, E>` | `# Examples` | `# Errors` (list each error variant) |
| `pub fn` that can panic | `# Examples` | `# Panics` (describe the condition) |
| `pub unsafe fn` | `# Examples`, `# Safety` | — (`# Safety` is deny-level in Clippy) |
| `pub struct` / `pub enum` | Summary + each pub field/variant doc | — |
| `pub trait` | Summary + each method doc | — |
| Crate root `lib.rs` | `//!` summary | `//!` quick-start example, `# Design` ADR link |
| Module root `mod.rs` | `//!` purpose | `# Design` ADR link if non-trivial |

### Section Examples

```rust
/// # Safety
///
/// `ptr` must be non-null, properly aligned for `T`, and valid for reads.
/// The memory must not be mutated for the duration of the returned reference.
pub unsafe fn as_ref_unchecked<T>(ptr: *const T) -> &'static T { ... }

/// # Errors
///
/// Returns [`Error::Timeout`] if the upstream did not respond within the configured deadline.
/// Returns [`Error::Unauthorized`] if the token is expired or invalid.
pub async fn fetch(&self, id: &str) -> Result<Agent, Error> { ... }

/// # Panics
///
/// Panics if `capacity` is zero.
pub fn new(capacity: usize) -> Self { ... }
```

---

## 3. Enforcement — Compile Time (Layer A)

These fire on every `cargo build` and `cargo check`. **The build fails if violated.**

**Preferred — `Cargo.toml` (Rust 1.73+, no need to touch source files):**

```toml
[lints.rust]
missing_docs = "warn"                      # every pub item needs ///

[lints.rustdoc]
broken_intra_doc_links  = "deny"           # [`BadLink`] must resolve — hard error
invalid_html_tags       = "deny"           # no malformed HTML in docs
missing_crate_level_docs = "warn"          # lib.rs needs //!
bare_urls               = "warn"           # wrap URLs: <https://...>
redundant_explicit_links = "warn"          # [`usize`] not [`usize`](usize)
unescaped_backticks     = "warn"           # broken inline code from mismatched backticks
```

**Alternative — `src/lib.rs` attributes (use if `Cargo.toml` lints aren't available):**

```rust
// src/lib.rs — add at the top
#![warn(missing_docs)]                           // every pub item needs ///
#![warn(rustdoc::missing_crate_level_docs)]      // lib.rs needs //!
#![deny(rustdoc::broken_intra_doc_links)]        // [`BadLink`] must resolve
#![deny(rustdoc::invalid_html_tags)]             // no malformed HTML in docs
#![warn(rustdoc::bare_urls)]                     // wrap URLs: <https://...>
#![warn(rustdoc::redundant_explicit_links)]      // [`usize`] not [`usize`](usize)
```

### All Rustdoc Lints Reference

| Lint | Default | What it catches |
|------|---------|----------------|
| `missing_docs` | allow | Public items with no doc comment |
| `rustdoc::missing_crate_level_docs` | allow | No `//!` at crate root |
| `rustdoc::missing_doc_code_examples` | allow (nightly only) | Doc blocks without code examples |
| `rustdoc::broken_intra_doc_links` | warn | Unresolvable `[`links`]` |
| `rustdoc::private_intra_doc_links` | warn | Public doc links to private items |
| `rustdoc::invalid_codeblock_attributes` | warn | Typos like `should-panic` vs `should_panic` |
| `rustdoc::invalid_html_tags` | warn | Unclosed/malformed HTML in docs |
| `rustdoc::invalid_rust_codeblocks` | warn | Empty or unparseable Rust code blocks |
| `rustdoc::bare_urls` | warn | Raw URLs not wrapped in `<...>` |
| `rustdoc::redundant_explicit_links` | warn | `` [`usize`](usize) `` is redundant — use `` [`usize`] `` |
| `rustdoc::unescaped_backticks` | allow | Broken inline code from mismatched backticks |
| `rustdoc::private_doc_tests` | allow | Doc tests on private items |

---

## 4. Enforcement — Clippy Doc Lints (Layer B)

Clippy catches structural doc problems rustc misses. Fires on `cargo clippy`.

**`Cargo.toml`:**

```toml
[lints.clippy]
missing_safety_doc           = "deny"      # unsafe fn with no # Safety = hard error
missing_errors_doc           = "warn"      # Result fn with no # Errors
missing_panics_doc           = "warn"      # panicking fn with no # Panics
empty_docs                   = "warn"      # /// with no content
doc_markdown                 = "warn"      # MyType outside backticks
needless_doctest_main        = "warn"      # fn main() {} in examples is noise
suspicious_doc_comments      = "warn"      # /// used where //! was intended
too_long_first_doc_paragraph = "warn"      # overly long summary line
```

### Full Clippy Doc Lints Reference

| Lint | Group | Catches |
|------|-------|---------|
| `clippy::missing_safety_doc` | style | `pub unsafe fn` without `# Safety` |
| `clippy::missing_errors_doc` | pedantic | `-> Result` without `# Errors` |
| `clippy::missing_panics_doc` | pedantic | May-panic fn without `# Panics` |
| `clippy::empty_docs` | suspicious | `///` with no text |
| `clippy::doc_markdown` | pedantic | Bare identifiers/types not in backticks |
| `clippy::doc_broken_link` | pedantic | Dead hyperlinks in doc comments |
| `clippy::doc_link_with_quotes` | pedantic | `["link"]` typo for intra-doc link |
| `clippy::needless_doctest_main` | style | Unnecessary `fn main()` wrapper in examples |
| `clippy::too_long_first_doc_paragraph` | style | Overly long summary line |
| `clippy::suspicious_doc_comments` | suspicious | Outer `///` used where `//!` belongs |
| `clippy::doc_lazy_continuation` | style | Unindented paragraph continuation lines |
| `clippy::doc_paragraphs_missing_punctuation` | restriction | Sentences without terminal punctuation |
| `clippy::doc_nested_refdefs` | suspicious | Link reference defined inside list or quote |
| `clippy::doc_overindented_list_items` | style | List items indented too far |

---

## 5. Enforcement — LSP Real-Time (Layer C)

Configure rust-analyzer to run Clippy on every save so violations appear as editor squiggles **before commit**. Any LLM agent with LSP access (Cursor, Zed, VS Code, Neovim) will see doc violations inline without needing to be reminded of standards.

**Commit `rust-analyzer.toml` to the repo root** — applies to all developers and agents:

```toml
# rust-analyzer.toml  (project root, committed to repo)
[check]
command = "clippy"
extra_args = [
  "--",
  "-D", "clippy::missing_safety_doc",
  "-W", "clippy::missing_errors_doc",
  "-W", "clippy::missing_panics_doc",
  "-W", "clippy::empty_docs",
  "-W", "clippy::doc_markdown",
]

[diagnostics]
styleLints.enable = true
```

**Also commit `.vscode/settings.json`** for VS Code / Cursor users:

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": [
    "--",
    "-W", "clippy::missing_safety_doc",
    "-W", "clippy::missing_errors_doc",
    "-W", "clippy::missing_panics_doc",
    "-W", "clippy::empty_docs",
    "-W", "clippy::doc_markdown"
  ],
  "rust-analyzer.diagnostics.styleLints.enable": true
}
```

---

## 6. Enforcement — Doc Coverage Threshold (Layer D)

Track what percentage of public items have documentation. Nightly only.

```bash
# Human-readable table
RUSTDOCFLAGS="-Z unstable-options --show-coverage" \
  cargo +nightly doc --no-deps 2>&1
```

Output:
```
+------------------+------------+------------+------------+------------+
| File             | Documented | Percentage |   Examples | Percentage |
+------------------+------------+------------+------------+------------+
| src/lib.rs       |         42 |      87.5% |         10 |      23.8% |
| src/transport.rs |         18 |     100.0% |          5 |      27.8% |
+------------------+------------+------------+------------+------------+
| Total            |         60 |      90.9% |         15 |      25.0% |
+------------------+------------+------------+------------+------------+
```

**Add to CI to fail below a threshold:**

```bash
# Requires nightly + jq
RUSTDOCFLAGS="-Z unstable-options --show-coverage --output-format json" \
  cargo +nightly doc --no-deps -q 2>&1 \
  | jq 'to_entries[] | select(.key | endswith(".rs")) | .value | .with_docs / .total' \
  | awk '{if ($1 < 0.90) { print "Doc coverage below 90%"; exit 1 }}'
```

---

## 7. Doc Tests — Examples Are Tests

Every `# Examples` block is compiled and run by `cargo test --doc`. A broken example fails CI.

### Code Block Annotations

````rust
/// ```rust
/// // Standard: compiled and run. Must pass.
/// assert_eq!(2 + 2, 4);
/// ```
///
/// ```rust,no_run
/// // Compiled but not run. Use for network I/O, filesystem, async.
/// fetch_agent("cleo").await?;
/// ```
///
/// ```rust,compile_fail
/// // Must NOT compile. Use to document type errors.
/// let x: u32 = "not a number";
/// ```
///
/// ```rust,should_panic
/// // Must panic at runtime.
/// panic!("expected failure");
/// ```
///
/// ```rust,ignore
/// // Not compiled at all. Last resort — hides broken examples. Use sparingly.
/// ```
````

### Hiding Boilerplate with `# `

Lines prefixed with `# ` are compiled and run but hidden from rendered documentation output. Use this to hide setup code that would clutter the example:

```rust
/// ```rust
/// # use my_crate::Client;             // hidden: import not shown in docs
/// # let rt = tokio::runtime::Runtime::new().unwrap();
/// # rt.block_on(async {
/// let client = Client::new("https://api.example.com");
/// let agent = client.fetch("cleo").await?;
/// println!("Got agent: {}", agent.name);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
```

The user sees only:
```rust
let client = Client::new("https://api.example.com");
let agent = client.fetch("cleo").await?;
println!("Got agent: {}", agent.name);
```

**Run doc tests:**
```bash
cargo test --doc
cargo test --doc -- --nocapture    # show println! output for debugging
```

---

## 8. Auto-Sync README (cargo-rdme)

`README.md` is generated from `lib.rs` `//!` comments. **Never edit it by hand.**

```bash
cargo install cargo-rdme --locked
cargo rdme           # sync README now
cargo rdme --check   # CI: exit non-zero if README is out of sync
```

**`README.md`** — place this marker where generated content should appear:
```markdown
# My Crate

<!-- cargo-rdme start -->
...auto-generated from lib.rs //! comments...
<!-- cargo-rdme end -->

## License
...
```

**`lib.rs`** — whatever is in `//!` becomes the README body:
```rust
//! Fast LAFS-compliant message delivery.
//!
//! # Quick Start
//!
//! ```rust
//! let client = signaldock::Client::connect("ws://localhost:9000")?;
//! client.send("agent-b", "hello").await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
```

**`.cargo-rdme.toml`** (optional config):
```toml
[readme]
path = "README.md"

[intralinks]
docs-rs-base-url = "https://docs.rs"
```

---

## 9. mdBook — Prose Documentation Site

mdBook builds end-user and maintainer documentation. It links to the rustdoc API reference but does not replace it. `docs/dev/` is NOT included — development docs are for engineers only.

```
docs/
├── book.toml
└── src/
    ├── SUMMARY.md
    ├── guides/          ← how-to guides (human-authored)
    │   └── agent-integration.md
    └── api.md           ← links to target/doc/ (rustdoc output)
```

**`book.toml`:**
```toml
[book]
title = "My Crate"
src = "src"

[output.html]
git-repository-url = "https://github.com/your-org/your-crate"

[preprocessor.mermaid]
command = "mdbook-mermaid"

[preprocessor.toc]
command = "mdbook-toc"

[output.linkcheck]
warning-policy = "error"          # broken links = build failure
exclude = ["https://crates.io/.*"]
```

### mdBook Preprocessors

| Tool | Install | Enforces |
|------|---------|---------|
| `mdbook-linkcheck` | `cargo install mdbook-linkcheck` | Broken links fail build |
| `mdbook-mermaid` | `cargo install mdbook-mermaid` | Render architecture diagrams inline |
| `mdbook-toc` | `cargo install mdbook-toc` | Auto table of contents in pages |

**Install and build:**
```bash
cargo install mdbook mdbook-linkcheck mdbook-mermaid mdbook-toc --locked
cargo doc --no-deps                  # generate API ref first
cp -r target/doc/ docs/src/api/      # make rustdoc output available to mdBook
cd docs && mdbook build
```

---

## 10. Full CI Pipeline

```yaml
# .github/workflows/docs.yml
name: Documentation
on: [push, pull_request]

jobs:
  # Layer A + B + C enforcement
  lint-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      # Layer A: rustc missing_docs via cargo check
      - name: cargo check (missing_docs)
        run: cargo check

      # Layer B: clippy doc lints
      - name: cargo clippy (doc lints)
        run: |
          cargo clippy -- \
            -D clippy::missing_safety_doc \
            -W clippy::missing_errors_doc \
            -W clippy::missing_panics_doc \
            -W clippy::empty_docs \
            -W clippy::doc_markdown

      # Layer A: rustdoc link and tag validation
      - name: cargo doc (broken links + html)
        run: |
          RUSTDOCFLAGS="\
            -D rustdoc::broken_intra_doc_links \
            -D rustdoc::invalid_html_tags \
            -W rustdoc::bare_urls" \
          cargo doc --no-deps --all-features

      # Doc tests: examples must compile and pass
      - name: cargo test --doc
        run: cargo test --doc

      # README sync: fail if out of sync with lib.rs
      - name: Check README is in sync
        run: |
          cargo install cargo-rdme --locked
          cargo rdme --check

  # Build mdBook prose docs
  build-book:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install mdbook tools
        run: |
          cargo install mdbook --locked
          cargo install mdbook-linkcheck --locked

      - name: Build prose docs
        run: cd docs && mdbook build

      # linkcheck runs as part of mdbook build via [output.linkcheck]
      - name: Verify link check ran
        run: echo "Link check completed as part of mdbook build above"
```

---

## 11. LLM Agent Decision Reference

When writing Rust code, apply this checklist before completing any `pub` item:

```
Is this item pub?
  YES →
    [ ] Does it have ///?              → ADD IT — missing_docs will fire
    [ ] Is it a fn returning Result?   → ADD # Errors section
    [ ] Can it panic?                  → ADD # Panics section
    [ ] Is it unsafe?                  → ADD # Safety section (deny-level)
    [ ] Does the summary mention a type without backticks?
                                       → WRAP in [`Type`]
    [ ] Does it have a code example?   → ADD # Examples block

Is this lib.rs or a module root?
  YES →
    [ ] Does it have //!?              → ADD IT
    [ ] Does //! include a quick-start example?
                                       → ADD ONE
    [ ] Was this module's design driven by an ADR?
                                       → ADD # Design with link to ADR

Am I writing a link to another item in the same crate?
    → Use [`Type`] or [`fn_name`] NOT a full URL
    → Wrap any raw external URL in <https://...>

Am I making an architectural or design decision?
    → Write docs/dev/adr/NNN-title.md BEFORE writing code
    → Write docs/dev/specs/title.md for interface/behavior contracts
    → Do NOT put this in code comments — code comments say WHAT, not WHY

Is this user-facing explanation or a how-to guide?
    → Write it in docs/src/guides/ NOT in code comments
    → Code comments are for maintainers, guides are for users
```

---

## 12. Recommended `Cargo.toml` Starter Block

Copy this into any new Rust library crate. Enforces all four layers without touching source files.

```toml
[lints.rust]
missing_docs = "warn"

[lints.rustdoc]
broken_intra_doc_links   = "deny"
invalid_html_tags        = "deny"
missing_crate_level_docs = "warn"
bare_urls                = "warn"
redundant_explicit_links = "warn"
unescaped_backticks      = "warn"

[lints.clippy]
missing_safety_doc           = "deny"
missing_errors_doc           = "warn"
missing_panics_doc           = "warn"
empty_docs                   = "warn"
doc_markdown                 = "warn"
needless_doctest_main        = "warn"
suspicious_doc_comments      = "warn"
too_long_first_doc_paragraph = "warn"
```

---

## Decision Flow Summary

```
PROBLEM OR REQUIREMENT
        │
        ▼
  Write ADR in docs/dev/adr/NNN-title.md
  (Why: context, decision, consequences, alternatives rejected)
        │
        ▼
  Write Spec in docs/dev/specs/title.md
  (What: interface, behavior, invariants, acceptance criteria)
        │
        ▼
  IMPLEMENT in src/
  ├── Link back to ADR in //! doc of affected modules
  ├── /// comments describe WHAT the code does (not WHY — ADR owns WHY)
  └── # Examples blocks = acceptance criteria from Spec, expressed as tests
        │
        ▼
  cargo doc   → target/doc/  (API reference)
  cargo-rdme  → README.md    (auto-synced, never hand-edited)
  mdBook      → book/        (guides + link to API reference)
        │
        ▼
  END USER / MAINTAINER DOCUMENTATION
```

**The connective tissue between layers:**

| Where | What it contains | Points to |
|-------|-----------------|-----------|
| `docs/dev/adr/*.md` | Why the decision was made | Spec if applicable |
| `docs/dev/specs/*.md` | What to build | ADR that approved it |
| `src/module/mod.rs` `//!` | What the module does | ADR + Spec (links) |
| `src/fn` `///` | What the function does | Nothing — self-contained |
| `book/` | How to use the library | API ref (rustdoc link) |

---

## Where Each Document Type Lives

| Document | Location | Authored By | Published To |
|----------|----------|-------------|-------------|
| Architecture Decision Record | `docs/dev/adr/NNN-title.md` | Human engineer | Internal only |
| Technical Specification | `docs/dev/specs/title.md` | Human engineer | Internal only |
| API reference | Generated from `src/**/*.rs` | Code (LLM or human) | `target/doc/` |
| README | Generated from `lib.rs //!` | Code (synced by cargo-rdme) | GitHub, crates.io |
| User guides | `docs/src/guides/*.md` | Human or reviewed LLM | `book/` |
| CHANGELOG | `CHANGELOG.md` | Human or `git-cliff` | GitHub, crates.io |

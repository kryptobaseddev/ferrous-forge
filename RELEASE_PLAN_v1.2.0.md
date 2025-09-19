# Ferrous Forge v1.2.0 Release Plan
## "Rust Version & Edition Management"

**Target Release Date**: October 3, 2025 (2 weeks development)  
**Version**: 1.2.0  
**Codename**: "Edition Eagle"

---

## 🎯 Release Scope

### Core Features for v1.2.0

We're implementing a **focused subset** of the planned features to deliver value quickly:

#### 1. ✅ **Rust Version Checking** (Priority 1)
- [x] Detect currently installed Rust version via `rustc --version`
- [x] Check latest stable release from GitHub API
- [x] Compare versions and recommend updates
- [x] Cache GitHub API responses (1hr TTL)

#### 2. ✅ **Edition Compliance** (Priority 1)
- [x] Detect project edition from Cargo.toml
- [x] Check if using latest edition (2024)
- [x] Provide migration recommendations
- [x] Basic `cargo fix --edition` wrapper

#### 3. ✅ **Enhanced Configuration** (Priority 2)
- [x] Add rust version preferences to config
- [x] Add edition preferences
- [x] Project-level defaults for dependencies
- [x] Save/load enhanced configs

#### 4. ✅ **New CLI Commands** (Priority 1)
```bash
ferrous-forge rust check        # Check Rust version status
ferrous-forge rust recommend    # Get update recommendations
ferrous-forge edition check     # Check edition compliance
ferrous-forge edition migrate   # Guided edition migration
```

---

## 🚫 Deferred to v1.3.0

These features will be implemented in the next release:
- Full template system 2.0
- Community config sharing
- Advanced rustup integration
- GitHub release notifications
- Workspace-level configurations
- Hook scripts

---

## 📋 Implementation Tasks

### Week 1: Core Development (Sept 20-26)

#### Day 1-2: Foundation
- [ ] Create `src/rust_version/mod.rs` module structure
- [ ] Implement version detection (`rustc --version` parser)
- [ ] Create GitHub API client with caching
- [ ] Add error handling for API failures

#### Day 3-4: Edition Management  
- [ ] Create `src/edition/mod.rs` module
- [ ] Implement Cargo.toml edition detection
- [ ] Create edition analyzer (check compatibility)
- [ ] Implement basic migration wrapper

#### Day 5: Configuration Enhancement
- [ ] Extend `Config` struct with new fields
- [ ] Add version/edition preferences
- [ ] Implement project defaults system
- [ ] Update config serialization

#### Day 6-7: CLI Integration
- [ ] Add `rust` subcommand to CLI
- [ ] Add `edition` subcommand to CLI
- [ ] Implement command handlers
- [ ] Add progress indicators and output formatting

### Week 2: Testing & Release (Sept 27 - Oct 3)

#### Day 8-9: Testing
- [ ] Unit tests for version parsing
- [ ] Unit tests for edition detection
- [ ] Integration tests for GitHub API
- [ ] Mock tests for CLI commands
- [ ] Manual testing on real projects

#### Day 10-11: Documentation
- [ ] Update README with new features
- [ ] Add examples for new commands
- [ ] Update CHANGELOG
- [ ] Create migration guide from v1.1.0

#### Day 12: Release Preparation
- [ ] Version bump to 1.2.0
- [ ] Final testing pass
- [ ] Build release binaries
- [ ] Prepare release notes

#### Day 13-14: Release
- [ ] Tag v1.2.0
- [ ] Create GitHub release
- [ ] Publish to crates.io
- [ ] Announce release

---

## 🏗️ Technical Implementation

### File Structure
```
src/
├── rust_version/
│   ├── mod.rs          # Module root
│   ├── detector.rs     # Version detection
│   ├── github.rs       # GitHub API client
│   └── cache.rs        # Response caching
├── edition/
│   ├── mod.rs          # Module root
│   ├── analyzer.rs     # Edition analysis
│   └── migrator.rs     # Migration wrapper
├── commands/
│   ├── rust.rs         # New rust commands
│   └── edition.rs      # New edition commands
└── lib.rs              # Update exports
```

### Dependencies to Add
```toml
[dependencies]
# Existing deps...
semver = "1.0"          # Version parsing
octocrab = "0.41"       # GitHub API client
cached = "0.54"         # Caching layer
```

### Key Implementation Details

#### Version Detection
```rust
pub fn detect_rust_version() -> Result<RustVersion> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()?;
    
    // Parse: rustc 1.90.0 (4b06a43a1 2025-08-07)
    let version_str = String::from_utf8(output.stdout)?;
    RustVersion::parse(&version_str)
}
```

#### GitHub API Integration
```rust
pub async fn get_latest_stable() -> Result<Release> {
    // Use cache-first approach
    if let Some(cached) = cache.get("latest_stable") {
        return Ok(cached);
    }
    
    let client = octocrab::instance();
    let release = client
        .repos("rust-lang", "rust")
        .releases()
        .get_latest()
        .await?;
    
    cache.set("latest_stable", release, Duration::from_secs(3600));
    Ok(release)
}
```

#### Edition Detection
```rust
pub fn detect_edition(manifest_path: &Path) -> Result<Edition> {
    let contents = fs::read_to_string(manifest_path)?;
    let manifest: toml::Value = toml::from_str(&contents)?;
    
    let edition = manifest
        .get("package")
        .and_then(|p| p.get("edition"))
        .and_then(|e| e.as_str())
        .unwrap_or("2015");
    
    Edition::from_str(edition)
}
```

---

## ✅ Testing Strategy

### Unit Tests Required
- [ ] Version string parsing (multiple formats)
- [ ] Edition detection from Cargo.toml
- [ ] Cache expiration logic
- [ ] Version comparison logic
- [ ] Config serialization/deserialization

### Integration Tests Required
- [ ] GitHub API with mock server
- [ ] Full CLI command flow
- [ ] Edition migration on test project
- [ ] Config file operations

### Manual Testing Checklist
- [ ] Test on project with Rust 2015 edition
- [ ] Test on project with Rust 2018 edition
- [ ] Test on project with Rust 2021 edition
- [ ] Test with no internet (cache/offline mode)
- [ ] Test on Windows, macOS, Linux

---

## 📝 Documentation Updates

### README.md Additions
```markdown
## 🆕 Version & Edition Management (v1.2.0)

Ferrous Forge now helps you stay up-to-date with Rust:

### Check Your Rust Version
```bash
ferrous-forge rust check
# Output:
# Current: rustc 1.89.0
# Latest: rustc 1.90.0
# 📦 Update available! Run `rustup update stable`
```

### Check Edition Compliance
```bash
ferrous-forge edition check
# Output:
# Current edition: 2021
# Latest edition: 2024
# Run `ferrous-forge edition migrate` to upgrade
```
```

### CHANGELOG.md Entry
```markdown
## [1.2.0] - 2025-10-03

### Added
- Rust version checking against GitHub releases
- Edition compliance detection and recommendations
- Basic edition migration assistance
- Enhanced configuration with version preferences
- New CLI commands: `rust check`, `rust recommend`, `edition check`, `edition migrate`
- GitHub API response caching for offline support

### Changed
- Extended Config struct with version and edition preferences
- Improved error messages with actionable recommendations

### Technical
- Added semver for version parsing
- Added octocrab for GitHub API integration
- Added cached for response caching
```

---

## 🚀 Release Checklist

### Pre-Release
- [ ] All tests passing (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] README examples tested

### Release Process
```bash
# 1. Final checks
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check

# 2. Build and test release
cargo build --release
./target/release/ferrous-forge rust check

# 3. Update version
# Edit Cargo.toml: version = "1.2.0"

# 4. Commit and tag
git add -A
git commit -m "chore(release): prepare v1.2.0"
git tag -a v1.2.0 -m "Release version 1.2.0 - Rust Version & Edition Management"

# 5. Push to GitHub
git push origin main
git push origin v1.2.0

# 6. Create GitHub release
# Via GitHub UI with release notes

# 7. Publish to crates.io
cargo publish

# 8. Verify
# Check https://crates.io/crates/ferrous-forge/1.2.0
# Check https://docs.rs/ferrous-forge/1.2.0
```

### Post-Release
- [ ] Verify crates.io listing
- [ ] Verify docs.rs generation
- [ ] Test installation: `cargo install ferrous-forge`
- [ ] Announce on Reddit/r/rust
- [ ] Update project website (if applicable)

---

## 📊 Success Metrics

### Technical Metrics
- ✓ < 100ms version check (with cache)
- ✓ < 2s GitHub API call (without cache)
- ✓ Zero panics in production
- ✓ 80%+ test coverage

### User Experience Metrics
- ✓ Clear, actionable output messages
- ✓ Graceful offline handling
- ✓ Helpful error messages
- ✓ Fast command response

### Release Metrics
- ✓ No critical bugs in first 48 hours
- ✓ Successful docs.rs build
- ✓ Clean installation on all platforms
- ✓ Positive initial feedback

---

## 🎯 Risk Management

### Identified Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| GitHub API rate limits | Medium | Low | Implement caching, handle gracefully |
| Breaking changes in deps | Low | High | Pin dependency versions |
| Platform-specific issues | Medium | Medium | Test on all platforms before release |
| rustc output format changes | Low | Medium | Flexible parsing with fallbacks |

### Contingency Plans

1. **If GitHub API is down**: 
   - Use cached data if available
   - Provide clear error message
   - Still allow other features to work

2. **If edition migration fails**:
   - Provide manual migration guide
   - Link to official documentation
   - Don't modify user's code

3. **If timeline slips**:
   - Reduce scope (defer edition migration)
   - Focus on version checking only
   - Release as 1.2.0-beta first

---

## 🎉 Definition of Done

### Feature Complete When:
- [x] All planned CLI commands work
- [x] Version checking accurate
- [x] Edition detection reliable
- [x] Tests provide 80%+ coverage
- [x] Documentation complete
- [x] No critical bugs

### Release Complete When:
- [x] Published to crates.io
- [x] GitHub release created
- [x] docs.rs successfully built
- [x] Installation verified on all platforms
- [x] Announcement posted

---

## 📅 Timeline Summary

**Week 1 (Sept 20-26)**: Core Development
- Mon-Tue: Foundation & Version Detection
- Wed-Thu: Edition Management
- Fri: Configuration Enhancement
- Sat-Sun: CLI Integration

**Week 2 (Sept 27 - Oct 3)**: Testing & Release
- Mon-Tue: Testing Suite
- Wed-Thu: Documentation
- Fri: Release Preparation
- Sat-Sun: Release & Announcement

**Total Duration**: 14 days
**Release Date**: October 3, 2025

---

## 🚦 Go/No-Go Criteria

### Go Criteria (Release)
- All tests passing
- Documentation complete
- No critical bugs
- Manual testing successful

### No-Go Criteria (Delay)
- Critical bugs found
- Test coverage < 70%
- Platform-specific failures
- Dependencies have breaking changes

---

## Next Steps After v1.2.0

### v1.3.0 Planning (Future)
- Full template system
- Community sharing features
- Advanced rustup integration
- Workspace support

### v2.0.0 Vision (Long-term)
- Complete rewrite if needed
- Plugin system
- Cloud synchronization
- IDE integrations

---

This plan provides a **realistic, achievable scope** for v1.2.0 that delivers immediate value while setting the foundation for future enhancements.

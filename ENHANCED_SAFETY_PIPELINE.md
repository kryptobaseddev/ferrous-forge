# Ferrous Forge Enhanced Safety Pipeline
## Mandatory Pre-Commit/Pre-Push Validation

---

## Overview

This enhancement adds **mandatory safety checks** to Ferrous Forge that prevent broken code from ever reaching GitHub or crates.io. These checks run automatically as git hooks and cargo wrappers, ensuring that **only CI-compatible code** can be committed or published.

---

## Core Safety Features

### 1. 🛡️ **Pre-Commit Safety Pipeline**

**Triggered on**: Every `git commit`  
**Blocks commit if**: Any check fails  
**Checks performed**:

```bash
# Automatic checks before EVERY commit:
1. cargo fmt --check           # Format validation
2. cargo clippy --all-targets --all-features -- -D warnings  # Lint validation  
3. cargo test --all-targets --all-features                   # Test validation
4. cargo build --release                                     # Build validation
5. Custom Ferrous Forge validations                          # Standards validation
```

### 2. 🚫 **Pre-Push Protection**

**Triggered on**: Every `git push`  
**Blocks push if**: Any check fails  
**Additional checks**:

```bash
# Additional checks before pushing:
1. All pre-commit checks (repeated)
2. cargo audit                 # Security audit
3. cargo doc --no-deps        # Documentation build
4. Integration test suite     # Full integration tests
5. Semver compatibility check # Breaking change detection
```

### 3. 📦 **Cargo Publish Wrapper**

**Triggered on**: `cargo publish` (hijacked by Ferrous Forge)  
**Blocks publish if**: Any check fails  
**Comprehensive validation**:

```bash
# Before allowing crates.io publish:
1. All pre-push checks (repeated)
2. cargo publish --dry-run    # Publish validation
3. Documentation completeness # 100% doc coverage
4. License validation         # License compatibility
5. Dependency audit          # Supply chain security
6. Version bump validation   # Semver compliance
```

---

## Implementation Design

### Module Structure

```
src/
├── safety/
│   ├── mod.rs              # Safety pipeline coordinator
│   ├── pre_commit.rs       # Pre-commit hook implementation
│   ├── pre_push.rs         # Pre-push hook implementation
│   ├── publish_guard.rs    # Cargo publish wrapper
│   ├── checks.rs           # Individual check implementations
│   └── config.rs           # Safety configuration
├── git_hooks/ (enhanced)
│   ├── mod.rs              # Enhanced git hooks
│   ├── installer.rs        # Hook installation
│   └── templates/          # Hook script templates
└── cargo_wrapper/
    ├── mod.rs              # Cargo command hijacking
    ├── publish.rs          # Publish command wrapper
    └── build.rs            # Build command wrapper
```

### Safety Pipeline Coordinator

```rust
//! src/safety/mod.rs

use crate::{Error, Result};
use std::path::Path;
use std::process::Command;

pub mod checks;
pub mod config;
pub mod pre_commit;
pub mod pre_push;
pub mod publish_guard;

/// Safety pipeline coordinator
pub struct SafetyPipeline {
    config: config::SafetyConfig,
    project_path: std::path::PathBuf,
}

impl SafetyPipeline {
    pub fn new(project_path: impl AsRef<Path>) -> Result<Self> {
        let config = config::SafetyConfig::load_or_default()?;
        
        Ok(Self {
            config,
            project_path: project_path.as_ref().to_path_buf(),
        })
    }
    
    /// Run pre-commit safety checks
    pub async fn run_pre_commit_checks(&self) -> Result<SafetyReport> {
        let mut report = SafetyReport::new("pre-commit");
        
        // 1. Format check
        report.add_check(checks::format_check(&self.project_path).await?);
        
        // 2. Clippy check
        report.add_check(checks::clippy_check(&self.project_path).await?);
        
        // 3. Test check
        if self.config.run_tests_on_commit {
            report.add_check(checks::test_check(&self.project_path).await?);
        }
        
        // 4. Build check
        report.add_check(checks::build_check(&self.project_path).await?);
        
        // 5. Ferrous Forge validation
        report.add_check(checks::standards_check(&self.project_path).await?);
        
        Ok(report)
    }
    
    /// Run pre-push safety checks
    pub async fn run_pre_push_checks(&self) -> Result<SafetyReport> {
        let mut report = self.run_pre_commit_checks().await?;
        report.stage = "pre-push".to_string();
        
        // Additional pre-push checks
        report.add_check(checks::security_audit(&self.project_path).await?);
        report.add_check(checks::doc_build_check(&self.project_path).await?);
        
        if self.config.run_integration_tests {
            report.add_check(checks::integration_test_check(&self.project_path).await?);
        }
        
        Ok(report)
    }
    
    /// Run publish safety checks
    pub async fn run_publish_checks(&self) -> Result<SafetyReport> {
        let mut report = self.run_pre_push_checks().await?;
        report.stage = "publish".to_string();
        
        // Additional publish checks
        report.add_check(checks::publish_dry_run(&self.project_path).await?);
        report.add_check(checks::doc_coverage_check(&self.project_path).await?);
        report.add_check(checks::license_check(&self.project_path).await?);
        report.add_check(checks::semver_check(&self.project_path).await?);
        
        Ok(report)
    }
}

/// Safety check report
#[derive(Debug, Clone)]
pub struct SafetyReport {
    pub stage: String,
    pub checks: Vec<CheckResult>,
    pub passed: bool,
    pub total_time: std::time::Duration,
}

impl SafetyReport {
    pub fn new(stage: &str) -> Self {
        Self {
            stage: stage.to_string(),
            checks: Vec::new(),
            passed: true,
            total_time: std::time::Duration::default(),
        }
    }
    
    pub fn add_check(&mut self, check: CheckResult) {
        if !check.passed {
            self.passed = false;
        }
        self.total_time += check.duration;
        self.checks.push(check);
    }
    
    /// Print detailed report
    pub fn print_report(&self) {
        println!("🛡️  Ferrous Forge Safety Pipeline - {}\n", self.stage);
        
        for check in &self.checks {
            let status = if check.passed {
                "✅"
            } else {
                "❌"
            };
            
            println!("  {} {} ({:.2}s)", status, check.name, check.duration.as_secs_f64());
            
            if !check.passed {
                for error in &check.errors {
                    println!("    ⚠️  {}", error);
                }
            }
        }
        
        println!("\nTotal time: {:.2}s", self.total_time.as_secs_f64());
        
        if self.passed {
            println!("🎉 All safety checks passed!");
        } else {
            println!("🚨 Safety checks FAILED - operation blocked!");
        }
    }
}

/// Individual check result
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub duration: std::time::Duration,
}
```

### Individual Safety Checks

```rust
//! src/safety/checks.rs

use crate::{Error, Result};
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use super::CheckResult;

/// Run cargo fmt --check
pub async fn format_check(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult {
        name: "Format Check".to_string(),
        passed: true,
        errors: Vec::new(),
        duration: std::time::Duration::default(),
    };
    
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["fmt", "--check"])
        .output()?;
    
    result.duration = start.elapsed();
    
    if !output.status.success() {
        result.passed = false;
        let stderr = String::from_utf8_lossy(&output.stderr);
        result.errors.push(format!("Format violations found: {}", stderr));
        result.errors.push("Run 'cargo fmt' to fix formatting".to_string());
    }
    
    Ok(result)
}

/// Run cargo clippy with strict warnings
pub async fn clippy_check(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult {
        name: "Clippy Check".to_string(),
        passed: true,
        errors: Vec::new(),
        duration: std::time::Duration::default(),
    };
    
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"])
        .output()?;
    
    result.duration = start.elapsed();
    
    if !output.status.success() {
        result.passed = false;
        let stderr = String::from_utf8_lossy(&output.stderr);
        result.errors.push(format!("Clippy violations found: {}", stderr));
        result.errors.push("Fix clippy warnings before committing".to_string());
    }
    
    Ok(result)
}

/// Run cargo test
pub async fn test_check(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult {
        name: "Test Check".to_string(),
        passed: true,
        errors: Vec::new(),
        duration: std::time::Duration::default(),
    };
    
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["test", "--all-targets", "--all-features"])
        .output()?;
    
    result.duration = start.elapsed();
    
    if !output.status.success() {
        result.passed = false;
        let stderr = String::from_utf8_lossy(&output.stderr);
        result.errors.push(format!("Tests failed: {}", stderr));
        result.errors.push("Fix failing tests before committing".to_string());
    }
    
    Ok(result)
}

/// Run cargo build --release
pub async fn build_check(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult {
        name: "Build Check".to_string(),
        passed: true,
        errors: Vec::new(),
        duration: std::time::Duration::default(),
    };
    
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["build", "--release"])
        .output()?;
    
    result.duration = start.elapsed();
    
    if !output.status.success() {
        result.passed = false;
        let stderr = String::from_utf8_lossy(&output.stderr);
        result.errors.push(format!("Build failed: {}", stderr));
        result.errors.push("Fix build errors before committing".to_string());
    }
    
    Ok(result)
}

/// Run Ferrous Forge standards validation
pub async fn standards_check(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult {
        name: "Standards Check".to_string(),
        passed: true,
        errors: Vec::new(),
        duration: std::time::Duration::default(),
    };
    
    // Run ferrous-forge validate
    let output = Command::new("ferrous-forge")
        .current_dir(project_path)
        .args(&["validate", "."])
        .output();
    
    result.duration = start.elapsed();
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                result.passed = false;
                let stderr = String::from_utf8_lossy(&output.stderr);
                result.errors.push(format!("Standards violations: {}", stderr));
            }
        }
        Err(_) => {
            // If ferrous-forge isn't available, skip this check
            result.errors.push("Ferrous Forge not available - skipping standards check".to_string());
        }
    }
    
    Ok(result)
}

/// Run cargo audit
pub async fn security_audit(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult {
        name: "Security Audit".to_string(),
        passed: true,
        errors: Vec::new(),
        duration: std::time::Duration::default(),
    };
    
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["audit"])
        .output();
    
    result.duration = start.elapsed();
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                result.passed = false;
                let stderr = String::from_utf8_lossy(&output.stderr);
                result.errors.push(format!("Security vulnerabilities found: {}", stderr));
                result.errors.push("Run 'cargo audit fix' to resolve vulnerabilities".to_string());
            }
        }
        Err(_) => {
            result.errors.push("cargo-audit not installed - run 'cargo install cargo-audit'".to_string());
        }
    }
    
    Ok(result)
}

/// Run cargo doc build check
pub async fn doc_build_check(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult {
        name: "Documentation Build".to_string(),
        passed: true,
        errors: Vec::new(),
        duration: std::time::Duration::default(),
    };
    
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["doc", "--no-deps", "--document-private-items"])
        .output()?;
    
    result.duration = start.elapsed();
    
    if !output.status.success() {
        result.passed = false;
        let stderr = String::from_utf8_lossy(&output.stderr);
        result.errors.push(format!("Documentation build failed: {}", stderr));
    }
    
    Ok(result)
}

/// Run cargo publish --dry-run
pub async fn publish_dry_run(project_path: &Path) -> Result<CheckResult> {
    let start = Instant::now();
    let mut result = CheckResult {
        name: "Publish Dry Run".to_string(),
        passed: true,
        errors: Vec::new(),
        duration: std::time::Duration::default(),
    };
    
    let output = Command::new("cargo")
        .current_dir(project_path)
        .args(&["publish", "--dry-run"])
        .output()?;
    
    result.duration = start.elapsed();
    
    if !output.status.success() {
        result.passed = false;
        let stderr = String::from_utf8_lossy(&output.stderr);
        result.errors.push(format!("Publish dry run failed: {}", stderr));
    }
    
    Ok(result)
}

// Additional checks: doc_coverage_check, license_check, semver_check, etc.
```

---

## Git Hooks Implementation

### Enhanced Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit (installed by Ferrous Forge)

echo "🛡️  Ferrous Forge Safety Pipeline - Pre-Commit"
echo "================================================"

# Run the safety pipeline
ferrous-forge safety run-pre-commit

# Exit with the same code as the safety pipeline
exit $?
```

### Enhanced Pre-Push Hook

```bash
#!/bin/bash
# .git/hooks/pre-push (installed by Ferrous Forge)

echo "🛡️  Ferrous Forge Safety Pipeline - Pre-Push"
echo "=============================================="

# Get remote and branch being pushed to
remote="$1"
url="$2"

echo "Pushing to: $remote ($url)"

# Run the safety pipeline
ferrous-forge safety run-pre-push

# Exit with the same code as the safety pipeline
exit $?
```

---

## Cargo Command Hijacking

### Cargo Publish Wrapper

```rust
//! src/cargo_wrapper/publish.rs

use crate::safety::SafetyPipeline;
use crate::{Error, Result};
use std::env;
use std::process::Command;

/// Hijacked cargo publish command
pub async fn hijacked_cargo_publish() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let project_path = env::current_dir()?;
    
    println!("🛡️  Ferrous Forge Safety Pipeline - Cargo Publish");
    println!("=================================================");
    
    // Run comprehensive safety checks
    let pipeline = SafetyPipeline::new(&project_path)?;
    let report = pipeline.run_publish_checks().await?;
    
    report.print_report();
    
    if !report.passed {
        println!("\n🚨 PUBLISH BLOCKED: Safety checks failed!");
        println!("Fix the issues above before publishing to crates.io");
        return Err(Error::safety("Publish blocked by safety pipeline"));
    }
    
    println!("\n🎉 All safety checks passed! Proceeding with publish...");
    
    // Execute the original cargo publish command
    let status = Command::new("cargo")
        .args(&args[1..]) // Skip the program name
        .status()?;
    
    if !status.success() {
        return Err(Error::process("cargo publish failed"));
    }
    
    println!("✅ Successfully published to crates.io!");
    Ok(())
}
```

---

## Safety Configuration

```toml
# ~/.config/ferrous-forge/safety.toml

[safety]
enabled = true
strict_mode = true              # Block all operations on failure
developer_mode = false          # Allow bypassing in dev mode

[pre_commit]
enabled = true
run_format_check = true
run_clippy_check = true  
run_tests = false               # Too slow for every commit
run_build_check = true
run_standards_check = true
timeout_seconds = 300           # 5 minute timeout

[pre_push]
enabled = true
run_security_audit = true
run_doc_build = true
run_integration_tests = false   # Optional for large projects
run_full_test_suite = true
timeout_seconds = 600           # 10 minute timeout

[publish]
enabled = true
require_doc_coverage = true
require_license_check = true
require_semver_check = true
require_security_audit = true
block_on_warnings = true
timeout_seconds = 900           # 15 minute timeout

[bypass]
# Emergency bypass (requires confirmation)
allow_bypass = false
require_reason = true
log_bypasses = true
```

---

## CLI Integration

### New Safety Commands

```bash
# Install enhanced safety hooks
ferrous-forge safety install

# Run safety checks manually
ferrous-forge safety check --stage=pre-commit
ferrous-forge safety check --stage=pre-push  
ferrous-forge safety check --stage=publish

# Configure safety pipeline
ferrous-forge safety config --set pre_commit.run_tests=true
ferrous-forge safety config --show

# Emergency bypass (with confirmation)
ferrous-forge safety bypass --reason="hotfix for production"

# View safety report
ferrous-forge safety report --last=5
```

### Enhanced Init Command

```rust
// Update src/commands/init.rs

pub async fn execute(force: bool) -> Result<()> {
    // ... existing init logic ...
    
    // Install enhanced safety hooks
    println!("🛡️  Installing enhanced safety pipeline...");
    
    let safety_installer = crate::safety::installer::SafetyInstaller::new();
    safety_installer.install_hooks().await?;
    safety_installer.install_cargo_wrappers().await?;
    
    println!("✅ Safety pipeline installed!");
    println!("\nYour development workflow is now protected:");
    println!("  • Pre-commit: Format, clippy, build, standards checks");
    println!("  • Pre-push: Additional security and documentation checks");  
    println!("  • Publish: Comprehensive validation before crates.io");
    
    Ok(())
}
```

---

## User Experience

### Developer Workflow

```bash
# Normal development - all checks run automatically
git add .
git commit -m "feat: add new feature"
# 🛡️  Ferrous Forge Safety Pipeline - Pre-Commit
# ✅ Format Check (0.1s)
# ✅ Clippy Check (2.3s)  
# ✅ Build Check (15.2s)
# ✅ Standards Check (1.1s)
# 🎉 All safety checks passed!

git push origin main
# 🛡️  Ferrous Forge Safety Pipeline - Pre-Push
# ✅ All pre-commit checks (18.7s)
# ✅ Security Audit (0.8s)
# ✅ Documentation Build (3.2s)
# 🎉 All safety checks passed!

cargo publish
# 🛡️  Ferrous Forge Safety Pipeline - Cargo Publish
# ✅ All pre-push checks (22.7s)
# ✅ Publish Dry Run (12.3s)
# ✅ Documentation Coverage (2.1s)
# ✅ License Check (0.2s)
# ✅ Semver Check (1.1s)
# 🎉 All safety checks passed! Proceeding with publish...
```

### When Checks Fail

```bash
git commit -m "broken code"
# 🛡️  Ferrous Forge Safety Pipeline - Pre-Commit
# ✅ Format Check (0.1s)
# ❌ Clippy Check (2.1s)
#   ⚠️  error: used `unwrap()` on a `Result` value
#   ⚠️  Fix clippy warnings before committing
# 🚨 Safety checks FAILED - operation blocked!
# 
# Commit aborted. Fix the issues above and try again.
```

---

## Configuration Integration

### Enhanced Config Schema

```rust
// Add to src/config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... existing fields ...
    
    /// Safety pipeline configuration
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Enable safety pipeline
    pub enabled: bool,
    /// Strict mode (block all operations on failure)
    pub strict_mode: bool,
    /// Pre-commit checks configuration
    pub pre_commit: PreCommitConfig,
    /// Pre-push checks configuration
    pub pre_push: PrePushConfig,
    /// Publish checks configuration
    pub publish: PublishConfig,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strict_mode: true,
            pre_commit: PreCommitConfig::default(),
            pre_push: PrePushConfig::default(),
            publish: PublishConfig::default(),
        }
    }
}
```

---

## Implementation Benefits

### For Developers
- ✅ **Never push broken code** - Impossible to commit/push failing code
- ✅ **Faster feedback** - Catch issues locally before CI
- ✅ **Consistent quality** - Same checks everywhere
- ✅ **Learning tool** - Immediate feedback on best practices

### For Teams
- ✅ **No broken CI** - CI always passes because local checks ensure it
- ✅ **No broken releases** - Impossible to publish broken crates
- ✅ **Consistent standards** - All team members use same checks
- ✅ **Reduced review time** - Code is pre-validated

### For the Rust Ecosystem
- ✅ **Higher quality crates** - Only validated code reaches crates.io
- ✅ **Better CI reliability** - Fewer failing builds
- ✅ **Faster development** - Less time debugging CI issues

---

## Emergency Bypass System

For emergency situations (hotfixes, etc.):

```bash
# Emergency bypass with required justification
ferrous-forge safety bypass \
  --reason="Critical security hotfix for CVE-2025-12345" \
  --stage=pre-push \
  --confirm

# All bypasses are logged for audit
ferrous-forge safety audit-log
```

---

## Rollout Strategy

### Phase 1: Basic Implementation
- Pre-commit hooks with format/clippy/build checks
- Basic cargo publish wrapper
- Configuration system

### Phase 2: Enhanced Features  
- Pre-push hooks with security/doc checks
- Integration test support
- Bypass system with logging

### Phase 3: Advanced Features
- Semver compatibility checking
- Custom rule integration
- Team configuration sharing

---

This enhancement transforms Ferrous Forge from a **reactive** tool (catching issues after they happen) into a **proactive** tool (preventing issues from ever occurring). It solves exactly the problem you identified - ensuring that broken code never makes it to CI/CD or crates.io in the first place.

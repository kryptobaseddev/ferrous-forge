# Ferrous Forge API Integration Design
## GitHub Releases & Rustup Integration

---

## Overview

This document details the API integration design for Ferrous Forge to interact with GitHub's Rust releases and the rustup toolchain manager. It provides a comprehensive approach to version checking, update management, and toolchain coordination.

---

## GitHub API Integration

### API Endpoints

```rust
// Base URLs
const GITHUB_API_BASE: &str = "https://api.github.com";
const RUST_REPO: &str = "rust-lang/rust";

// Endpoints
const RELEASES_LATEST: &str = "/repos/rust-lang/rust/releases/latest";
const RELEASES_ALL: &str = "/repos/rust-lang/rust/releases";
const TAGS: &str = "/repos/rust-lang/rust/tags";
const COMMITS: &str = "/repos/rust-lang/rust/commits";
```

### Release Data Structure

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub id: u64,
    pub tag_name: String,           // e.g., "1.90.0"
    pub target_commitish: String,   // branch or commit
    pub name: String,               // Release name
    pub body: String,              // Release notes (markdown)
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub assets: Vec<ReleaseAsset>,
    pub author: Author,
    
    // Custom fields we'll add
    #[serde(skip)]
    pub parsed_version: semver::Version,
    #[serde(skip)]
    pub channel: ReleaseChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub id: u64,
    pub name: String,
    pub label: Option<String>,
    pub content_type: String,
    pub size: u64,
    pub download_url: String,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReleaseChannel {
    Stable,
    Beta,
    Nightly,
}
```

### GitHub API Client

```rust
use reqwest::{Client, header};
use std::time::Duration;

pub struct GitHubClient {
    client: Client,
    auth_token: Option<String>,
    cache: Cache<String, Vec<u8>>,
}

impl GitHubClient {
    pub fn new(auth_token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("ferrous-forge/{}", env!("CARGO_PKG_VERSION")))
            .build()?;
            
        Ok(Self {
            client,
            auth_token,
            cache: Cache::new(Duration::from_secs(3600)), // 1hr cache
        })
    }
    
    pub async fn get_latest_release(&self) -> Result<GitHubRelease> {
        let url = format!("{}{}", GITHUB_API_BASE, RELEASES_LATEST);
        
        // Check cache first
        if let Some(cached) = self.cache.get(&url) {
            return Ok(serde_json::from_slice(&cached)?);
        }
        
        let mut request = self.client.get(&url)
            .header(header::ACCEPT, "application/vnd.github.v3+json");
            
        if let Some(token) = &self.auth_token {
            request = request.header(header::AUTHORIZATION, format!("token {}", token));
        }
        
        let response = request.send().await?;
        
        // Handle rate limiting
        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("X-RateLimit-Reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60);
                
            return Err(Error::RateLimited { retry_after });
        }
        
        let bytes = response.bytes().await?;
        self.cache.insert(url, bytes.to_vec());
        
        let release: GitHubRelease = serde_json::from_slice(&bytes)?;
        Ok(self.parse_release(release)?)
    }
    
    pub async fn get_releases(&self, per_page: u8, page: u8) -> Result<Vec<GitHubRelease>> {
        let url = format!("{}{}?per_page={}&page={}", 
            GITHUB_API_BASE, RELEASES_ALL, per_page, page);
            
        // Similar implementation with caching and rate limiting
        todo!()
    }
    
    pub async fn get_release_by_version(&self, version: &str) -> Result<Option<GitHubRelease>> {
        let releases = self.get_releases(100, 1).await?;
        
        Ok(releases.into_iter()
            .find(|r| r.tag_name == version || r.tag_name == format!("v{}", version)))
    }
    
    fn parse_release(&self, mut release: GitHubRelease) -> Result<GitHubRelease> {
        // Parse version from tag
        let version_str = release.tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name);
            
        release.parsed_version = semver::Version::parse(version_str)?;
        
        // Determine channel
        release.channel = if release.prerelease {
            if version_str.contains("beta") {
                ReleaseChannel::Beta
            } else if version_str.contains("nightly") {
                ReleaseChannel::Nightly
            } else {
                ReleaseChannel::Beta // Default prerelease to beta
            }
        } else {
            ReleaseChannel::Stable
        };
        
        Ok(release)
    }
}
```

### Release Notes Parser

```rust
pub struct ReleaseNotesParser;

impl ReleaseNotesParser {
    pub fn parse(markdown: &str) -> ReleaseNotes {
        let mut notes = ReleaseNotes::default();
        let mut current_section = None;
        
        for line in markdown.lines() {
            if line.starts_with("## ") {
                current_section = Some(line[3..].trim().to_string());
            } else if line.starts_with("### ") {
                // Handle subsections
            } else if let Some(ref section) = current_section {
                match section.as_str() {
                    "Security" | "Security Updates" => {
                        if line.starts_with("* ") || line.starts_with("- ") {
                            notes.security_updates.push(line[2..].trim().to_string());
                        }
                    }
                    "Breaking Changes" => {
                        notes.breaking_changes.push(line[2..].trim().to_string());
                    }
                    "New Features" => {
                        notes.new_features.push(line[2..].trim().to_string());
                    }
                    _ => {}
                }
            }
        }
        
        notes
    }
}

#[derive(Debug, Default)]
pub struct ReleaseNotes {
    pub security_updates: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub new_features: Vec<String>,
    pub bug_fixes: Vec<String>,
    pub performance: Vec<String>,
}
```

---

## Rustup Integration

### Rustup Command Wrapper

```rust
use std::process::{Command, Output};
use std::str;

pub struct RustupClient {
    rustup_path: PathBuf,
}

impl RustupClient {
    pub fn new() -> Result<Self> {
        let rustup_path = which::which("rustup")
            .map_err(|_| Error::RustupNotFound)?;
            
        Ok(Self { rustup_path })
    }
    
    /// Get currently active toolchain
    pub fn get_active_toolchain(&self) -> Result<Toolchain> {
        let output = self.run_command(&["show", "active-toolchain"])?;
        Toolchain::parse(&output)
    }
    
    /// List all installed toolchains
    pub fn list_toolchains(&self) -> Result<Vec<Toolchain>> {
        let output = self.run_command(&["toolchain", "list"])?;
        
        output.lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let name = line.split_whitespace().next().unwrap_or("");
                let is_default = line.contains("(default)");
                Toolchain::from_str(name, is_default)
            })
            .collect()
    }
    
    /// Get version of specific toolchain
    pub fn get_toolchain_version(&self, toolchain: &str) -> Result<RustVersion> {
        let output = self.run_command(&[
            "run", toolchain, "rustc", "--version"
        ])?;
        
        RustVersion::parse(&output)
    }
    
    /// Install a toolchain
    pub async fn install_toolchain(&self, toolchain: &str) -> Result<()> {
        let output = Command::new(&self.rustup_path)
            .args(&["toolchain", "install", toolchain])
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(Error::ToolchainInstallFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Update toolchains
    pub async fn update(&self) -> Result<UpdateResult> {
        let output = self.run_command(&["update"])?;
        UpdateResult::parse(&output)
    }
    
    /// Set default toolchain
    pub fn set_default(&self, toolchain: &str) -> Result<()> {
        self.run_command(&["default", toolchain])?;
        Ok(())
    }
    
    /// Override toolchain for current directory
    pub fn set_override(&self, toolchain: &str) -> Result<()> {
        self.run_command(&["override", "set", toolchain])?;
        Ok(())
    }
    
    fn run_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.rustup_path)
            .args(args)
            .output()?;
            
        if !output.status.success() {
            return Err(Error::RustupCommand(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
```

### Toolchain Data Structures

```rust
#[derive(Debug, Clone)]
pub struct Toolchain {
    pub name: String,              // e.g., "stable-x86_64-unknown-linux-gnu"
    pub channel: Channel,          // stable/beta/nightly
    pub date: Option<NaiveDate>,   // For dated nightly/beta
    pub host: String,              // Target triple
    pub is_default: bool,
    pub components: Vec<Component>,
}

#[derive(Debug, Clone)]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,              // e.g., "rustc", "cargo", "clippy"
    pub version: Option<String>,
    pub installed: bool,
}

#[derive(Debug, Clone)]
pub struct RustVersion {
    pub version: semver::Version,
    pub commit_hash: String,
    pub commit_date: NaiveDate,
    pub host: String,
    pub release_channel: Channel,
}

impl RustVersion {
    pub fn parse(version_output: &str) -> Result<Self> {
        // Parse output like:
        // rustc 1.90.0 (4b06a43a1 2025-08-07)
        
        let regex = regex::Regex::new(
            r"rustc (\d+\.\d+\.\d+(?:-[\w.]+)?) \(([a-f0-9]+) (\d{4}-\d{2}-\d{2})\)"
        )?;
        
        let captures = regex.captures(version_output)
            .ok_or_else(|| Error::ParseError("Invalid rustc version output"))?;
            
        Ok(Self {
            version: semver::Version::parse(&captures[1])?,
            commit_hash: captures[2].to_string(),
            commit_date: NaiveDate::parse_from_str(&captures[3], "%Y-%m-%d")?,
            host: detect_host()?,
            release_channel: detect_channel(&captures[1])?,
        })
    }
}
```

---

## Version Comparison Engine

```rust
pub struct VersionComparator {
    current: RustVersion,
    available: Vec<GitHubRelease>,
}

impl VersionComparator {
    pub fn new(current: RustVersion, available: Vec<GitHubRelease>) -> Self {
        Self { current, available }
    }
    
    /// Check if update is available
    pub fn has_update(&self) -> bool {
        self.get_latest_applicable()
            .map(|latest| latest.parsed_version > self.current.version)
            .unwrap_or(false)
    }
    
    /// Get recommended update
    pub fn get_recommendation(&self) -> UpdateRecommendation {
        let latest = match self.get_latest_applicable() {
            Some(r) => r,
            None => return UpdateRecommendation::UpToDate,
        };
        
        if self.has_security_update() {
            UpdateRecommendation::SecurityUpdate(latest.clone())
        } else if self.has_breaking_changes() {
            UpdateRecommendation::MajorUpdate(latest.clone())
        } else if latest.parsed_version > self.current.version {
            UpdateRecommendation::MinorUpdate(latest.clone())
        } else {
            UpdateRecommendation::UpToDate
        }
    }
    
    /// Check for security updates
    pub fn has_security_update(&self) -> bool {
        self.available.iter()
            .filter(|r| r.parsed_version > self.current.version)
            .any(|r| {
                r.body.to_lowercase().contains("security") ||
                r.name.to_lowercase().contains("security")
            })
    }
    
    /// Check for breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        self.available.iter()
            .filter(|r| r.parsed_version > self.current.version)
            .any(|r| {
                r.parsed_version.major > self.current.version.major ||
                r.body.to_lowercase().contains("breaking")
            })
    }
    
    fn get_latest_applicable(&self) -> Option<&GitHubRelease> {
        self.available.iter()
            .filter(|r| {
                match self.current.release_channel {
                    Channel::Stable => r.channel == ReleaseChannel::Stable,
                    Channel::Beta => r.channel != ReleaseChannel::Nightly,
                    Channel::Nightly => true,
                    _ => r.channel == ReleaseChannel::Stable,
                }
            })
            .max_by_key(|r| &r.parsed_version)
    }
}

#[derive(Debug, Clone)]
pub enum UpdateRecommendation {
    UpToDate,
    MinorUpdate(GitHubRelease),
    MajorUpdate(GitHubRelease),
    SecurityUpdate(GitHubRelease),
}
```

---

## Cache System

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Cache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    inserted: Instant,
}

impl<K: Eq + Hash, V: Clone> Cache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.get(key).and_then(|entry| {
            if entry.inserted.elapsed() < self.ttl {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }
    
    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(key, CacheEntry {
            value,
            inserted: Instant::now(),
        });
    }
    
    pub fn invalidate(&mut self, key: &K) {
        self.entries.remove(key);
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| {
            now.duration_since(entry.inserted) < self.ttl
        });
    }
}
```

---

## Update Flow Orchestrator

```rust
pub struct UpdateOrchestrator {
    github_client: GitHubClient,
    rustup_client: RustupClient,
    config: Config,
}

impl UpdateOrchestrator {
    pub async fn check_for_updates(&self) -> Result<UpdateStatus> {
        // 1. Get current version
        let current = self.rustup_client.get_active_toolchain()?;
        let current_version = self.rustup_client
            .get_toolchain_version(&current.name)?;
            
        // 2. Fetch available releases
        let releases = self.github_client.get_releases(20, 1).await?;
        
        // 3. Compare versions
        let comparator = VersionComparator::new(current_version, releases);
        
        // 4. Generate recommendation
        let recommendation = comparator.get_recommendation();
        
        Ok(UpdateStatus {
            current: current_version,
            recommendation,
            last_checked: Utc::now(),
        })
    }
    
    pub async fn perform_update(&self, version: &str) -> Result<()> {
        // 1. Validate version exists
        let release = self.github_client
            .get_release_by_version(version)
            .await?
            .ok_or_else(|| Error::VersionNotFound(version.to_string()))?;
            
        // 2. Determine toolchain string
        let toolchain = match release.channel {
            ReleaseChannel::Stable => "stable".to_string(),
            ReleaseChannel::Beta => "beta".to_string(),
            ReleaseChannel::Nightly => format!("nightly-{}", release.published_at
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "latest".to_string())),
        };
        
        // 3. Install via rustup
        println!("Installing Rust {} via rustup...", version);
        self.rustup_client.install_toolchain(&toolchain).await?;
        
        // 4. Optionally set as default
        if self.config.rust.auto_set_default {
            self.rustup_client.set_default(&toolchain)?;
        }
        
        Ok(())
    }
    
    pub async fn schedule_periodic_checks(&self) {
        let interval = self.config.version_management.check_interval;
        
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);
            
            loop {
                timer.tick().await;
                
                if let Ok(status) = self.check_for_updates().await {
                    match status.recommendation {
                        UpdateRecommendation::SecurityUpdate(release) => {
                            println!("🚨 Security update available: {}", release.tag_name);
                            self.notify_security_update(&release).await;
                        }
                        UpdateRecommendation::MajorUpdate(release) => {
                            println!("📦 Major update available: {}", release.tag_name);
                        }
                        UpdateRecommendation::MinorUpdate(release) => {
                            println!("✨ Update available: {}", release.tag_name);
                        }
                        UpdateRecommendation::UpToDate => {
                            // Silent when up to date
                        }
                    }
                }
            }
        });
    }
}
```

---

## Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("GitHub API error: {0}")]
    GitHubApi(#[from] reqwest::Error),
    
    #[error("Rate limited. Retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },
    
    #[error("Rustup not found. Please install rustup from https://rustup.rs")]
    RustupNotFound,
    
    #[error("Rustup command failed: {0}")]
    RustupCommand(String),
    
    #[error("Failed to parse version: {0}")]
    VersionParse(#[from] semver::Error),
    
    #[error("Version {0} not found in releases")]
    VersionNotFound(String),
    
    #[error("Toolchain installation failed: {0}")]
    ToolchainInstallFailed(String),
    
    #[error("Parse error: {0}")]
    ParseError(&'static str),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}
```

---

## CLI Integration

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ferrous-forge")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Rust version management
    Rust {
        #[command(subcommand)]
        action: RustActions,
    },
}

#[derive(Subcommand)]
pub enum RustActions {
    /// Check current Rust installation and available updates
    Check {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
        
        /// Check specific channel
        #[arg(short, long)]
        channel: Option<String>,
    },
    
    /// Update Rust to latest version
    Update {
        /// Update to specific version
        #[arg(short, long)]
        version: Option<String>,
        
        /// Update to specific channel
        #[arg(short, long)]
        channel: Option<String>,
        
        /// Set as default toolchain
        #[arg(short, long)]
        set_default: bool,
    },
    
    /// Get update recommendations
    Recommend {
        /// Consider only stable releases
        #[arg(short, long)]
        stable_only: bool,
        
        /// Include pre-releases
        #[arg(short, long)]
        include_prerelease: bool,
    },
    
    /// List available Rust versions
    List {
        /// Number of versions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
        
        /// Filter by channel
        #[arg(short, long)]
        channel: Option<String>,
    },
}

pub async fn handle_rust_command(action: RustActions) -> Result<()> {
    let orchestrator = UpdateOrchestrator::new().await?;
    
    match action {
        RustActions::Check { verbose, channel } => {
            let status = orchestrator.check_for_updates().await?;
            
            if verbose {
                println!("{:#?}", status);
            } else {
                println!("Current: {}", status.current.version);
                match status.recommendation {
                    UpdateRecommendation::UpToDate => {
                        println!("✅ You're up to date!");
                    }
                    UpdateRecommendation::MinorUpdate(r) |
                    UpdateRecommendation::MajorUpdate(r) |
                    UpdateRecommendation::SecurityUpdate(r) => {
                        println!("📦 Update available: {}", r.tag_name);
                        println!("Run `ferrous-forge rust update` to upgrade");
                    }
                }
            }
        }
        RustActions::Update { version, channel, set_default } => {
            let version = version.unwrap_or_else(|| {
                // Get latest stable by default
                "stable".to_string()
            });
            
            orchestrator.perform_update(&version).await?;
            println!("✅ Successfully updated to Rust {}", version);
        }
        RustActions::Recommend { stable_only, include_prerelease } => {
            // Implementation
        }
        RustActions::List { limit, channel } => {
            // Implementation
        }
    }
    
    Ok(())
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        let output = "rustc 1.90.0 (4b06a43a1 2025-08-07)";
        let version = RustVersion::parse(output).unwrap();
        
        assert_eq!(version.version, semver::Version::new(1, 90, 0));
        assert_eq!(version.commit_hash, "4b06a43a1");
    }
    
    #[test]
    fn test_release_channel_detection() {
        assert_eq!(detect_channel("1.90.0"), Ok(Channel::Stable));
        assert_eq!(detect_channel("1.91.0-beta.1"), Ok(Channel::Beta));
        assert_eq!(detect_channel("1.92.0-nightly"), Ok(Channel::Nightly));
    }
    
    #[tokio::test]
    async fn test_github_api_mock() {
        let mock_server = mockito::Server::new();
        let mock = mock_server.mock("GET", "/repos/rust-lang/rust/releases/latest")
            .with_header("content-type", "application/json")
            .with_body(include_str!("../fixtures/release.json"))
            .create();
            
        let client = GitHubClient::new_with_base_url(
            None,
            &mock_server.url()
        ).unwrap();
        
        let release = client.get_latest_release().await.unwrap();
        assert_eq!(release.tag_name, "1.90.0");
        
        mock.assert();
    }
    
    #[test]
    fn test_cache_expiry() {
        let mut cache = Cache::new(Duration::from_millis(100));
        cache.insert("key", "value");
        
        assert_eq!(cache.get(&"key"), Some("value"));
        
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key"), None);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn test_real_github_api() {
        let client = GitHubClient::new(None).unwrap();
        let release = client.get_latest_release().await.unwrap();
        
        assert!(!release.tag_name.is_empty());
        assert!(release.parsed_version.major >= 1);
    }
    
    #[test]
    #[ignore]
    fn test_real_rustup() {
        let client = RustupClient::new().unwrap();
        let toolchains = client.list_toolchains().unwrap();
        
        assert!(!toolchains.is_empty());
    }
}
```

---

## Performance Considerations

1. **Caching Strategy**:
   - Cache GitHub API responses for 1 hour
   - Cache rustup queries for 5 minutes
   - Persistent disk cache for offline mode

2. **Rate Limiting**:
   - Respect GitHub's rate limits (60/hour unauthenticated, 5000/hour authenticated)
   - Implement exponential backoff
   - Queue requests to avoid bursts

3. **Parallel Operations**:
   - Fetch GitHub releases in parallel
   - Run rustup commands sequentially (avoid conflicts)
   - Background periodic checks

4. **Memory Management**:
   - Stream large responses
   - Limit cache size
   - Clean up old cache entries

---

## Security Considerations

1. **API Token Management**:
   - Store tokens in system keychain
   - Never log tokens
   - Validate token permissions

2. **HTTPS Only**:
   - Enforce TLS for all API calls
   - Certificate pinning for critical endpoints

3. **Input Validation**:
   - Sanitize version strings
   - Validate toolchain names
   - Prevent command injection

4. **Update Verification**:
   - Verify release signatures (when available)
   - Check commit hashes
   - Validate release authors

---

## Conclusion

This API integration design provides a robust foundation for Ferrous Forge to interact with GitHub's Rust releases and the rustup toolchain manager. The modular architecture allows for easy testing, maintenance, and future enhancements.

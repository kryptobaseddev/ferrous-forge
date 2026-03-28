//! GitHub client for fetching templates
//!
//! @task T021
//! @epic T014

use crate::error::{Error, Result};
use crate::templates::manifest::TemplateManifest;
use crate::templates::repository::{CachedTemplate, TemplateRepository};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

/// GitHub API client for template fetching
pub struct GitHubClient {
    client: reqwest::Client,
    api_base: String,
}

/// GitHub repository reference
#[derive(Debug, Clone)]
pub struct RepoRef {
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Git reference (branch, tag, or commit)
    pub git_ref: Option<String>,
}

/// GitHub repository information
#[derive(Debug, Deserialize)]
struct GitHubRepo {
    #[allow(dead_code)]
    id: u64,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    full_name: String,
    #[allow(dead_code)]
    description: Option<String>,
    #[serde(rename = "stargazers_count")]
    #[allow(dead_code)]
    stars: u32,
    #[serde(rename = "updated_at")]
    #[allow(dead_code)]
    updated_at: String,
    default_branch: String,
}

/// GitHub tree entry
#[derive(Debug, Deserialize)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
    #[allow(dead_code)]
    sha: String,
    #[allow(dead_code)]
    size: Option<u64>,
}

/// GitHub tree response
#[derive(Debug, Deserialize)]
struct GitHubTree {
    tree: Vec<TreeEntry>,
    truncated: bool,
}

impl GitHubClient {
    /// Create a new GitHub client
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("ferrous-forge-template-fetcher/1.0")
            .build()
            .map_err(|e| Error::network(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            client,
            api_base: "https://api.github.com".to_string(),
        })
    }

    /// Parse a repository reference from string
    ///
    /// Supports formats:
    /// - `gh:owner/repo` - default branch
    /// - `gh:owner/repo@ref` - specific ref
    /// - `owner/repo` - without gh: prefix
    /// - `owner/repo@ref` - without gh: prefix
    ///
    /// # Errors
    ///
    /// Returns an error if the input format is invalid.
    pub fn parse_repo_ref(input: &str) -> Result<RepoRef> {
        let input = input.strip_prefix("gh:").unwrap_or(input);

        let parts: Vec<&str> = input.split('@').collect();
        let repo_part = parts[0];
        let git_ref = parts.get(1).map(|s| s.to_string());

        let repo_parts: Vec<&str> = repo_part.split('/').collect();
        if repo_parts.len() != 2 {
            return Err(Error::template(format!(
                "Invalid repository format: '{input}'. Use owner/repo or gh:owner/repo"
            )));
        }

        Ok(RepoRef {
            owner: repo_parts[0].to_string(),
            repo: repo_parts[1].to_string(),
            git_ref,
        })
    }

    /// Fetch template from GitHub repository
    ///
    /// # Errors
    ///
    /// Returns an error if the GitHub API request fails or the template cannot be fetched.
    pub async fn fetch_template(
        &self,
        repo_ref: &RepoRef,
        repository: &mut TemplateRepository,
    ) -> Result<CachedTemplate> {
        // Get repository info
        let repo_info = self.get_repo_info(repo_ref).await?;
        let git_ref = repo_ref
            .git_ref
            .clone()
            .unwrap_or_else(|| repo_info.default_branch.clone());

        // Check if already cached and up to date
        let cache_name = format!("{}-{}", repo_ref.owner, repo_ref.repo);
        if let Some(cached) = repository.get_cached(&cache_name) {
            // If we have a specific ref, check if it matches
            if repo_ref.git_ref.is_some() && cached.version == git_ref {
                return Ok(cached.clone());
            }
            // For default branch, check update time
            if repo_ref.git_ref.is_none() && !cached.needs_update() {
                return Ok(cached.clone());
            }
        }

        // Fetch template files
        let template_dir = repository.template_cache_path(&cache_name);
        self.download_template(repo_ref, &git_ref, &template_dir)
            .await?;

        // Load and validate manifest
        let manifest_path = template_dir.join("template.toml");
        let manifest_content = tokio::fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| Error::template(format!("Failed to read template manifest: {e}")))?;
        let manifest: TemplateManifest = toml::from_str(&manifest_content)
            .map_err(|e| Error::template(format!("Failed to parse template manifest: {e}")))?;

        // Validate template
        validate_template_structure(&template_dir, &manifest).await?;

        // Create cached template entry
        let cached = CachedTemplate {
            name: cache_name.clone(),
            source: format!("gh:{}/{}", repo_ref.owner, repo_ref.repo),
            version: git_ref,
            fetched_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            cache_path: template_dir,
            manifest,
        };

        // Add to cache index
        repository.add_to_cache(cached.clone())?;

        Ok(cached)
    }

    /// Get repository information from GitHub API
    async fn get_repo_info(&self, repo_ref: &RepoRef) -> Result<GitHubRepo> {
        let url = format!(
            "{}/repos/{}/{}",
            self.api_base, repo_ref.owner, repo_ref.repo
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to fetch repository info: {e}")))?;

        if response.status() == 404 {
            return Err(Error::template(format!(
                "Repository not found: {}/{}",
                repo_ref.owner, repo_ref.repo
            )));
        }

        if response.status() == 403 {
            return Err(Error::template(
                "GitHub API rate limit exceeded. Please try again later or provide a GitHub token.",
            ));
        }

        if !response.status().is_success() {
            return Err(Error::template(format!(
                "GitHub API error: {}",
                response.status()
            )));
        }

        let repo: GitHubRepo = response
            .json()
            .await
            .map_err(|e| Error::template(format!("Failed to parse repository info: {e}")))?;

        Ok(repo)
    }

    /// Download template files from GitHub
    async fn download_template(
        &self,
        repo_ref: &RepoRef,
        git_ref: &str,
        target_dir: &Path,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        // Get repository tree
        let tree = self.get_tree(repo_ref, git_ref).await?;

        // Clean target directory
        if target_dir.exists() {
            tokio::fs::remove_dir_all(target_dir)
                .await
                .map_err(|e| Error::template(format!("Failed to clean template directory: {e}")))?;
        }

        tokio::fs::create_dir_all(target_dir)
            .await
            .map_err(|e| Error::template(format!("Failed to create template directory: {e}")))?;

        // Download each file
        for entry in tree.tree {
            if entry.entry_type != "blob" {
                continue;
            }

            let file_path = target_dir.join(&entry.path);
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| Error::template(format!("Failed to create directory: {e}")))?;
            }

            let content = self
                .fetch_file_content(repo_ref, git_ref, &entry.path)
                .await?;

            let mut file = tokio::fs::File::create(&file_path)
                .await
                .map_err(|e| Error::template(format!("Failed to create file: {e}")))?;
            file.write_all(&content)
                .await
                .map_err(|e| Error::template(format!("Failed to write file: {e}")))?;
        }

        Ok(())
    }

    /// Get repository tree from GitHub API
    async fn get_tree(&self, repo_ref: &RepoRef, git_ref: &str) -> Result<GitHubTree> {
        let url = format!(
            "{}/repos/{}/{}/git/trees/{}?recursive=1",
            self.api_base, repo_ref.owner, repo_ref.repo, git_ref
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to fetch repository tree: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::template(format!(
                "Failed to fetch repository tree: {}",
                response.status()
            )));
        }

        let tree: GitHubTree = response
            .json()
            .await
            .map_err(|e| Error::template(format!("Failed to parse repository tree: {e}")))?;

        if tree.truncated {
            tracing::warn!("Repository tree was truncated, some files may be missing");
        }

        Ok(tree)
    }

    /// Fetch file content from GitHub
    async fn fetch_file_content(
        &self,
        repo_ref: &RepoRef,
        git_ref: &str,
        path: &str,
    ) -> Result<Vec<u8>> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            repo_ref.owner, repo_ref.repo, git_ref, path
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to fetch file {path}: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::template(format!(
                "Failed to fetch file {path}: {}",
                response.status()
            )));
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| Error::network(format!("Failed to read file content: {e}")))?;

        Ok(content.to_vec())
    }
}

/// Validate template structure before installation
async fn validate_template_structure(
    template_dir: &Path,
    manifest: &TemplateManifest,
) -> Result<()> {
    use crate::templates::validation::validate_template;

    // Run standard template validation
    validate_template(template_dir, manifest).await?;

    Ok(())
}

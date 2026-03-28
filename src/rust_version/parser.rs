//! Release notes parser for extracting security updates and breaking changes
//!
//! Parses GitHub release notes to identify security advisories,
//! breaking changes, and other important information.
//!
//! @task T024
//! @epic T014

use regex::Regex;
use std::sync::OnceLock;

/// Parsed release information
#[derive(Debug, Clone)]
pub struct ParsedRelease {
    /// Version string
    pub version: String,
    /// Full release notes
    pub full_notes: String,
    /// Security advisories found
    pub security_advisories: Vec<SecurityAdvisory>,
    /// Breaking changes
    pub breaking_changes: Vec<BreakingChange>,
    /// New features
    pub new_features: Vec<String>,
    /// Performance improvements
    pub performance_improvements: Vec<String>,
    /// Bug fixes
    pub bug_fixes: Vec<String>,
}

/// Security advisory information
#[derive(Debug, Clone)]
pub struct SecurityAdvisory {
    /// Advisory ID (e.g., CVE number)
    pub id: Option<String>,
    /// Description of the vulnerability
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Affected components
    pub affected_components: Vec<String>,
}

/// Breaking change information
#[derive(Debug, Clone)]
pub struct BreakingChange {
    /// Description of the change
    pub description: String,
    /// Migration guidance
    pub migration: Option<String>,
    /// Affected edition
    pub affected_edition: Option<String>,
}

/// Severity level for security issues
///
/// Variants are ordered so that `Critical > High > Medium > Low > Unknown`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Unknown severity
    Unknown,
    /// Low - informational
    Low,
    /// Medium - update when convenient
    Medium,
    /// High - should update soon
    High,
    /// Critical - immediate action required
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::High => write!(f, "HIGH"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::Low => write!(f, "LOW"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Security keywords for detection
#[allow(clippy::panic)] // Hardcoded regex patterns are programmer-verified
fn security_keywords() -> &'static [Regex] {
    static KEYWORDS: OnceLock<Vec<Regex>> = OnceLock::new();
    KEYWORDS.get_or_init(|| {
        vec![
            Regex::new(r"(?i)security")
                .unwrap_or_else(|_| panic!("Invalid regex pattern for security")),
            Regex::new(r"(?i)vulnerability")
                .unwrap_or_else(|_| panic!("Invalid regex pattern for vulnerability")),
            Regex::new(r"(?i)CVE-\d{4}-\d+")
                .unwrap_or_else(|_| panic!("Invalid regex pattern for CVE")),
            Regex::new(r"(?i)exploit")
                .unwrap_or_else(|_| panic!("Invalid regex pattern for exploit")),
            Regex::new(r"(?i)buffer.?overflow")
                .unwrap_or_else(|_| panic!("Invalid regex pattern for buffer overflow")),
            Regex::new(r"(?i)memory.?safety")
                .unwrap_or_else(|_| panic!("Invalid regex pattern for memory safety")),
            Regex::new(r"(?i)unsound")
                .unwrap_or_else(|_| panic!("Invalid regex pattern for unsound")),
            Regex::new(r"(?i)undefined.?behavior")
                .unwrap_or_else(|_| panic!("Invalid regex pattern for undefined behavior")),
        ]
    })
}

/// Breaking change keywords for detection
#[allow(clippy::panic)] // Hardcoded regex patterns are programmer-verified
fn breaking_keywords() -> &'static [Regex] {
    static KEYWORDS: OnceLock<Vec<Regex>> = OnceLock::new();
    KEYWORDS.get_or_init(|| {
        vec![
            Regex::new(r"(?i)breaking.?change")
                .unwrap_or_else(|_| panic!("Invalid regex for breaking change")),
            Regex::new(r"(?i)\[breaking\]")
                .unwrap_or_else(|_| panic!("Invalid regex for [breaking]")),
            Regex::new(r"(?i)incompatible")
                .unwrap_or_else(|_| panic!("Invalid regex for incompatible")),
            Regex::new(r"(?i)deprecated")
                .unwrap_or_else(|_| panic!("Invalid regex for deprecated")),
            Regex::new(r"(?i)removed").unwrap_or_else(|_| panic!("Invalid regex for removed")),
        ]
    })
}

/// Feature keywords
#[allow(dead_code)]
#[allow(clippy::panic)] // Hardcoded regex patterns are programmer-verified
fn feature_keywords() -> &'static [Regex] {
    static KEYWORDS: OnceLock<Vec<Regex>> = OnceLock::new();
    KEYWORDS.get_or_init(|| {
        vec![
            Regex::new(r"(?i)new.?feature")
                .unwrap_or_else(|_| panic!("Invalid regex for new feature")),
            Regex::new(r"(?i)stabilized")
                .unwrap_or_else(|_| panic!("Invalid regex for stabilized")),
            Regex::new(r"(?i)added.?support")
                .unwrap_or_else(|_| panic!("Invalid regex for added support")),
        ]
    })
}

/// Performance keywords
#[allow(dead_code)]
#[allow(clippy::panic)] // Hardcoded regex patterns are programmer-verified
fn performance_keywords() -> &'static [Regex] {
    static KEYWORDS: OnceLock<Vec<Regex>> = OnceLock::new();
    KEYWORDS.get_or_init(|| {
        vec![
            Regex::new(r"(?i)performance")
                .unwrap_or_else(|_| panic!("Invalid regex for performance")),
            Regex::new(r"(?i)faster").unwrap_or_else(|_| panic!("Invalid regex for faster")),
            Regex::new(r"(?i)optimized").unwrap_or_else(|_| panic!("Invalid regex for optimized")),
            Regex::new(r"(?i)improved.?compile")
                .unwrap_or_else(|_| panic!("Invalid regex for improved compile")),
        ]
    })
}

/// Parse release notes and extract structured information
///
/// # Examples
///
/// ```
/// # use ferrous_forge::rust_version::parser::parse_release_notes;
/// let notes = "Rust 1.70.0\n\nSecurity:\n- Fixed CVE-2023-1234 buffer overflow\n\nBreaking Changes:\n- Deprecated old API";
/// let parsed = parse_release_notes("1.70.0", notes);
/// assert_eq!(parsed.version, "1.70.0");
/// assert!(!parsed.security_advisories.is_empty());
/// ```
pub fn parse_release_notes(version: &str, notes: &str) -> ParsedRelease {
    let mut parsed = ParsedRelease {
        version: version.to_string(),
        full_notes: notes.to_string(),
        security_advisories: Vec::new(),
        breaking_changes: Vec::new(),
        new_features: Vec::new(),
        performance_improvements: Vec::new(),
        bug_fixes: Vec::new(),
    };

    let lines: Vec<&str> = notes.lines().collect();
    let mut current_section: Option<&str> = None;

    for line in lines {
        let trimmed = line.trim();

        // Detect section headers
        if trimmed.starts_with("#") || trimmed.ends_with(':') {
            current_section = Some(trimmed.trim_start_matches('#').trim());
            continue;
        }

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Parse based on current section or content
        if is_security_related(trimmed)
            && let Some(advisory) = parse_security_advisory(trimmed)
        {
            parsed.security_advisories.push(advisory);
        }

        if is_breaking_change(trimmed)
            && let Some(change) = parse_breaking_change(trimmed)
        {
            parsed.breaking_changes.push(change);
        }

        // Categorize by section if detected
        if let Some(section) = current_section {
            categorize_by_section(&mut parsed, section, trimmed);
        }
    }

    parsed
}

/// Check if line contains security-related content
fn is_security_related(line: &str) -> bool {
    let lower = line.to_lowercase();
    security_keywords().iter().any(|re| re.is_match(&lower))
}

/// Check if line indicates a breaking change
fn is_breaking_change(line: &str) -> bool {
    let lower = line.to_lowercase();
    breaking_keywords().iter().any(|re| re.is_match(&lower))
}

/// Parse a security advisory from a line
fn parse_security_advisory(line: &str) -> Option<SecurityAdvisory> {
    let line_lower = line.to_lowercase();

    // Extract CVE ID
    let id = extract_cve_id(line);

    // Determine severity
    let severity = if line_lower.contains("critical") || line_lower.contains("severe") {
        Severity::Critical
    } else if line_lower.contains("high") {
        Severity::High
    } else if line_lower.contains("medium") || line_lower.contains("moderate") {
        Severity::Medium
    } else if line_lower.contains("low") {
        Severity::Low
    } else {
        Severity::Unknown
    };

    // Extract description (remove bullet points and IDs)
    let description = line.trim_start_matches(['-', '*', '•']).trim().to_string();

    Some(SecurityAdvisory {
        id,
        description,
        severity,
        affected_components: Vec::new(),
    })
}

/// Extract CVE ID from text
fn extract_cve_id(text: &str) -> Option<String> {
    // CVE pattern is hardcoded and validated - use unwrap_or with empty fallback
    let re = Regex::new(r"CVE-\d{4}-\d+").unwrap_or_else(|_| {
        // This should never happen with a hardcoded valid regex
        Regex::new(r"$^").unwrap_or_else(|_| unreachable!())
    });
    re.find(text).map(|m| m.as_str().to_string())
}

/// Parse a breaking change from a line
fn parse_breaking_change(line: &str) -> Option<BreakingChange> {
    let description = line.trim_start_matches(['-', '*', '•']).trim().to_string();

    // Try to detect migration guidance
    let migration =
        if line.to_lowercase().contains("use") || line.to_lowercase().contains("replace") {
            Some(description.clone())
        } else {
            None
        };

    Some(BreakingChange {
        description,
        migration,
        affected_edition: None,
    })
}

/// Categorize content based on section header
fn categorize_by_section(parsed: &mut ParsedRelease, section: &str, content: &str) {
    let section_lower = section.to_lowercase();

    if section_lower.contains("feature") || section_lower.contains("language") {
        parsed.new_features.push(content.to_string());
    } else if section_lower.contains("performance") || section_lower.contains("compile") {
        parsed.performance_improvements.push(content.to_string());
    } else if section_lower.contains("bug") || section_lower.contains("fix") {
        parsed.bug_fixes.push(content.to_string());
    }
}

/// Check if a version has critical security issues
///
/// # Examples
///
/// ```
/// # use ferrous_forge::rust_version::parser::has_critical_security_issues;
/// let notes = "Security: Fixed CRITICAL vulnerability";
/// assert!(has_critical_security_issues(notes));
/// ```
pub fn has_critical_security_issues(notes: &str) -> bool {
    let parsed = parse_release_notes("", notes);
    parsed
        .security_advisories
        .iter()
        .any(|a| a.severity == Severity::Critical)
}

/// Get security summary for a release
///
/// Returns a human-readable summary of security issues.
///
/// # Examples
///
/// ```
/// # use ferrous_forge::rust_version::parser::get_security_summary;
/// let notes = "CVE-2023-1234: Security fix\nCVE-2023-5678: Another fix";
/// let summary = get_security_summary(notes);
/// assert!(summary.contains("2 security"));
/// ```
pub fn get_security_summary(notes: &str) -> String {
    let parsed = parse_release_notes("", notes);

    if parsed.security_advisories.is_empty() {
        return "No security advisories".to_string();
    }

    let critical_count = parsed
        .security_advisories
        .iter()
        .filter(|a| a.severity == Severity::Critical)
        .count();
    let high_count = parsed
        .security_advisories
        .iter()
        .filter(|a| a.severity == Severity::High)
        .count();

    let mut summary = format!("{} security advisory", parsed.security_advisories.len());
    if parsed.security_advisories.len() > 1 {
        summary.push('s');
    }

    if critical_count > 0 {
        summary.push_str(&format!(", {} CRITICAL", critical_count));
    }
    if high_count > 0 {
        summary.push_str(&format!(", {} HIGH", high_count));
    }

    summary
}

/// Check if current version is affected by security advisories
///
/// Compares current version against releases with security fixes.
///
/// # Arguments
///
/// * `current_version` - The currently installed Rust version
/// * `releases` - List of recent releases to check
///
/// # Returns
///
/// Returns true if the current version is missing security updates.
pub fn is_version_affected(
    current_version: &str,
    releases: &[crate::rust_version::GitHubRelease],
) -> bool {
    let Ok(current) = semver::Version::parse(current_version.trim_start_matches('v')) else {
        return false;
    };

    for release in releases {
        // Only check releases newer than current
        if release.version > current {
            let parsed = parse_release_notes(&release.tag_name, &release.body);
            if !parsed.security_advisories.is_empty() {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_security_advisory() {
        let line = "- Fixed CVE-2023-1234: Critical buffer overflow vulnerability";
        let advisory = parse_security_advisory(line).unwrap();

        assert_eq!(advisory.id, Some("CVE-2023-1234".to_string()));
        assert_eq!(advisory.severity, Severity::Critical);
        assert!(advisory.description.contains("buffer overflow"));
    }

    #[test]
    fn test_extract_cve_id() {
        assert_eq!(
            extract_cve_id("Fixed CVE-2023-1234 issue"),
            Some("CVE-2023-1234".to_string())
        );
        assert_eq!(extract_cve_id("No CVE here"), None);
    }

    #[test]
    fn test_has_critical_security_issues() {
        assert!(has_critical_security_issues(
            "Security: Fixed CRITICAL vulnerability"
        ));
        assert!(!has_critical_security_issues("Added new feature"));
    }

    #[test]
    fn test_get_security_summary() {
        let notes = "CVE-2023-1234: High severity\nCVE-2023-5678: Critical severity";
        let summary = get_security_summary(notes);

        assert!(summary.contains("2 security"));
        assert!(summary.contains("1 CRITICAL"));
        assert!(summary.contains("1 HIGH"));
    }

    #[test]
    fn test_is_security_related() {
        assert!(is_security_related("Fixed security vulnerability"));
        assert!(is_security_related("CVE-2023-1234 buffer overflow"));
        assert!(!is_security_related("Added new feature"));
    }

    #[test]
    fn test_is_breaking_change() {
        assert!(is_breaking_change("[Breaking] Removed old API"));
        assert!(is_breaking_change("Deprecated function"));
        assert!(!is_breaking_change("Bug fix"));
    }

    #[test]
    fn test_parse_breaking_change() {
        let line = "- Deprecated std::mem::uninitialized()";
        let change = parse_breaking_change(line).unwrap();

        assert!(change.description.contains("uninitialized"));
    }

    #[test]
    fn test_parse_release_notes_comprehensive() {
        let notes = r#"Rust 1.70.0

## Security
- Fixed CVE-2023-1234: Critical buffer overflow (CVE-2023-1234)
- Addressed CVE-2023-5678: HIGH severity memory safety issue

## Breaking Changes
- Deprecated old API

## Language
- Stabilized new features

## Performance
- Improved compile times
"#;

        let parsed = parse_release_notes("1.70.0", notes);

        assert_eq!(parsed.version, "1.70.0");
        assert_eq!(parsed.security_advisories.len(), 2);
        assert_eq!(parsed.breaking_changes.len(), 1);
        assert!(!parsed.new_features.is_empty());
        assert!(!parsed.performance_improvements.is_empty());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }
}

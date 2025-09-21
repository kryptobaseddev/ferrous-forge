//! Default implementations for coding standards

use super::types::*;

impl Default for CodingStandards {
    fn default() -> Self {
        Self {
            edition: EditionStandards {
                required_edition: "2024".to_string(),
                min_rust_version: "1.82.0".to_string(),
                auto_upgrade: false,
            },
            file_limits: FileLimits {
                max_lines: 300,
                max_line_length: 100,
                exempt_files: vec!["tests.rs".to_string(), "benches.rs".to_string()],
            },
            function_limits: FunctionLimits {
                max_lines: 50,
                max_complexity: 10,
                exempt_functions: vec!["main".to_string(), "test_*".to_string()],
            },
            documentation: DocumentationStandards {
                require_public_docs: true,
                require_private_docs: false,
                min_coverage: 80.0,
                require_examples: false,
            },
            banned_patterns: BannedPatterns {
                ban_unwrap: true,
                ban_expect: true,
                ban_panic: true,
                ban_todo: true,
                ban_unimplemented: true,
                ban_underscore_bandaid: true,
                custom_patterns: vec![
                    BannedPattern {
                        name: "print_debug".to_string(),
                        pattern: r"(println!|print!|eprintln!|eprint!|dbg!)".to_string(),
                        message: "Debug print statements should not be in production code"
                            .to_string(),
                        severity: "warning".to_string(),
                    },
                    BannedPattern {
                        name: "sleep_in_async".to_string(),
                        pattern: r"std::thread::sleep".to_string(),
                        message: "Use tokio::time::sleep in async code".to_string(),
                        severity: "error".to_string(),
                    },
                ],
            },
            dependencies: DependencyStandards {
                max_dependencies: 100,
                require_license_check: true,
                banned_licenses: vec!["GPL-3.0".to_string(), "AGPL-3.0".to_string()],
                require_msrv_compatible: true,
            },
            security: SecurityStandards {
                ban_unsafe: true,
                require_audit: true,
                max_cve_score: 7.0,
            },
        }
    }
}

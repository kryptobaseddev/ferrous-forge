//! Test coverage module tests

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_coverage_config_default() {
        let config = CoverageConfig::default();
        assert_eq!(config.min_line_coverage, 80.0);
        assert_eq!(config.min_function_coverage, 85.0);
        assert_eq!(config.min_branch_coverage, 75.0);
        assert!(config.fail_on_low_coverage);
    }

    #[test]
    fn test_coverage_analyzer_creation() {
        let analyzer = CoverageAnalyzer::new();
        assert_eq!(analyzer.config().min_line_coverage, 80.0);

        let custom_config = CoverageConfig {
            min_line_coverage: 90.0,
            ..Default::default()
        };
        let custom_analyzer = CoverageAnalyzer::with_config(custom_config);
        assert_eq!(custom_analyzer.config().min_line_coverage, 90.0);
    }

    #[test]
    fn test_validate_coverage_success() {
        let analyzer = CoverageAnalyzer::new();
        let report = CoverageReport {
            line_coverage: 85.0,
            function_coverage: 90.0,
            branch_coverage: 80.0,
            file_coverage: HashMap::new(),
            lines_tested: 85,
            total_lines: 100,
            functions_tested: 18,
            total_functions: 20,
            branches_tested: 8,
            total_branches: 10,
        };

        assert!(analyzer.validate_coverage(&report).is_ok());
    }

    #[test]
    fn test_validate_coverage_failure() {
        let analyzer = CoverageAnalyzer::new();
        let report = CoverageReport {
            line_coverage: 70.0, // Below 80% minimum
            function_coverage: 90.0,
            branch_coverage: 80.0,
            file_coverage: HashMap::new(),
            lines_tested: 70,
            total_lines: 100,
            functions_tested: 18,
            total_functions: 20,
            branches_tested: 8,
            total_branches: 10,
        };

        assert!(analyzer.validate_coverage(&report).is_err());
    }

    #[test]
    fn test_format_coverage_report() {
        let analyzer = CoverageAnalyzer::new();
        let report = CoverageReport {
            line_coverage: 85.0,
            function_coverage: 90.0,
            branch_coverage: 80.0,
            file_coverage: HashMap::new(),
            lines_tested: 85,
            total_lines: 100,
            functions_tested: 18,
            total_functions: 20,
            branches_tested: 8,
            total_branches: 10,
        };

        let formatted = analyzer.format_coverage_report(&report);
        assert!(formatted.contains("Test Coverage Report"));
        assert!(formatted.contains("85.0%"));
        assert!(formatted.contains("90.0%"));
        assert!(formatted.contains("80.0%"));
    }
}

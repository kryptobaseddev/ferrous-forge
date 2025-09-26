//! Tests for safety module

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

#[test]
fn test_pipeline_stage_from_str() {
    assert_eq!(
        "pre-commit".parse::<PipelineStage>().unwrap(),
        PipelineStage::PreCommit
    );
    assert_eq!(
        "pre-push".parse::<PipelineStage>().unwrap(),
        PipelineStage::PrePush
    );
    assert_eq!(
        "publish".parse::<PipelineStage>().unwrap(),
        PipelineStage::Publish
    );
    assert!("invalid".parse::<PipelineStage>().is_err());
}

#[test]
fn test_check_type_for_stage() {
    let pre_commit_checks = CheckType::for_stage(PipelineStage::PreCommit);
    assert!(pre_commit_checks.contains(&CheckType::Format));
    assert!(pre_commit_checks.contains(&CheckType::Clippy));
    assert!(!pre_commit_checks.contains(&CheckType::Test)); // Not in pre-commit

    let publish_checks = CheckType::for_stage(PipelineStage::Publish);
    assert!(publish_checks.contains(&CheckType::PublishDryRun));
    assert!(publish_checks.contains(&CheckType::Semver));
}

#[test]
fn test_safety_result_is_allowed() {
    assert!(SafetyResult::Passed.is_allowed());
    assert!(
        SafetyResult::Bypassed {
            reason: "test".to_string(),
            user: "test".to_string()
        }
        .is_allowed()
    );
    assert!(
        !SafetyResult::Blocked {
            failures: vec![],
            suggestions: vec![]
        }
        .is_allowed()
    );
}

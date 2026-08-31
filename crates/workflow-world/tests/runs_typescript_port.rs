use serde_json::json;
use workflow_world::runs::{
    BULK_CANCEL_MAX_RUN_IDS, BulkCancelWorkflowRunResult, BulkCancelWorkflowRunsRequest,
    BulkCancelWorkflowRunsResult,
};

#[test]
fn request_accepts_one_unique_id_with_an_optional_reason() {
    let request = BulkCancelWorkflowRunsRequest {
        run_ids: vec!["wrun_1".to_owned()],
        cancel_reason: Some("cleanup".to_owned()),
    };
    assert!(request.validate().is_ok());
    assert_eq!(
        serde_json::to_value(request).unwrap(),
        json!({ "runIds": ["wrun_1"], "cancelReason": "cleanup" })
    );
}

#[test]
fn request_accepts_the_maximum_number_of_ids() {
    let request = BulkCancelWorkflowRunsRequest {
        run_ids: (0..BULK_CANCEL_MAX_RUN_IDS)
            .map(|index| format!("wrun_{index}"))
            .collect(),
        cancel_reason: None,
    };
    assert!(request.validate().is_ok());
}

#[test]
fn request_rejects_an_empty_list() {
    let request = BulkCancelWorkflowRunsRequest {
        run_ids: Vec::new(),
        cancel_reason: None,
    };
    assert!(request.validate().is_err());
}

#[test]
fn request_rejects_duplicate_ids() {
    let request = BulkCancelWorkflowRunsRequest {
        run_ids: vec!["wrun_1".to_owned(), "wrun_1".to_owned()],
        cancel_reason: None,
    };
    assert!(request.validate().is_err());
}

#[test]
fn request_rejects_more_than_the_maximum_number_of_ids() {
    let request = BulkCancelWorkflowRunsRequest {
        run_ids: (0..=BULK_CANCEL_MAX_RUN_IDS)
            .map(|index| format!("wrun_{index}"))
            .collect(),
        cancel_reason: None,
    };
    assert!(request.validate().is_err());
}

#[test]
fn request_rejects_a_reason_longer_than_512_javascript_characters() {
    let request = BulkCancelWorkflowRunsRequest {
        run_ids: vec!["wrun_1".to_owned()],
        cancel_reason: Some("x".repeat(513)),
    };
    assert!(request.validate().is_err());
}

#[test]
fn parses_each_per_run_outcome_variant() {
    let fixtures = [
        json!({ "runId": "a", "outcome": "cancelled" }),
        json!({ "runId": "b", "outcome": "already_cancelled" }),
        json!({
            "runId": "c",
            "outcome": "not_cancellable",
            "status": "completed",
        }),
        json!({ "runId": "d", "outcome": "not_found" }),
        json!({
            "runId": "e",
            "outcome": "failed",
            "code": "internal_error",
            "retryable": true,
        }),
    ];

    for fixture in fixtures {
        let parsed =
            serde_json::from_value::<BulkCancelWorkflowRunResult>(fixture.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), fixture);
    }
}

#[test]
fn rejects_not_cancellable_without_a_status() {
    assert!(
        serde_json::from_value::<BulkCancelWorkflowRunResult>(json!({
            "runId": "c",
            "outcome": "not_cancellable",
        }))
        .is_err()
    );
}

#[test]
fn rejects_failed_without_code_and_retryable() {
    assert!(
        serde_json::from_value::<BulkCancelWorkflowRunResult>(json!({
            "runId": "e",
            "outcome": "failed",
        }))
        .is_err()
    );
}

#[test]
fn rejects_an_unknown_outcome() {
    assert!(
        serde_json::from_value::<BulkCancelWorkflowRunResult>(json!({
            "runId": "x",
            "outcome": "exploded",
        }))
        .is_err()
    );
}

#[test]
fn parses_and_validates_a_full_aggregate_result() {
    let value = json!({
        "summary": {
            "requested": 2,
            "cancelled": 1,
            "alreadyCancelled": 0,
            "notCancellable": 0,
            "notFound": 1,
            "failed": 0,
        },
        "results": [
            { "runId": "a", "outcome": "cancelled" },
            { "runId": "b", "outcome": "not_found" },
        ],
    });
    let parsed = serde_json::from_value::<BulkCancelWorkflowRunsResult>(value.clone()).unwrap();
    assert!(parsed.validate_consistency().is_ok());
    assert_eq!(serde_json::to_value(parsed).unwrap(), value);
}

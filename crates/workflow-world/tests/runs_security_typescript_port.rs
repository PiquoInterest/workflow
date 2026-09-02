use serde_json::json;
use workflow_world::runs::{
    BULK_CANCEL_MAX_RUN_IDS, BulkCancelWorkflowRunResult, BulkCancelWorkflowRunsResult,
    BulkCancelWorkflowRunsSummary,
};

fn valid_result() -> BulkCancelWorkflowRunsResult {
    BulkCancelWorkflowRunsResult {
        summary: BulkCancelWorkflowRunsSummary {
            requested: 3,
            cancelled: 1,
            already_cancelled: 0,
            not_cancellable: 1,
            not_found: 1,
            failed: 0,
        },
        results: vec![
            BulkCancelWorkflowRunResult::Cancelled {
                run_id: "wrun_cancelled".to_owned(),
            },
            BulkCancelWorkflowRunResult::NotCancellable {
                run_id: "wrun_terminal".to_owned(),
                status: "completed".to_owned(),
            },
            BulkCancelWorkflowRunResult::NotFound {
                run_id: "wrun_missing".to_owned(),
            },
        ],
    }
}

#[test]
fn accepts_an_exact_projection_of_the_per_run_outcomes() {
    assert!(valid_result().validate_consistency().is_ok());
}

#[test]
fn rejects_every_mismatched_summary_counter() {
    let mutations: [fn(&mut BulkCancelWorkflowRunsSummary); 6] = [
        |summary| summary.requested = 2,
        |summary| summary.cancelled = 2,
        |summary| summary.already_cancelled = 1,
        |summary| summary.not_cancellable = 0,
        |summary| summary.not_found = 0,
        |summary| summary.failed = 1,
    ];

    for mutate in mutations {
        let mut value = valid_result();
        mutate(&mut value.summary);
        assert_eq!(
            value.validate_consistency().unwrap_err().code(),
            "bulk_cancel_summary_mismatch"
        );
    }
}

#[test]
fn rejects_negative_and_fractional_summary_counts_at_deserialization() {
    for requested in [json!(-1), json!(1.5)] {
        let value = json!({
            "summary": {
                "requested": requested,
                "cancelled": 1,
                "alreadyCancelled": 0,
                "notCancellable": 1,
                "notFound": 1,
                "failed": 0,
            },
            "results": [
                { "runId": "a", "outcome": "cancelled" },
                {
                    "runId": "b",
                    "outcome": "not_cancellable",
                    "status": "completed",
                },
                { "runId": "c", "outcome": "not_found" },
            ],
        });
        assert!(serde_json::from_value::<BulkCancelWorkflowRunsResult>(value).is_err());
    }
}

#[test]
fn rejects_duplicate_run_ids_without_reflecting_the_sensitive_id() {
    let sensitive_run_id = "sensitive-run-id";
    let value = BulkCancelWorkflowRunsResult {
        summary: BulkCancelWorkflowRunsSummary {
            requested: 2,
            cancelled: 1,
            already_cancelled: 0,
            not_cancellable: 0,
            not_found: 1,
            failed: 0,
        },
        results: vec![
            BulkCancelWorkflowRunResult::Cancelled {
                run_id: sensitive_run_id.to_owned(),
            },
            BulkCancelWorkflowRunResult::NotFound {
                run_id: sensitive_run_id.to_owned(),
            },
        ],
    };
    let error = value.validate_consistency().unwrap_err();
    assert_eq!(error.code(), "bulk_cancel_results_duplicate");
    assert!(!error.message().contains(sensitive_run_id));
}

#[test]
fn rejects_an_empty_aggregate_response() {
    let value = BulkCancelWorkflowRunsResult {
        summary: BulkCancelWorkflowRunsSummary::default(),
        results: Vec::new(),
    };
    assert_eq!(
        value.validate_consistency().unwrap_err().code(),
        "bulk_cancel_results_empty"
    );
}

#[test]
fn rejects_more_results_than_one_valid_request_can_contain() {
    let results: Vec<_> = (0..=BULK_CANCEL_MAX_RUN_IDS)
        .map(|index| BulkCancelWorkflowRunResult::Cancelled {
            run_id: format!("wrun_{index}"),
        })
        .collect();
    let value = BulkCancelWorkflowRunsResult {
        summary: BulkCancelWorkflowRunsSummary {
            requested: results.len(),
            cancelled: results.len(),
            ..BulkCancelWorkflowRunsSummary::default()
        },
        results,
    };
    assert_eq!(
        value.validate_consistency().unwrap_err().code(),
        "bulk_cancel_results_too_many"
    );
}

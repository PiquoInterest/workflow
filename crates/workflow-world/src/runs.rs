use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::attributes::javascript_string_length;
use crate::{ValidationError, ValidationResult};

/// Materialized workflow-run state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl WorkflowRunStatus {
    /// Whether no further workflow execution may transition this run.
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Maximum number of run ids accepted by one bulk-cancel operation.
pub const BULK_CANCEL_MAX_RUN_IDS: usize = 500;
/// Maximum cancellation reason length, measured like JavaScript strings.
pub const CANCEL_REASON_MAX_LENGTH: usize = 512;

/// Request for cancelling multiple runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkCancelWorkflowRunsRequest {
    pub run_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_reason: Option<String>,
}

impl BulkCancelWorkflowRunsRequest {
    /// Validates the same request constraints as the TypeScript schema.
    pub fn validate(&self) -> ValidationResult<()> {
        if self.run_ids.is_empty() {
            return Err(ValidationError::new(
                "bulk_cancel_empty",
                "runIds must contain at least one run ID",
            ));
        }
        if self.run_ids.len() > BULK_CANCEL_MAX_RUN_IDS {
            return Err(ValidationError::new(
                "bulk_cancel_too_many",
                format!(
                    "runIds contains {} IDs; maximum is {BULK_CANCEL_MAX_RUN_IDS}",
                    self.run_ids.len()
                ),
            ));
        }

        let unique: BTreeSet<&str> = self.run_ids.iter().map(String::as_str).collect();
        if unique.len() != self.run_ids.len() {
            return Err(ValidationError::new(
                "bulk_cancel_duplicate",
                "runIds must not contain duplicates",
            ));
        }

        if let Some(reason) = &self.cancel_reason {
            let length = javascript_string_length(reason);
            if length > CANCEL_REASON_MAX_LENGTH {
                return Err(ValidationError::new(
                    "cancel_reason_too_long",
                    format!(
                        "cancelReason length {length} exceeds limit {CANCEL_REASON_MAX_LENGTH}"
                    ),
                ));
            }
        }
        Ok(())
    }
}

/// Per-run outcome of a bulk cancellation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BulkCancelWorkflowRunResult {
    Cancelled {
        #[serde(rename = "runId")]
        run_id: String,
    },
    AlreadyCancelled {
        #[serde(rename = "runId")]
        run_id: String,
    },
    NotCancellable {
        #[serde(rename = "runId")]
        run_id: String,
        status: String,
    },
    NotFound {
        #[serde(rename = "runId")]
        run_id: String,
    },
    Failed {
        #[serde(rename = "runId")]
        run_id: String,
        code: String,
        retryable: bool,
    },
}

impl BulkCancelWorkflowRunResult {
    /// Run id associated with this result.
    pub fn run_id(&self) -> &str {
        match self {
            Self::Cancelled { run_id }
            | Self::AlreadyCancelled { run_id }
            | Self::NotCancellable { run_id, .. }
            | Self::NotFound { run_id }
            | Self::Failed { run_id, .. } => run_id,
        }
    }
}

/// Aggregate outcome counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkCancelWorkflowRunsSummary {
    pub requested: usize,
    pub cancelled: usize,
    pub already_cancelled: usize,
    pub not_cancellable: usize,
    pub not_found: usize,
    pub failed: usize,
}

/// Aggregate bulk-cancellation response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkCancelWorkflowRunsResult {
    pub summary: BulkCancelWorkflowRunsSummary,
    pub results: Vec<BulkCancelWorkflowRunResult>,
}

impl BulkCancelWorkflowRunsResult {
    /// Validates that summary counters are an exact projection of `results`.
    ///
    /// The TypeScript Zod shape checks field types but does not check these
    /// cross-field invariants. Rust refuses internally inconsistent responses
    /// before callers make retry or accounting decisions from them.
    pub fn validate_consistency(&self) -> ValidationResult<()> {
        let mut actual = BulkCancelWorkflowRunsSummary {
            requested: self.results.len(),
            ..BulkCancelWorkflowRunsSummary::default()
        };
        for result in &self.results {
            match result {
                BulkCancelWorkflowRunResult::Cancelled { .. } => actual.cancelled += 1,
                BulkCancelWorkflowRunResult::AlreadyCancelled { .. } => {
                    actual.already_cancelled += 1;
                }
                BulkCancelWorkflowRunResult::NotCancellable { .. } => {
                    actual.not_cancellable += 1;
                }
                BulkCancelWorkflowRunResult::NotFound { .. } => actual.not_found += 1,
                BulkCancelWorkflowRunResult::Failed { .. } => actual.failed += 1,
            }
        }

        if self.summary != actual {
            return Err(ValidationError::new(
                "bulk_cancel_summary_mismatch",
                format!(
                    "Bulk cancellation summary {:?} does not match result-derived summary {:?}",
                    self.summary, actual
                ),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_statuses_match_the_typescript_contract() {
        assert!(!WorkflowRunStatus::Pending.is_terminal());
        assert!(!WorkflowRunStatus::Running.is_terminal());
        assert!(WorkflowRunStatus::Completed.is_terminal());
        assert!(WorkflowRunStatus::Failed.is_terminal());
        assert!(WorkflowRunStatus::Cancelled.is_terminal());
    }

    #[test]
    fn validates_bulk_cancel_request_boundaries() {
        let request = BulkCancelWorkflowRunsRequest {
            run_ids: vec!["wrun_1".to_owned()],
            cancel_reason: Some("cleanup".to_owned()),
        };
        assert!(request.validate().is_ok());

        let empty = BulkCancelWorkflowRunsRequest {
            run_ids: Vec::new(),
            cancel_reason: None,
        };
        assert!(empty.validate().is_err());

        let duplicate = BulkCancelWorkflowRunsRequest {
            run_ids: vec!["wrun_1".to_owned(), "wrun_1".to_owned()],
            cancel_reason: None,
        };
        assert!(duplicate.validate().is_err());

        let too_many = BulkCancelWorkflowRunsRequest {
            run_ids: (0..=BULK_CANCEL_MAX_RUN_IDS)
                .map(|index| format!("wrun_{index}"))
                .collect(),
            cancel_reason: None,
        };
        assert!(too_many.validate().is_err());
    }

    #[test]
    fn cancel_reason_uses_javascript_utf16_length() {
        let at_limit = BulkCancelWorkflowRunsRequest {
            run_ids: vec!["wrun_1".to_owned()],
            cancel_reason: Some("💥".repeat(256)),
        };
        assert!(at_limit.validate().is_ok());

        let over_limit = BulkCancelWorkflowRunsRequest {
            run_ids: vec!["wrun_1".to_owned()],
            cancel_reason: Some("💥".repeat(257)),
        };
        assert!(over_limit.validate().is_err());
    }

    #[test]
    fn rejects_inconsistent_aggregate_summaries() {
        let result = BulkCancelWorkflowRunsResult {
            summary: BulkCancelWorkflowRunsSummary {
                requested: 2,
                cancelled: 2,
                ..BulkCancelWorkflowRunsSummary::default()
            },
            results: vec![
                BulkCancelWorkflowRunResult::Cancelled {
                    run_id: "a".to_owned(),
                },
                BulkCancelWorkflowRunResult::NotFound {
                    run_id: "b".to_owned(),
                },
            ],
        };
        let error = result.validate_consistency().unwrap_err();
        assert_eq!(error.code(), "bulk_cancel_summary_mismatch");
    }
}

/// Minimum accepted CLI bulk-cancel batch size.
pub const BULK_CANCEL_MIN_LIMIT: f64 = 1.0;
/// Maximum accepted CLI bulk-cancel batch size.
pub const BULK_CANCEL_MAX_LIMIT: f64 = 500.0;
/// Cancellation reason emitted by the CLI.
pub const CLI_CANCEL_REASON: &str = "Cancelled via Workflow CLI";
/// Guidance shown when one bounded batch did not exhaust the matches.
pub const HAS_MORE_GUIDANCE: &str =
    "More runs match these filters. Re-run this command to cancel the next batch,\n\
     or use --limit up to 500.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub const CANCELLABLE_STATUSES: [RunStatus; 2] = [RunStatus::Pending, RunStatus::Running];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeRun {
    pub run_id: String,
    pub workflow_name: String,
    pub status: RunStatus,
    pub started_at: Option<String>,
}

impl FakeRun {
    pub fn new(run_id: &str, status: RunStatus) -> Self {
        Self {
            run_id: run_id.to_owned(),
            workflow_name: "wf".to_owned(),
            status,
            started_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulkCancelRunResult {
    Cancelled {
        run_id: String,
    },
    AlreadyCancelled {
        run_id: String,
    },
    NotCancellable {
        run_id: String,
        status: String,
    },
    NotFound {
        run_id: String,
    },
    Failed {
        run_id: String,
        code: String,
        retryable: bool,
    },
}

impl BulkCancelRunResult {
    pub fn cancelled(run_id: &str) -> Self {
        Self::Cancelled {
            run_id: run_id.to_owned(),
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BulkCancelSummary {
    pub requested: usize,
    pub cancelled: usize,
    pub already_cancelled: usize,
    pub not_cancellable: usize,
    pub not_found: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BulkCancelResult {
    pub summary: BulkCancelSummary,
    pub results: Vec<BulkCancelRunResult>,
}

impl BulkCancelResult {
    pub fn cancelled(run_ids: &[&str]) -> Self {
        Self {
            summary: BulkCancelSummary {
                requested: run_ids.len(),
                cancelled: run_ids.len(),
                ..BulkCancelSummary::default()
            },
            results: run_ids
                .iter()
                .map(|run_id| BulkCancelRunResult::cancelled(run_id))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkCancelRequest {
    pub run_ids: Vec<String>,
    pub cancel_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedCancelEvent {
    pub run_id: String,
    pub cancel_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelManyBehavior {
    Absent,
    CancelAll,
    Fixed(BulkCancelResult),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeWorld {
    pub runs: Vec<FakeRun>,
    pub has_more: bool,
    pub cancel_many: CancelManyBehavior,
}

impl FakeWorld {
    pub fn new(runs: Vec<FakeRun>) -> Self {
        Self {
            runs,
            has_more: false,
            cancel_many: CancelManyBehavior::Absent,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkCancelParams {
    pub world: FakeWorld,
    pub status: Option<RunStatus>,
    pub workflow_name: Option<String>,
    pub limit: usize,
    pub confirm: bool,
    pub prompt_confirmation: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BulkCancelObservation {
    pub exit_code: i32,
    pub result: Option<BulkCancelResult>,
    pub logs: Vec<String>,
    pub warns: Vec<String>,
    pub errors: Vec<String>,
    pub requested_statuses: Vec<RunStatus>,
    pub cancel_many_requests: Vec<BulkCancelRequest>,
    pub created_events: Vec<CreatedCancelEvent>,
}

/// Validates the `--limit` value with JavaScript number semantics.
pub fn validate_bulk_cancel_limit(limit: f64) -> Option<String> {
    let _ = limit;
    panic!("TDD RED: packages/cli/src/lib/bulk-cancel.test.ts implementation pending")
}

/// Produces the exact compact multi-line summary shown by the CLI.
pub fn format_bulk_cancel_summary(result: &BulkCancelResult) -> String {
    let _ = result;
    panic!("TDD RED: packages/cli/src/lib/bulk-cancel.test.ts implementation pending")
}

/// Selects the per-run outcomes that require individual warning lines.
pub fn bulk_cancel_failure_lines(result: &BulkCancelResult) -> Vec<String> {
    let _ = result;
    panic!("TDD RED: packages/cli/src/lib/bulk-cancel.test.ts implementation pending")
}

/// Executes one bounded bulk-cancel CLI operation against the fake World boundary.
pub fn perform_bulk_cancel(params: BulkCancelParams) -> BulkCancelObservation {
    let _ = params;
    panic!("TDD RED: packages/cli/src/lib/bulk-cancel.test.ts implementation pending")
}

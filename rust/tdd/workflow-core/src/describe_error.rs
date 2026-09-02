fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/describe-error.test.ts implementation pending")
}

pub const SERIALIZATION_ERROR_HINT: &str = "A value passed across a workflow/step boundary could not be serialized. See the error message for the offending path and the Learn More link for details.";
pub const CONTEXT_ERROR_HINT: &str = "A workflow-only or step-only API was called from the wrong context. The error message includes the exact API and how to move the call.";
pub const RUNTIME_ERROR_HINT: &str = "This is an internal workflow SDK error, not a bug in your code. If it keeps happening, please report it with the stack trace and the runId.";
pub const CORRUPTED_EVENT_LOG_HINT: &str = "The workflow event log contains orphaned or mismatched events and cannot be replayed. This is an internal workflow SDK error; please report it with the runId.";
pub const REPLAY_TIMEOUT_HINT: &str = "The workflow replay between step boundaries took too long. This bounds workflow-VM and event-log replay time only — step bodies (`\"use step\"` functions) are excluded. This usually means the event log is unusually large or the workflow function is doing heavy synchronous work in workflow code outside of step bodies. Override the default budget via the WORKFLOW_REPLAY_TIMEOUT_MS env var if needed.";
pub const MAX_DELIVERIES_HINT: &str = "The workflow queue exceeded its max-delivery budget. This usually indicates a persistent runtime failure — check the most recent stack traces for the underlying cause.";
pub const MAX_EVENTS_HINT: &str = "The workflow exceeded the maximum number of events per run. This usually means unbounded work in the workflow function — e.g. a loop that keeps creating steps without terminating. Break long-running workflows into child workflows to stay under the limit.";
pub const WORLD_CONTRACT_HINT: &str = "The workflow backend returned data that violated the SDK contract. This is not retryable; please report it with the stack trace and runId.";
pub const DEPLOYMENT_MISMATCH_HINT: &str = "The run was delivered to a deployment other than the deployment it is pinned to, and was stopped to protect against code-skew errors after the runtime failed to re-route it there. Verify that the run's deployment is still available and that queue callbacks route to it.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAttribution {
    User,
    Sdk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunErrorCode {
    UserError,
    RuntimeError,
    CorruptedEventLog,
    ReplayTimeout,
    MaxDeliveriesExceeded,
    WorldContractError,
    MaxEventsExceeded,
    DeploymentMismatch,
}

impl RunErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserError => "USER_ERROR",
            Self::RuntimeError => "RUNTIME_ERROR",
            Self::CorruptedEventLog => "CORRUPTED_EVENT_LOG",
            Self::ReplayTimeout => "REPLAY_TIMEOUT",
            Self::MaxDeliveriesExceeded => "MAX_DELIVERIES_EXCEEDED",
            Self::WorldContractError => "WORLD_CONTRACT_ERROR",
            Self::MaxEventsExceeded => "MAX_EVENTS_EXCEEDED",
            Self::DeploymentMismatch => "DEPLOYMENT_MISMATCH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescribableError {
    PlainUser,
    NonError,
    Serialization,
    ContextViolation,
    WorkflowRuntime,
    CorruptedEventLog,
    StepNotRegistered,
    DeploymentMismatch,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PersistedErrorSignal {
    pub error_code: Option<String>,
    pub error_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDescription {
    pub attribution: ErrorAttribution,
    pub error_code: RunErrorCode,
    pub hint: Option<&'static str>,
}

/// Describes a live runtime error using typed classification and static hints.
pub fn describe_error(
    error: DescribableError,
    precomputed_error_code: Option<RunErrorCode>,
) -> ErrorDescription {
    let _ = (error, precomputed_error_code);
    pending()
}

/// Reconstructs presentation details from untrusted persisted signal strings.
pub fn describe_run_error(signal: &PersistedErrorSignal) -> ErrorDescription {
    let _ = signal;
    pending()
}

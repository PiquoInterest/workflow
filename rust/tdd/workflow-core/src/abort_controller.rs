pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/src/abort-controller.test.ts implementation pending";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AbortReason {
    #[default]
    Undefined,
    Text(String),
    Error(String),
    DomException {
        name: String,
        message: String,
    },
}

impl AbortReason {
    pub fn text(value: &str) -> Self {
        Self::Text(value.to_owned())
    }

    pub fn error(message: &str) -> Self {
        Self::Error(message.to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationKind {
    Hook,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookRecord {
    pub kind: InvocationKind,
    pub correlation_id: String,
    pub token: String,
    pub is_system: bool,
    pub is_webhook: bool,
    pub abort_requested: bool,
    pub abort_reason: AbortReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowErrorKind {
    ReplayDivergence,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThrowObservation {
    pub threw: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AbortControllerObservation {
    pub signal_defined: bool,
    pub abort_method_defined: bool,
    pub abort_method_is_function: bool,
    pub signal_aborted: bool,
    pub signal_reason: AbortReason,
    pub listener_calls: usize,
    pub queue_size_before: usize,
    pub queue_size_after: usize,
    pub hooks_before_abort: Vec<HookRecord>,
    pub hooks_after_abort: Vec<HookRecord>,
    pub consumed_event_count: usize,
    pub event_index: usize,
    pub unconsumed_event_count: usize,
    pub workflow_error_kind: Option<WorkflowErrorKind>,
    pub workflow_error_event_id: Option<String>,
    pub workflow_error_message: Option<String>,
    pub expected_token: Option<String>,
    pub observed_token: Option<String>,
    pub secondary_signal_aborted: Option<bool>,
    pub secondary_signal_reason: Option<AbortReason>,
    pub input_listener_removals: Vec<usize>,
    pub iterable_passes: usize,
    pub timeout_error: Option<String>,
    pub replay_tokens: Vec<String>,
    pub throw_if_aborted: ThrowObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortControllerScenario {
    Construct,
    AbortWithoutReason,
    AbortWithReason,
    DoubleAbort,
    ReplayTokenMismatch,
    DuplicateReceipts,
    InitialSignal,
    ListenerFires,
    RemovedListener,
    ThrowWhenAborted,
    ThrowWhenNotAborted,
    IndependentControllers,
    StaticAbortDefault,
    StaticAbortReason,
    AnyLaterAbort,
    AnyPreAborted,
    AnySingleShotIterable,
    AnyListenerCleanup,
    TimeoutUnsupported,
    HookCreation,
    HookAbortQueue,
    ReplayStableToken,
}

/// Runs one future Rust durable AbortController scenario through the workflow
/// VM, event consumer, invocation queue, and replay boundary.
pub fn observe_abort_controller(scenario: AbortControllerScenario) -> AbortControllerObservation {
    let _ = scenario;
    panic!("{TDD_RED_MARKER}")
}

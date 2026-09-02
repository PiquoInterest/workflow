fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/abort-consistency.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortConsistencyScenario {
    PreAbortedSerialized,
    LateAbortPersistsStreamPacket,
    ListenerInstalledBeforeStateCheck,
    WorkflowRemainsLiveBeforeHookReceipt,
    StreamOnlyAbortReachesStep,
    MissingHookLeavesReplayLive,
    StepAbortCarriesStableSymbols,
    HookOnlyAbortReachesReplay,
    StreamFailureDegradesGracefully,
    DualFailureDoesNotCorruptState,
    AbortAfterStepCompletion,
    OrphanAbortPacket,
    DoubleAbortIsIdempotent,
    FirstRunListenerOrder,
    ReplayAbortDelivery,
    CrossExecutionAbortDelivery,
    LateListenerObservesReplayAbort,
    AbortAfterLastSuspensionPoint,
    FireAndForgetSleep,
    PendingStepAtSuspension,
    PendingHookAtSuspension,
    PendingWaitAtSuspension,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AbortConsistencyObservation {
    pub serialized_contains_aborted: bool,
    pub queued_stream_ops_before_abort: usize,
    pub queued_stream_ops_after_abort: usize,
    pub aborted: bool,
    pub reason: Option<String>,
    pub workflow_suspended: bool,
    pub workflow_aborted: bool,
    pub hook_abort_requested: bool,
    pub step_received_abort: bool,
    pub stream_name_bound: bool,
    pub hook_token_bound: bool,
    pub signal_symbols_match_controller: bool,
    pub hook_event_present: bool,
    pub unhandled_errors: usize,
    pub corrupted: bool,
    pub listener_order: Vec<String>,
    pub listener_calls: usize,
    pub workflow_completed: bool,
    pub return_value: Option<String>,
    pub step_count: usize,
    pub hook_count: usize,
    pub wait_count: usize,
    pub pending_step_name: Option<String>,
    pub pending_step_args: Vec<i64>,
    pub pending_hook_token: Option<String>,
    pub wait_has_resume_at: bool,
}

/// Drives one future Rust dual-backed abort consistency scenario.
pub fn run_abort_consistency_scenario(
    scenario: AbortConsistencyScenario,
) -> AbortConsistencyObservation {
    let _ = scenario;
    pending()
}

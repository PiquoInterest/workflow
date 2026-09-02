fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/abort-controller-step.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortStepScenario {
    ReaderRegistered,
    PreAbortedSkipsReader,
    PreAbortedImmediatelyVisible,
    StreamPacketAborts,
    StreamReasonPropagates,
    ListenerFires,
    ThrowIfAborted,
    StepAbortQueuesStreamWrite,
    StepAbortQueuesDurableHookResume,
    StepAbortIsSynchronous,
    AbortOutsideStepContext,
    SharedStreamAbortsEveryConsumer,
    CompositeSignalWithLocalAbort,
    FetchAbortIsFatal,
    ThrowIfAbortedIsFatal,
    CustomAbortReasonIsFatal,
    FatalAbortSkipsRetries,
    RegularErrorRemainsNonFatal,
    DurableHookResumePrecedesStepCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AbortStepObservation {
    pub reader_ops: usize,
    pub background_ops: usize,
    pub pre_completion_ops: usize,
    pub aborted: bool,
    pub reason: Option<String>,
    pub listener_calls: usize,
    pub throws_when_checked: bool,
    pub stream_writes: usize,
    pub durable_hook_resumes: usize,
    pub lazy_hook_resumes: usize,
    pub durable_hook_token: Option<String>,
    pub durable_hook_reason: Option<String>,
    pub consumer_abort_states: Vec<bool>,
    pub composite_aborted: bool,
    pub fatal: bool,
    pub error_message: Option<String>,
    pub crashed: bool,
    pub retries_skipped: bool,
}

/// Drives one future Rust step-side AbortController bridge scenario.
pub fn run_abort_step_scenario(scenario: AbortStepScenario) -> AbortStepObservation {
    let _ = scenario;
    pending()
}

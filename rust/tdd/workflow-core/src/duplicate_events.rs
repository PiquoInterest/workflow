fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/duplicate-events.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateReplayScenario {
    StepStartedAfterCompletion,
    WaitCreatedAfterCompletion,
    SecondHookDisposal,
    HookCreatedAfterDisposal,
    SecondAttributeWrite,
    UnrelatedUnconsumedEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateNotification {
    pub event_index: usize,
    pub event_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DuplicateReplayObservation {
    pub suspended: bool,
    pub result: Option<String>,
    pub observed_values: Vec<String>,
    pub pending_step_names: Vec<String>,
    pub duplicate_notifications: Vec<DuplicateNotification>,
    pub stranded_event: Option<String>,
    pub error: Option<String>,
}

/// Replays one hand-authored log through the future Rust workflow primitives.
pub fn replay_duplicate_scenario(scenario: DuplicateReplayScenario) -> DuplicateReplayObservation {
    let _ = scenario;
    pending()
}

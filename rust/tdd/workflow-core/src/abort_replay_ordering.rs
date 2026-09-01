#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbortReplayObservation {
    pub pending_during_hydration: usize,
    pub aborted_during_hydration: bool,
    pub captured_when_idle: bool,
    pub final_aborted: bool,
    pub final_reason: String,
    pub final_pending_deliveries: usize,
}

/// Replays an abort receipt while reason hydration is delayed past an idle turn.
pub fn replay_abort_with_delayed_hydration(delay_ms: u64) -> AbortReplayObservation {
    let _ = delay_ms;
    panic!("TDD RED: packages/core/src/abort-replay-ordering.test.ts implementation pending")
}

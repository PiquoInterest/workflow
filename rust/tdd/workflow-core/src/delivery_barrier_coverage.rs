pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/src/delivery-barrier-coverage.test.ts implementation pending";

pub const EXTRA_HOPS: [usize; 6] = [0, 1, 3, 8, 20, 50];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryBarrierScenario {
    StepBehindStep,
    WaitBehindStep { extra_hops: usize },
    HookBehindStep { extra_hops: usize },
    HookBehindHook,
    AbortBehindStep,
    ParkedChainIdleReachability,
    AllArmedBatchBlocksIdle,
    ParallelBatchSuspensionSnapshot,
    SingleStepSuspensionControl,
    TurnstileParkedChain { log_order_draws: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeliveryBarrierObservation {
    pub suspended: bool,
    pub pending_steps: Vec<String>,
    pub suspension_snapshot_steps: Vec<String>,
    pub extra_hops: Option<usize>,
    pub reaches_idle: Option<bool>,
    pub initial_barriers: usize,
    pub pre_idle_order: Vec<String>,
    pub delivery_order: Vec<String>,
    pub payload_retired_before_wait: Option<bool>,
    pub remaining_barriers: usize,
    pub log_order_draws: Option<bool>,
    pub replay_error: Option<String>,
}

/// Replays one future Rust delivery-barrier scenario through event
/// consumption, detached delivery continuations, idle detection, and
/// suspension snapshotting.
pub fn observe_delivery_barrier(
    scenario: DeliveryBarrierScenario,
) -> DeliveryBarrierObservation {
    let _ = scenario;
    panic!("{TDD_RED_MARKER}")
}

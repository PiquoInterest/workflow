fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/log-order-draws.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMode {
    ArrivalOrder,
    LogOrder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityBinding {
    pub correlation_id: String,
    pub entity: String,
}

impl EntityBinding {
    pub fn is_finalize_step(&self) -> bool {
        self.entity == "step:finalizeTask"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DrawReplayObservation {
    pub shorter: Vec<EntityBinding>,
    pub longer: Vec<EntityBinding>,
    pub dense_prefixes: Vec<Vec<EntityBinding>>,
    pub repeated_full: Vec<Vec<EntityBinding>>,
}

/// Replays the blocked-branch production shape under one correlation draw mode.
pub fn observe_blocked_branch_draws(mode: DrawMode) -> DrawReplayObservation {
    let _ = mode;
    pending()
}

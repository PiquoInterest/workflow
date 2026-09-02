fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/race-padded-draw-ordering.test.ts implementation pending")
}

/// Deterministic correlation IDs drawn by the seeded TypeScript replay fixture.
pub const CORRELATION_IDS: [&str; 7] = [
    "01K11TFZ62YS0YYFDQ3E8B9YCV",
    "01K11TFZ62YS0YYFDQ3E8B9YCW",
    "01K11TFZ62YS0YYFDQ3E8B9YCX",
    "01K11TFZ62YS0YYFDQ3E8B9YCY",
    "01K11TFZ62YS0YYFDQ3E8B9YCZ",
    "01K11TFZ62YS0YYFDQ3E8B9YD0",
    "01K11TFZ62YS0YYFDQ3E8B9YD1",
];

pub const RECORDED_EVENT_COUNT: usize = 13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayTemperature {
    Cold,
    WarmSharedCache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawBinding {
    pub correlation_id: String,
    pub entity: String,
}

impl DrawBinding {
    pub fn new(correlation_id: &str, entity: &str) -> Self {
        Self {
            correlation_id: correlation_id.to_owned(),
            entity: entity.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayPassObservation {
    pub suspended: bool,
    pub corruption_error: Option<String>,
    pub event_index: usize,
    pub event_count: usize,
    pub pending_steps: Vec<String>,
    pub bindings: Vec<DrawBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RacePaddedReplayObservation {
    /// Cold mode records one pass. Warm mode records the cache-priming cold pass
    /// followed by the replay that consumes memoized payloads.
    pub passes: Vec<ReplayPassObservation>,
    pub hydration_calls: usize,
}

/// Replays the Promise.race watchdog fixture under the future Rust event consumer.
pub fn replay_race_padded_draws(temperature: ReplayTemperature) -> RacePaddedReplayObservation {
    let _ = temperature;
    pending()
}

pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/src/async-deserialization-ordering.test.ts implementation pending";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryValue {
    Text(String),
    Number(i64),
    Message(String),
    Unit,
}

impl DeliveryValue {
    pub fn text(value: &str) -> Self {
        Self::Text(value.to_owned())
    }

    pub fn message(value: &str) -> Self {
        Self::Message(value.to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionKind {
    FatalError,
    EncryptionKeyMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Settlement {
    Fulfilled(DeliveryValue),
    Rejected {
        kind: RejectionKind,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AsyncOrderingObservation {
    pub hydration_delays_ms: Vec<u64>,
    pub settlements: Vec<Settlement>,
    pub resolution_order: Vec<String>,
    pub unhandled_rejections_before_claim: Vec<String>,
    pub claimed_hook_error: Option<String>,
    pub wait_completed: bool,
    pub pending_delivery_barriers: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsyncOrderingScenario {
    ConcurrentSteps,
    SequentialSteps,
    HookPayloads,
    MixedCompletedAndFailedSteps,
    TenConcurrentSteps,
    SleepBetweenSteps,
    InterleavedStepFunctions,
    BufferedHookFailureBeforeClaim,
    UnclaimedHookBarrierRetirement,
}

/// Runs one future Rust replay scenario through the event consumer, async
/// payload hydration, promise queue, and delivery-barrier registry.
pub fn observe_async_deserialization(
    scenario: AsyncOrderingScenario,
) -> AsyncOrderingObservation {
    let _ = scenario;
    panic!("{TDD_RED_MARKER}")
}

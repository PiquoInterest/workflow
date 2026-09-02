fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/flushable-stream.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlushableScenario {
    RejectionBeforeAwait,
    WritableLockReleased,
    WritableClosed,
    SinkWriteError,
    ReadableClosed,
    ConcurrentWrites,
    MultipleWritablePollers,
    MultipleReadablePollers,
    CloseWithPendingWrite,
    SourceError,
    DrainBarrierAdopted,
    LockReleaseWaitsForDrain,
    DrainBarrierFailure,
    FailureWaitsForAcceptedPrefix,
    PlainSink,
    OrderedDeliveryAndClose,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FlushableObservation {
    pub resolved: bool,
    pub error: Option<String>,
    pub chunks: Vec<Vec<u8>>,
    pub sink_closed: bool,
    pub stream_ended: bool,
    pub unhandled_rejections: usize,
    pub writable_pollers: usize,
    pub readable_pollers: usize,
    pub drain_barrier_attached: bool,
    pub completion_claimed_before_barrier: bool,
    pub settled_before_barrier: bool,
    pub settled_after_barrier: bool,
    pub drained: bool,
    pub pending_ops: usize,
}

/// Drives one future Rust flushable-stream scenario to an observable outcome.
pub fn run_flushable_scenario(scenario: FlushableScenario) -> FlushableObservation {
    let _ = scenario;
    pending()
}

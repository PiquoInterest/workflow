fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/replay-payload-cache.test.ts implementation pending")
}

fn security_pending<T>() -> T {
    panic!(
        "TDD RED: packages/core/src/replay-payload-cache-security.test.ts implementation pending"
    )
}

pub const MAX_MEMOIZED_PRIMITIVE_LENGTH: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedPayload {
    pub data: Vec<u8>,
}

impl PreparedPayload {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeduplicationObservation {
    pub same_in_flight_handle: bool,
    pub prepared: PreparedPayload,
    pub preparation_calls: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedPrewarmObservation {
    pub prewarm_returned_successfully: bool,
    pub first_consumer_error: String,
    pub calls_after_first_consumer: usize,
    pub retried: PreparedPayload,
    pub calls_after_retry: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrentPrewarmObservation {
    pub preparations_started_before_resolution: usize,
    pub calls_after_first_prewarm: usize,
    pub newly_awaited_on_second_prewarm: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshRevivalObservation {
    pub preparation_calls: usize,
    pub same_prepared_payload: bool,
    pub same_object_identity: bool,
    pub second_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetScanObservation {
    pub calls_after_sparse_log: usize,
    pub calls_without_reset: usize,
    pub calls_after_reset: usize,
    pub last_prepared_payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyBypassObservation {
    pub calls_after_direct_consumption: usize,
    pub calls_after_prewarm: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveKind {
    Zero,
    False,
    EmptyString,
    Null,
    Undefined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimitiveMemoCase {
    pub kind: PrimitiveKind,
    pub first_equals_second: bool,
    pub hydration_calls: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableOversizedObservation {
    pub object_hydration_calls: usize,
    pub object_identity_reused: bool,
    pub oversized_hydration_calls: usize,
    pub oversized_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedHydrationObservation {
    pub first_error: String,
    pub second_value: String,
    pub hydration_calls: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadConflictObservation {
    pub accepted_conflicting_payload: bool,
    pub preparation_calls: usize,
    pub returned_payload: Option<Vec<u8>>,
    pub error: Option<String>,
    pub reflected_event_id: bool,
}

/// Observes exact in-flight promise sharing for one binary event payload.
pub fn observe_synchronous_preparer_deduplication() -> DeduplicationObservation {
    pending()
}

/// Observes retention of a speculative rejection until its ordered consumer sees it.
pub fn observe_failed_prewarm_retry() -> FailedPrewarmObservation {
    pending()
}

/// Observes concurrent workflow/result/error/hook prewarming and cached rescans.
pub fn observe_concurrent_prewarm() -> ConcurrentPrewarmObservation {
    pending()
}

/// Observes cached decrypt/decompress preparation with fresh object revival per VM.
pub fn observe_cached_preparation_with_fresh_revival() -> FreshRevivalObservation {
    pending()
}

/// Observes a stale-snapshot rescan after an event was inserted below the old prefix.
pub fn observe_reset_scan_after_inserted_event() -> ResetScanObservation {
    pending()
}

/// Observes legacy-value cache bypass and missing event-data handling during prewarm.
pub fn observe_legacy_and_missing_payload_bypass() -> LegacyBypassObservation {
    pending()
}

/// Observes memoization for every share-safe primitive, including undefined.
pub fn observe_primitive_step_result_memoization() -> Vec<PrimitiveMemoCase> {
    pending()
}

/// Observes fresh hydration for mutable values and strings above the memoization cap.
pub fn observe_mutable_and_oversized_rehydration() -> MutableOversizedObservation {
    pending()
}

/// Observes that a failed step hydration is never installed in the primitive cache.
pub fn observe_failed_step_hydration_retry() -> FailedHydrationObservation {
    pending()
}

/// Proves one cache key cannot be rebound to different authenticated bytes.
pub fn observe_conflicting_event_payload_alias() -> PayloadConflictObservation {
    security_pending()
}

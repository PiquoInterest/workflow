use workflow_core_tdd::flushable_stream::{
    FlushableObservation, FlushableScenario, run_flushable_scenario,
};

fn bytes(value: &str) -> Vec<u8> {
    value.as_bytes().to_vec()
}

fn contains(observation: &FlushableObservation, value: &str) -> bool {
    observation.chunks.contains(&bytes(value))
}

#[test]
fn rejected_state_is_handled_before_the_runtime_awaits_it() {
    let observation = run_flushable_scenario(FlushableScenario::RejectionBeforeAwait);
    assert_eq!(observation.unhandled_rejections, 0);
    assert!(
        observation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Stream write failed")
    );
}

#[test]
fn writable_lock_release_resolves_after_the_pipe_catches_up() {
    let observation = run_flushable_scenario(FlushableScenario::WritableLockReleased);
    assert!(observation.resolved);
    assert!(contains(&observation, "chunk1"));
    assert!(contains(&observation, "chunk2"));
    assert!(!observation.sink_closed);
}

#[test]
fn natural_writable_close_resolves_and_closes_the_sink() {
    let observation = run_flushable_scenario(FlushableScenario::WritableClosed);
    assert!(observation.resolved);
    assert!(contains(&observation, "data"));
    assert!(observation.sink_closed);
}

#[test]
fn sink_write_errors_reject_after_the_accepted_prefix() {
    let observation = run_flushable_scenario(FlushableScenario::SinkWriteError);
    assert!(contains(&observation, "chunk1"));
    assert!(
        observation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Write failed")
    );
}

#[test]
fn readable_lock_polling_resolves_after_source_close() {
    let observation = run_flushable_scenario(FlushableScenario::ReadableClosed);
    assert!(observation.resolved);
    assert!(contains(&observation, "data1"));
    assert!(contains(&observation, "data2"));
}

#[test]
fn concurrent_writes_are_all_delivered() {
    let observation = run_flushable_scenario(FlushableScenario::ConcurrentWrites);
    assert!(observation.resolved);
    assert_eq!(observation.chunks.len(), 3);
    assert!(contains(&observation, "chunk1"));
    assert!(contains(&observation, "chunk2"));
    assert!(contains(&observation, "chunk3"));
}

#[test]
fn writable_polling_is_singleton_per_state() {
    let observation = run_flushable_scenario(FlushableScenario::MultipleWritablePollers);
    assert_eq!(observation.writable_pollers, 1);
}

#[test]
fn readable_polling_is_singleton_per_state() {
    let observation = run_flushable_scenario(FlushableScenario::MultipleReadablePollers);
    assert_eq!(observation.readable_pollers, 1);
}

#[test]
fn close_waits_for_pending_writes() {
    let observation = run_flushable_scenario(FlushableScenario::CloseWithPendingWrite);
    assert!(observation.resolved);
    assert!(contains(&observation, "fast"));
    assert!(contains(&observation, "slow"));
}

#[test]
fn source_errors_propagate_after_the_accepted_prefix() {
    let observation = run_flushable_scenario(FlushableScenario::SourceError);
    assert!(contains(&observation, "valid chunk"));
    assert!(observation.stream_ended);
    assert!(
        observation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Client disconnected")
    );
}

#[test]
fn group_commit_sink_drain_barrier_is_adopted() {
    let observation = run_flushable_scenario(FlushableScenario::DrainBarrierAdopted);
    assert!(observation.drain_barrier_attached);
    assert!(observation.resolved);
}

#[test]
fn lock_release_completion_waits_for_the_drain_barrier() {
    let observation = run_flushable_scenario(FlushableScenario::LockReleaseWaitsForDrain);
    assert!(observation.completion_claimed_before_barrier);
    assert!(!observation.settled_before_barrier);
    assert!(observation.settled_after_barrier);
    assert!(observation.resolved);
}

#[test]
fn drain_barrier_failure_rejects_completion() {
    let observation = run_flushable_scenario(FlushableScenario::DrainBarrierFailure);
    assert!(
        observation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("group flush failed")
    );
}

#[test]
fn failed_pipe_drains_the_accepted_prefix_before_settling() {
    let observation = run_flushable_scenario(FlushableScenario::FailureWaitsForAcceptedPrefix);
    assert!(!observation.settled_before_barrier);
    assert!(observation.drained);
    assert!(
        observation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("producer failed")
    );
}

#[test]
fn plain_sinks_do_not_install_a_drain_barrier() {
    let observation = run_flushable_scenario(FlushableScenario::PlainSink);
    assert!(!observation.drain_barrier_attached);
    assert_eq!(observation.chunks, vec![vec![1]]);
    assert!(observation.resolved);
}

#[test]
fn every_chunk_is_delivered_in_order_and_the_sink_closes() {
    let observation = run_flushable_scenario(FlushableScenario::OrderedDeliveryAndClose);
    let expected: Vec<Vec<u8>> = (0..25).map(|value| vec![value]).collect();
    assert_eq!(observation.chunks, expected);
    assert!(observation.sink_closed);
    assert!(observation.resolved);
    assert_eq!(observation.pending_ops, 0);
}

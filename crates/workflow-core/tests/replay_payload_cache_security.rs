#![forbid(unsafe_code)]

use std::sync::{
    Arc, Barrier,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;

use workflow_core::replay_payload_cache::{
    PAYLOAD_CONFLICT_MESSAGE, PreparedReplayPayload, ReplayCacheErrorKind, ReplayPayload,
    ReplayPayloadCache, ReplayPayloadField, WorkflowRunPayload,
};

fn binary(value: u8) -> ReplayPayload {
    ReplayPayload::binary(vec![value])
}

#[test]
fn conflicting_workflow_input_marks_the_run_key_terminal_and_redacts_identity() {
    let calls = Arc::new(AtomicUsize::new(0));
    let preparer_calls = Arc::clone(&calls);
    let cache = ReplayPayloadCache::new(move |input: &ReplayPayload| {
        preparer_calls.fetch_add(1, Ordering::SeqCst);
        Ok(PreparedReplayPayload::from_input(input))
    });
    let run_id = "wrun_conflicting_payload";
    let first = WorkflowRunPayload::new(run_id, binary(1));
    let conflicting = WorkflowRunPayload::new(run_id, binary(2));

    cache.prepare_workflow_input(&first).unwrap();
    let conflict = cache.prepare_workflow_input(&conflicting).unwrap_err();

    assert_eq!(conflict.kind, ReplayCacheErrorKind::PayloadConflict);
    assert_eq!(conflict.message, PAYLOAD_CONFLICT_MESSAGE);
    assert!(!conflict.message.contains(run_id));
    assert!(!conflict.message.contains('1'));
    assert!(!conflict.message.contains('2'));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let repeated = cache.prepare_workflow_input(&first).unwrap_err();
    assert_eq!(repeated, conflict);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn in_flight_conflict_remains_terminal_after_original_preparation_finishes() {
    let calls = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let cache = Arc::new(ReplayPayloadCache::new({
        let calls = Arc::clone(&calls);
        let started = Arc::clone(&started);
        let release = Arc::clone(&release);
        move |input: &ReplayPayload| {
            calls.fetch_add(1, Ordering::SeqCst);
            started.wait();
            release.wait();
            Ok(PreparedReplayPayload::from_input(input))
        }
    }));
    let event_id = "evnt_in_flight_conflict";

    let first_consumer = {
        let cache = Arc::clone(&cache);
        thread::spawn(move || {
            let first = binary(1);
            cache.prepare_event_payload(event_id, ReplayPayloadField::Result, &first)
        })
    };

    started.wait();

    let conflicting = binary(2);
    let conflict = cache
        .prepare_event_payload(event_id, ReplayPayloadField::Result, &conflicting)
        .unwrap_err();
    assert_eq!(conflict.kind, ReplayCacheErrorKind::PayloadConflict);
    assert_eq!(conflict.message, PAYLOAD_CONFLICT_MESSAGE);
    assert!(!conflict.message.contains(event_id));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    release.wait();

    let first_error = first_consumer.join().unwrap().unwrap_err();
    assert_eq!(first_error, conflict);

    let original_bytes = binary(1);
    let repeated = cache
        .prepare_event_payload(event_id, ReplayPayloadField::Result, &original_bytes)
        .unwrap_err();
    assert_eq!(repeated, conflict);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

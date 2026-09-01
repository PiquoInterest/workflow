use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use workflow_core::replay_payload_cache::{
    MAX_MEMOIZED_PRIMITIVE_UTF16_LENGTH, PAYLOAD_CONFLICT_MESSAGE, PreparedReplayPayload,
    ReplayCacheError, ReplayCacheErrorKind, ReplayEvent, ReplayEventType, ReplayPayload,
    ReplayPayloadCache, ReplayPayloadField, ReplayValue, WorkflowRunPayload,
};

fn identity_preparer(
    calls: Arc<AtomicUsize>,
) -> impl Fn(&ReplayPayload) -> Result<PreparedReplayPayload, ReplayCacheError> + Send + Sync {
    move |input| {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(PreparedReplayPayload::from_input(input))
    }
}

fn binary(value: u8) -> ReplayPayload {
    ReplayPayload::binary(vec![value])
}

fn object_count(value: i64) -> ReplayValue {
    ReplayValue::Object(vec![("count".to_owned(), ReplayValue::Integer(value))])
}

fn read_object_count(value: &ReplayValue) -> Option<i64> {
    let ReplayValue::Object(fields) = value else {
        return None;
    };
    fields.iter().find_map(|(key, value)| {
        if key == "count" {
            if let ReplayValue::Integer(value) = value {
                return Some(*value);
            }
        }
        None
    })
}

#[test]
fn shares_one_preparation_for_identical_event_payload_bytes() {
    let calls = Arc::new(AtomicUsize::new(0));
    let cache = ReplayPayloadCache::new(identity_preparer(Arc::clone(&calls)));
    let payload = binary(1);

    let first = cache
        .prepare_event_payload("evnt_one", ReplayPayloadField::Result, &payload)
        .unwrap();
    let second = cache
        .prepare_event_payload("evnt_one", ReplayPayloadField::Result, &payload)
        .unwrap();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.value, ReplayValue::Bytes(vec![1]));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn failed_prewarm_is_observed_once_then_the_next_consumer_retries() {
    let calls = Arc::new(AtomicUsize::new(0));
    let preparer_calls = Arc::clone(&calls);
    let cache = ReplayPayloadCache::new(move |input: &ReplayPayload| {
        let call = preparer_calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            Err(ReplayCacheError::preparation("decrypt failed"))
        } else {
            Ok(PreparedReplayPayload::from_input(input))
        }
    });
    let run = WorkflowRunPayload::new("wrun_failed_prewarm", binary(1));

    let report = cache.prewarm(&run, &[]);
    assert_eq!(report.discovered, 1);
    assert_eq!(report.completed, 0);
    assert_eq!(report.failed, 1);

    let first_error = cache.prepare_workflow_input(&run).unwrap_err();
    assert_eq!(first_error.kind, ReplayCacheErrorKind::Preparation);
    assert_eq!(first_error.message, "decrypt failed");
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let retry = cache.prepare_workflow_input(&run).unwrap();
    assert_eq!(retry.value, ReplayValue::Bytes(vec![1]));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn prewarm_starts_workflow_result_error_and_hook_payloads_concurrently() {
    struct Gate {
        started: AtomicUsize,
        released: AtomicBool,
        lock: Mutex<()>,
        changed: Condvar,
    }

    let gate = Arc::new(Gate {
        started: AtomicUsize::new(0),
        released: AtomicBool::new(false),
        lock: Mutex::new(()),
        changed: Condvar::new(),
    });
    let preparer_gate = Arc::clone(&gate);
    let cache = ReplayPayloadCache::with_max_prewarm_concurrency(
        move |input: &ReplayPayload| {
            preparer_gate.started.fetch_add(1, Ordering::SeqCst);
            preparer_gate.changed.notify_all();
            let mut guard = preparer_gate
                .lock
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            while !preparer_gate.released.load(Ordering::SeqCst) {
                guard = preparer_gate
                    .changed
                    .wait(guard)
                    .unwrap_or_else(|error| error.into_inner());
            }
            Ok(PreparedReplayPayload::from_input(input))
        },
        4,
    );

    let run = WorkflowRunPayload::new("wrun_concurrent", binary(0));
    let events = vec![
        ReplayEvent::new(
            "evnt_result",
            ReplayEventType::StepCompleted,
            Some(binary(1)),
        ),
        ReplayEvent::new("evnt_error", ReplayEventType::StepFailed, Some(binary(2))),
        ReplayEvent::new("evnt_hook", ReplayEventType::HookReceived, Some(binary(3))),
    ];

    let cache_for_thread = cache.clone();
    let run_for_thread = run.clone();
    let events_for_thread = events.clone();
    let handle =
        thread::spawn(move || cache_for_thread.prewarm(&run_for_thread, &events_for_thread));

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut guard = gate.lock.lock().unwrap_or_else(|error| error.into_inner());
    while gate.started.load(Ordering::SeqCst) < 4 && Instant::now() < deadline {
        let waited = gate
            .changed
            .wait_timeout(guard, Duration::from_millis(10))
            .unwrap_or_else(|error| error.into_inner());
        guard = waited.0;
    }
    let started_before_release = gate.started.load(Ordering::SeqCst);
    gate.released.store(true, Ordering::SeqCst);
    gate.changed.notify_all();
    drop(guard);

    let report = handle.join().expect("prewarm worker should not panic");
    assert_eq!(started_before_release, 4);
    assert_eq!(report.discovered, 4);
    assert_eq!(report.completed, 4);
    assert_eq!(report.failed, 0);

    let second = cache.prewarm(&run, &events);
    assert_eq!(second.discovered, 0);
    assert_eq!(gate.started.load(Ordering::SeqCst), 4);
}

#[test]
fn caches_binary_preparation_but_rehydrates_mutable_objects_for_each_vm() {
    let preparation_calls = Arc::new(AtomicUsize::new(0));
    let cache = ReplayPayloadCache::new(identity_preparer(Arc::clone(&preparation_calls)));
    let payload = binary(7);

    let first_prepared = cache
        .prepare_event_payload("evnt_object", ReplayPayloadField::Result, &payload)
        .unwrap();
    let second_prepared = cache
        .prepare_event_payload("evnt_object", ReplayPayloadField::Result, &payload)
        .unwrap();
    assert!(Arc::ptr_eq(&first_prepared, &second_prepared));
    assert_eq!(preparation_calls.load(Ordering::SeqCst), 1);

    let hydration_calls = AtomicUsize::new(0);
    let mut first = cache
        .get_step_result("evnt_object", || {
            hydration_calls.fetch_add(1, Ordering::SeqCst);
            Ok(object_count(0))
        })
        .unwrap();
    if let ReplayValue::Object(fields) = &mut first {
        fields[0].1 = ReplayValue::Integer(99);
    }
    let second = cache
        .get_step_result("evnt_object", || {
            hydration_calls.fetch_add(1, Ordering::SeqCst);
            Ok(object_count(0))
        })
        .unwrap();

    assert_eq!(hydration_calls.load(Ordering::SeqCst), 2);
    assert_eq!(read_object_count(&first), Some(99));
    assert_eq!(read_object_count(&second), Some(0));
}

#[test]
fn reset_scan_discovers_an_event_inserted_below_the_previous_prefix() {
    let calls = Arc::new(AtomicUsize::new(0));
    let last_payload = Arc::new(Mutex::new(Vec::new()));
    let preparer_calls = Arc::clone(&calls);
    let preparer_last = Arc::clone(&last_payload);
    let cache = ReplayPayloadCache::new(move |input: &ReplayPayload| {
        preparer_calls.fetch_add(1, Ordering::SeqCst);
        if let ReplayPayload::Binary(bytes) = input {
            *preparer_last
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = bytes.to_vec();
        }
        Ok(PreparedReplayPayload::from_input(input))
    });
    let run = WorkflowRunPayload::new("wrun_reset", ReplayPayload::legacy(ReplayValue::Undefined));
    let first = ReplayEvent::new(
        "evnt_first",
        ReplayEventType::StepCompleted,
        Some(binary(0)),
    );
    let missing = ReplayEvent::new("evnt_missing", ReplayEventType::StepFailed, Some(binary(1)));
    let second = ReplayEvent::new(
        "evnt_second",
        ReplayEventType::HookReceived,
        Some(binary(2)),
    );

    cache.prewarm(&run, &[first.clone(), second.clone()]);
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    cache.prewarm(&run, &[first.clone(), missing.clone(), second.clone()]);
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    cache.reset_scan();
    cache.prewarm(&run, &[first, missing, second]);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(
        *last_payload
            .lock()
            .unwrap_or_else(|error| error.into_inner()),
        vec![1]
    );
}

#[test]
fn legacy_values_bypass_the_cache_and_missing_event_data_is_ignored() {
    let calls = Arc::new(AtomicUsize::new(0));
    let cache = ReplayPayloadCache::new(identity_preparer(Arc::clone(&calls)));
    let legacy = ReplayPayload::legacy(ReplayValue::Object(vec![(
        "value".to_owned(),
        ReplayValue::Integer(1),
    )]));

    cache
        .prepare_event_payload("evnt_legacy", ReplayPayloadField::Result, &legacy)
        .unwrap();
    cache
        .prepare_event_payload("evnt_legacy", ReplayPayloadField::Result, &legacy)
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let run = WorkflowRunPayload::new("wrun_legacy", legacy.clone());
    let events = vec![
        ReplayEvent::new(
            "evnt_result",
            ReplayEventType::StepCompleted,
            Some(legacy.clone()),
        ),
        ReplayEvent::new("evnt_error", ReplayEventType::StepFailed, Some(legacy)),
        ReplayEvent::new("evnt_hook", ReplayEventType::HookReceived, None),
    ];
    let report = cache.prewarm(&run, &events);
    assert_eq!(report.discovered, 0);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn memoizes_every_share_safe_primitive_including_undefined() {
    let values = vec![
        ReplayValue::Integer(0),
        ReplayValue::Boolean(false),
        ReplayValue::String(String::new()),
        ReplayValue::Null,
        ReplayValue::Undefined,
    ];

    for (index, expected) in values.into_iter().enumerate() {
        let cache = ReplayPayloadCache::identity();
        let calls = AtomicUsize::new(0);
        let event_id = format!("evnt_primitive_{index}");
        let first = cache
            .get_step_result(&event_id, || {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(expected.clone())
            })
            .unwrap();
        let second = cache
            .get_step_result(&event_id, || {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(ReplayValue::Integer(999))
            })
            .unwrap();
        assert_eq!(first, expected);
        assert_eq!(second, expected);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn mutable_and_oversized_step_results_are_rehydrated() {
    let cache = ReplayPayloadCache::identity();
    let object_calls = AtomicUsize::new(0);
    let first_object = cache
        .get_step_result("evnt_mutable", || {
            object_calls.fetch_add(1, Ordering::SeqCst);
            Ok(object_count(0))
        })
        .unwrap();
    let second_object = cache
        .get_step_result("evnt_mutable", || {
            object_calls.fetch_add(1, Ordering::SeqCst);
            Ok(object_count(0))
        })
        .unwrap();
    assert_eq!(first_object, second_object);
    assert_eq!(object_calls.load(Ordering::SeqCst), 2);

    let oversized = "x".repeat(MAX_MEMOIZED_PRIMITIVE_UTF16_LENGTH + 1);
    let string_calls = AtomicUsize::new(0);
    for _ in 0..2 {
        let value = cache
            .get_step_result("evnt_oversized", || {
                string_calls.fetch_add(1, Ordering::SeqCst);
                Ok(ReplayValue::String(oversized.clone()))
            })
            .unwrap();
        assert_eq!(value, ReplayValue::String(oversized.clone()));
    }
    assert_eq!(string_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn failed_step_hydration_is_not_memoized() {
    let cache = ReplayPayloadCache::identity();
    let calls = AtomicUsize::new(0);

    let first = cache.get_step_result("evnt_failed_hydration", || {
        calls.fetch_add(1, Ordering::SeqCst);
        Err(ReplayCacheError::hydration("boom"))
    });
    assert_eq!(first.unwrap_err().message, "boom");

    let second = cache
        .get_step_result("evnt_failed_hydration", || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(ReplayValue::String("ok".to_owned()))
        })
        .unwrap();
    assert_eq!(second, ReplayValue::String("ok".to_owned()));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn conflicting_binary_data_marks_the_cache_key_terminal_and_redacts_identity() {
    let calls = Arc::new(AtomicUsize::new(0));
    let cache = ReplayPayloadCache::new(identity_preparer(Arc::clone(&calls)));
    let first = binary(1);
    let conflicting = binary(2);
    let event_id = "evnt_conflicting_payload";

    cache
        .prepare_event_payload(event_id, ReplayPayloadField::Result, &first)
        .unwrap();
    let conflict = cache
        .prepare_event_payload(event_id, ReplayPayloadField::Result, &conflicting)
        .unwrap_err();
    assert_eq!(conflict.kind, ReplayCacheErrorKind::PayloadConflict);
    assert_eq!(conflict.message, PAYLOAD_CONFLICT_MESSAGE);
    assert!(!conflict.message.contains(event_id));
    assert!(!conflict.message.contains('1'));
    assert!(!conflict.message.contains('2'));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let repeated = cache
        .prepare_event_payload(event_id, ReplayPayloadField::Result, &first)
        .unwrap_err();
    assert_eq!(repeated, conflict);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn structured_cache_keys_keep_fields_and_workflow_inputs_isolated() {
    let calls = Arc::new(AtomicUsize::new(0));
    let cache = ReplayPayloadCache::new(identity_preparer(Arc::clone(&calls)));
    let result = binary(1);
    let error = binary(2);
    let workflow = WorkflowRunPayload::new("event:shared:result", binary(3));

    let prepared_result = cache
        .prepare_event_payload("shared", ReplayPayloadField::Result, &result)
        .unwrap();
    let prepared_error = cache
        .prepare_event_payload("shared", ReplayPayloadField::Error, &error)
        .unwrap();
    let prepared_workflow = cache.prepare_workflow_input(&workflow).unwrap();

    assert_eq!(prepared_result.value, ReplayValue::Bytes(vec![1]));
    assert_eq!(prepared_error.value, ReplayValue::Bytes(vec![2]));
    assert_eq!(prepared_workflow.value, ReplayValue::Bytes(vec![3]));
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

use workflow_core_tdd::replay_payload_cache::{
    MAX_MEMOIZED_PRIMITIVE_LENGTH, PreparedPayload, PrimitiveKind,
    observe_cached_preparation_with_fresh_revival, observe_concurrent_prewarm,
    observe_failed_prewarm_retry, observe_failed_step_hydration_retry,
    observe_legacy_and_missing_payload_bypass, observe_mutable_and_oversized_rehydration,
    observe_primitive_step_result_memoization, observe_reset_scan_after_inserted_event,
    observe_synchronous_preparer_deduplication,
};

#[test]
fn deduplicates_preparation_and_accepts_a_synchronous_preparer() {
    let observation = observe_synchronous_preparer_deduplication();
    assert!(observation.same_in_flight_handle);
    assert_eq!(observation.prepared, PreparedPayload::new(&[1]));
    assert_eq!(observation.preparation_calls, 1);
}

#[test]
fn retains_a_failed_prewarm_until_the_consumer_observes_it_then_retries() {
    let observation = observe_failed_prewarm_retry();
    assert!(observation.prewarm_returned_successfully);
    assert_eq!(observation.first_consumer_error, "decrypt failed");
    assert_eq!(observation.calls_after_first_consumer, 1);
    assert_eq!(observation.retried, PreparedPayload::new(&[1]));
    assert_eq!(observation.calls_after_retry, 2);
}

#[test]
fn prewarms_workflow_step_error_and_hook_payloads_concurrently() {
    let observation = observe_concurrent_prewarm();
    assert_eq!(observation.preparations_started_before_resolution, 4);
    assert_eq!(observation.calls_after_first_prewarm, 4);
    assert_eq!(observation.newly_awaited_on_second_prewarm, 0);
}

#[test]
fn caches_decrypt_and_decompress_output_but_revives_fresh_objects() {
    let observation = observe_cached_preparation_with_fresh_revival();
    assert_eq!(observation.preparation_calls, 1);
    assert!(observation.same_prepared_payload);
    assert!(!observation.same_object_identity);
    assert_eq!(observation.second_count, 0);
}

#[test]
fn reset_scan_rescans_events_inserted_below_the_previous_prefix() {
    let observation = observe_reset_scan_after_inserted_event();
    assert_eq!(observation.calls_after_sparse_log, 2);
    assert_eq!(observation.calls_without_reset, 2);
    assert_eq!(observation.calls_after_reset, 3);
    assert_eq!(observation.last_prepared_payload, vec![1]);
}

#[test]
fn legacy_values_bypass_the_cache_and_missing_event_data_is_ignored() {
    let observation = observe_legacy_and_missing_payload_bypass();
    assert_eq!(observation.calls_after_direct_consumption, 2);
    assert_eq!(observation.calls_after_prewarm, 2);
}

#[test]
fn memoizes_primitive_step_results_including_undefined() {
    let observations = observe_primitive_step_result_memoization();
    assert_eq!(
        observations
            .iter()
            .map(|case| case.kind)
            .collect::<Vec<_>>(),
        vec![
            PrimitiveKind::Zero,
            PrimitiveKind::False,
            PrimitiveKind::EmptyString,
            PrimitiveKind::Null,
            PrimitiveKind::Undefined,
        ]
    );
    for observation in observations {
        assert!(observation.first_equals_second);
        assert_eq!(observation.hydration_calls, 1);
    }
}

#[test]
fn rehydrates_mutable_and_oversized_step_results() {
    let observation = observe_mutable_and_oversized_rehydration();
    assert_eq!(observation.object_hydration_calls, 2);
    assert!(!observation.object_identity_reused);
    assert_eq!(observation.oversized_hydration_calls, 2);
    assert_eq!(
        observation.oversized_length,
        MAX_MEMOIZED_PRIMITIVE_LENGTH + 1
    );
}

#[test]
fn failed_step_hydration_is_not_memoized() {
    let observation = observe_failed_step_hydration_retry();
    assert_eq!(observation.first_error, "boom");
    assert_eq!(observation.second_value, "ok");
    assert_eq!(observation.hydration_calls, 2);
}

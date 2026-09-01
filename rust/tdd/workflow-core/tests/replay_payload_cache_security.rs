use workflow_core_tdd::replay_payload_cache::observe_conflicting_event_payload_alias;

const EVENT_ID: &str = "evnt_conflicting_payload";

#[test]
fn rejects_conflicting_binary_data_for_one_event_cache_key() {
    let observation = observe_conflicting_event_payload_alias();
    assert!(!observation.accepted_conflicting_payload);
    assert_eq!(observation.preparation_calls, 1);
    assert_eq!(observation.returned_payload, None);
    let error = observation.error.expect("conflicting bytes must fail closed");
    assert!(error.contains("different binary data"));
    assert!(!error.contains(EVENT_ID));
    assert!(!observation.reflected_event_id);
}

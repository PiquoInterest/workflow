use workflow_core_tdd::abort_replay_ordering::replay_abort_with_delayed_hydration;

const HYDRATE_DELAY_MS: u64 = 50;

#[test]
fn holds_the_idle_gate_until_abort_reason_hydration_lands() {
    let observation = replay_abort_with_delayed_hydration(HYDRATE_DELAY_MS);
    assert!(observation.captured_when_idle);
    assert!(observation.final_aborted);
    assert_eq!(observation.final_reason, "aborted from step");
    assert_eq!(observation.final_pending_deliveries, 0);
}

#[test]
fn counts_the_in_flight_abort_as_a_pending_delivery() {
    let observation = replay_abort_with_delayed_hydration(HYDRATE_DELAY_MS);
    assert_eq!(observation.pending_during_hydration, 1);
    assert!(!observation.aborted_during_hydration);
    assert_eq!(observation.final_pending_deliveries, 0);
    assert!(observation.final_aborted);
}

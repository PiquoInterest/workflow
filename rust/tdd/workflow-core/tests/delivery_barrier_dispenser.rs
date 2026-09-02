use workflow_core_tdd::delivery_barrier_dispenser::{
    DeliveryBarrierScenario, run_delivery_barrier_scenario,
};

#[test]
fn suspension_waits_for_deliveries_parked_behind_an_unclaimed_payload() {
    let observation =
        run_delivery_barrier_scenario(DeliveryBarrierScenario::ParkedPayloadBeforeSuspension);
    assert!(observation.suspended);
    assert_eq!(observation.pending_steps.len(), 1);
    assert_eq!(observation.pending_steps[0].name, "afterSleep");
}

#[test]
fn several_parked_segments_wake_in_log_order() {
    let observation =
        run_delivery_barrier_scenario(DeliveryBarrierScenario::MultipleParkedSegments);
    assert!(observation.suspended);

    let mut names: Vec<&str> = observation
        .pending_steps
        .iter()
        .map(|step| step.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["afterA", "afterB"]);

    let after_a = observation
        .pending_steps
        .iter()
        .find(|step| step.name == "afterA")
        .expect("afterA must be pending");
    let after_b = observation
        .pending_steps
        .iter()
        .find(|step| step.name == "afterB")
        .expect("afterB must be pending");
    assert!(after_a.correlation_id < after_b.correlation_id);
}

#[test]
fn rejected_promise_queue_does_not_kill_the_dispenser() {
    let observation = run_delivery_barrier_scenario(DeliveryBarrierScenario::RejectedPromiseQueue);
    assert_eq!(observation.registry_size_before_release, 1);
    assert!(!observation.delivery_idle_before_release);
    assert_eq!(observation.registry_size_after_release, 0);
    assert!(observation.delivery_idle_after_release);
}

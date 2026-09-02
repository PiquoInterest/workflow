use workflow_core_tdd::async_deserialization_ordering::{
    AsyncOrderingObservation, AsyncOrderingScenario, DeliveryValue, RejectionKind, Settlement,
    observe_async_deserialization,
};

fn observe(scenario: AsyncOrderingScenario) -> AsyncOrderingObservation {
    observe_async_deserialization(scenario)
}

#[test]
fn concurrent_steps_resolve_in_event_log_order_despite_inverse_hydration_speed() {
    let observation = observe(AsyncOrderingScenario::ConcurrentSteps);
    assert_eq!(observation.hydration_delays_ms, vec![50, 5]);
    assert_eq!(
        observation.settlements,
        vec![
            Settlement::Fulfilled(DeliveryValue::text("result_A")),
            Settlement::Fulfilled(DeliveryValue::text("result_B")),
        ]
    );
    assert_eq!(
        observation.resolution_order,
        vec!["A:result_A".to_owned(), "B:result_B".to_owned()]
    );
}

#[test]
fn sequential_steps_resolve_in_event_log_order_with_decreasing_delays() {
    let observation = observe(AsyncOrderingScenario::SequentialSteps);
    assert_eq!(observation.hydration_delays_ms, vec![60, 30, 5]);
    assert_eq!(
        observation.settlements,
        vec![
            Settlement::Fulfilled(DeliveryValue::Number(10)),
            Settlement::Fulfilled(DeliveryValue::Number(20)),
            Settlement::Fulfilled(DeliveryValue::Number(30)),
        ]
    );
    assert_eq!(
        observation.resolution_order,
        vec!["10".to_owned(), "20".to_owned(), "30".to_owned()]
    );
}

#[test]
fn hook_payloads_resolve_in_event_log_order_despite_inverse_hydration_speed() {
    let observation = observe(AsyncOrderingScenario::HookPayloads);
    assert_eq!(observation.hydration_delays_ms, vec![50, 5]);
    assert_eq!(
        observation.settlements,
        vec![
            Settlement::Fulfilled(DeliveryValue::message("first")),
            Settlement::Fulfilled(DeliveryValue::message("second")),
        ]
    );
    assert_eq!(
        observation.resolution_order,
        vec!["A:first".to_owned(), "B:second".to_owned()]
    );
}

#[test]
fn completed_failed_and_completed_steps_settle_in_event_log_order() {
    let observation = observe(AsyncOrderingScenario::MixedCompletedAndFailedSteps);
    assert_eq!(observation.hydration_delays_ms, vec![50, 5]);
    assert_eq!(
        observation.settlements,
        vec![
            Settlement::Fulfilled(DeliveryValue::text("success_A")),
            Settlement::Rejected {
                kind: RejectionKind::FatalError,
                message: "step B failed".to_owned(),
            },
            Settlement::Fulfilled(DeliveryValue::text("success_C")),
        ]
    );
    assert_eq!(
        observation.resolution_order,
        vec![
            "A:success_A".to_owned(),
            "B:step B failed".to_owned(),
            "C:success_C".to_owned(),
        ]
    );
}

#[test]
fn ten_concurrent_steps_preserve_all_values_and_resolution_order() {
    let observation = observe(AsyncOrderingScenario::TenConcurrentSteps);
    assert_eq!(
        observation.hydration_delays_ms,
        vec![100, 90, 80, 70, 60, 50, 40, 30, 20, 10]
    );
    assert_eq!(
        observation.settlements,
        (0..10)
            .map(|value| Settlement::Fulfilled(DeliveryValue::Number(value)))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        observation.resolution_order,
        (0..10).map(|value| value.to_string()).collect::<Vec<_>>()
    );
}

#[test]
fn sleep_and_steps_resolve_in_their_event_log_order() {
    let observation = observe(AsyncOrderingScenario::SleepBetweenSteps);
    assert_eq!(observation.hydration_delays_ms, vec![50, 5]);
    assert_eq!(
        observation.settlements,
        vec![
            Settlement::Fulfilled(DeliveryValue::text("step_result")),
            Settlement::Fulfilled(DeliveryValue::Unit),
            Settlement::Fulfilled(DeliveryValue::text("after_sleep")),
        ]
    );
    assert_eq!(
        observation.resolution_order,
        vec![
            "step:step_result".to_owned(),
            "sleep".to_owned(),
            "step:after_sleep".to_owned(),
        ]
    );
}

#[test]
fn interleaved_step_functions_still_resolve_by_completion_event_position() {
    let observation = observe(AsyncOrderingScenario::InterleavedStepFunctions);
    assert_eq!(observation.hydration_delays_ms, vec![50, 5]);
    assert_eq!(
        observation.settlements,
        vec![
            Settlement::Fulfilled(DeliveryValue::text("value_A")),
            Settlement::Fulfilled(DeliveryValue::text("value_B")),
        ]
    );
    assert_eq!(
        observation.resolution_order,
        vec!["A:value_A".to_owned(), "B:value_B".to_owned()]
    );
}

#[test]
fn buffered_hook_hydration_failure_does_not_reject_before_a_consumer_claims_it() {
    let observation = observe(AsyncOrderingScenario::BufferedHookFailureBeforeClaim);
    assert_eq!(
        observation.unhandled_rejections_before_claim,
        Vec::<String>::new()
    );
    assert_eq!(
        observation.settlements,
        vec![Settlement::Rejected {
            kind: RejectionKind::EncryptionKeyMissing,
            message: "Encrypted data encountered but no encryption key".to_owned(),
        }]
    );
    assert!(
        observation
            .claimed_hook_error
            .as_deref()
            .unwrap()
            .contains("Encrypted data encountered but no encryption key")
    );
}

#[test]
fn unclaimed_hook_barrier_is_retired_after_a_later_wait_proceeds() {
    let observation = observe(AsyncOrderingScenario::UnclaimedHookBarrierRetirement);
    assert!(observation.wait_completed);
    assert_eq!(observation.pending_delivery_barriers, 0);
}

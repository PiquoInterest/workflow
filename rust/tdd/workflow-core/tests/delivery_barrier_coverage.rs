use workflow_core_tdd::delivery_barrier_coverage::{
    DeliveryBarrierObservation, DeliveryBarrierScenario, EXTRA_HOPS, observe_delivery_barrier,
};

fn observe(scenario: DeliveryBarrierScenario) -> DeliveryBarrierObservation {
    observe_delivery_barrier(scenario)
}

fn owned(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn assert_suspended_with_pending_steps(
    observation: &DeliveryBarrierObservation,
    expected: &[&str],
) {
    assert!(observation.suspended);
    assert_eq!(observation.pending_steps, owned(expected));
    assert_eq!(observation.replay_error, None);
}

#[test]
fn earlier_step_result_delivers_first_across_drain_windows() {
    let observation = observe(DeliveryBarrierScenario::StepBehindStep);
    assert_suspended_with_pending_steps(&observation, &["afterA", "afterB"]);
}

macro_rules! wait_behind_step_case {
    ($name:ident, $hops:expr) => {
        #[test]
        fn $name() {
            let observation =
                observe(DeliveryBarrierScenario::WaitBehindStep { extra_hops: $hops });
            assert_eq!(observation.extra_hops, Some($hops));
            assert_suspended_with_pending_steps(&observation, &["afterStep", "afterSleep"]);
        }
    };
}

wait_behind_step_case!(wait_behind_step_zero_hops, EXTRA_HOPS[0]);
wait_behind_step_case!(wait_behind_step_one_hop, EXTRA_HOPS[1]);
wait_behind_step_case!(wait_behind_step_three_hops, EXTRA_HOPS[2]);
wait_behind_step_case!(wait_behind_step_eight_hops, EXTRA_HOPS[3]);
wait_behind_step_case!(wait_behind_step_twenty_hops, EXTRA_HOPS[4]);
wait_behind_step_case!(wait_behind_step_fifty_hops, EXTRA_HOPS[5]);

macro_rules! hook_behind_step_case {
    ($name:ident, $hops:expr) => {
        #[test]
        fn $name() {
            let observation =
                observe(DeliveryBarrierScenario::HookBehindStep { extra_hops: $hops });
            assert_eq!(observation.extra_hops, Some($hops));
            assert_suspended_with_pending_steps(&observation, &["afterStep", "afterHook"]);
        }
    };
}

hook_behind_step_case!(hook_behind_step_zero_hops, EXTRA_HOPS[0]);
hook_behind_step_case!(hook_behind_step_one_hop, EXTRA_HOPS[1]);
hook_behind_step_case!(hook_behind_step_three_hops, EXTRA_HOPS[2]);
hook_behind_step_case!(hook_behind_step_eight_hops, EXTRA_HOPS[3]);
hook_behind_step_case!(hook_behind_step_twenty_hops, EXTRA_HOPS[4]);
hook_behind_step_case!(hook_behind_step_fifty_hops, EXTRA_HOPS[5]);

#[test]
fn earlier_hook_payload_delivers_first_for_an_async_iterator_consumer() {
    let observation = observe(DeliveryBarrierScenario::HookBehindHook);
    assert_suspended_with_pending_steps(&observation, &["afterFirst", "afterSecond"]);
}

#[test]
fn earlier_step_result_delivers_before_abort_listener_continuation() {
    let observation = observe(DeliveryBarrierScenario::AbortBehindStep);
    assert_suspended_with_pending_steps(&observation, &["afterStep", "afterAbort"]);
}

#[test]
fn idle_unwinds_a_step_parked_behind_a_wait_and_unclaimed_payload() {
    let observation = observe(DeliveryBarrierScenario::ParkedChainIdleReachability);
    assert_eq!(observation.initial_barriers, 3);
    assert_eq!(observation.pre_idle_order, Vec::<String>::new());
    assert_eq!(observation.reaches_idle, Some(true));
    assert_eq!(observation.payload_retired_before_wait, Some(true));
    assert_eq!(observation.delivery_order, owned(&["wait", "step"]));
    assert_eq!(observation.remaining_barriers, 0);
}

#[test]
fn all_armed_step_batch_blocks_idle_until_delivery() {
    let observation = observe(DeliveryBarrierScenario::AllArmedBatchBlocksIdle);
    assert_eq!(observation.reaches_idle, Some(false));
    assert_eq!(observation.remaining_barriers, 3);
}

#[test]
fn parallel_batch_suspension_snapshot_carries_the_follow_up_step() {
    let observation = observe(DeliveryBarrierScenario::ParallelBatchSuspensionSnapshot);
    assert!(observation.suspended);
    assert_eq!(observation.suspension_snapshot_steps, owned(&["followUp"]));
    assert_eq!(observation.replay_error, None);
}

#[test]
fn single_step_suspension_control_carries_the_follow_up_step() {
    let observation = observe(DeliveryBarrierScenario::SingleStepSuspensionControl);
    assert!(observation.suspended);
    assert_eq!(observation.suspension_snapshot_steps, owned(&["followUp"]));
    assert_eq!(observation.replay_error, None);
}

#[test]
fn parked_chain_turnstile_terminates_with_log_order_draws_enabled() {
    let observation = observe(DeliveryBarrierScenario::TurnstileParkedChain {
        log_order_draws: true,
    });
    assert_eq!(observation.log_order_draws, Some(true));
    assert_suspended_with_pending_steps(&observation, &["afterBoth"]);
}

#[test]
fn parked_chain_turnstile_terminates_with_log_order_draws_disabled() {
    let observation = observe(DeliveryBarrierScenario::TurnstileParkedChain {
        log_order_draws: false,
    });
    assert_eq!(observation.log_order_draws, Some(false));
    assert_suspended_with_pending_steps(&observation, &["afterBoth"]);
}

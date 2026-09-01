use workflow_core_tdd::abort_consistency::{
    AbortConsistencyScenario, run_abort_consistency_scenario,
};

#[test]
fn external_pre_aborted_signal_serializes_its_state() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::PreAbortedSerialized);
    assert!(observation.serialized_contains_aborted);
}

#[test]
fn abort_after_serialization_persists_one_stream_packet() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::LateAbortPersistsStreamPacket);
    assert_eq!(observation.queued_stream_ops_before_abort, 0);
    assert_eq!(observation.queued_stream_ops_after_abort, 1);
    assert!(observation.aborted);
    assert_eq!(observation.reason.as_deref(), Some("late abort"));
}

#[test]
fn reducer_captures_pre_aborted_state_without_a_listener_race() {
    let observation = run_abort_consistency_scenario(
        AbortConsistencyScenario::ListenerInstalledBeforeStateCheck,
    );
    assert!(observation.serialized_contains_aborted);
    assert_eq!(observation.queued_stream_ops_after_abort, 0);
}

#[test]
fn workflow_signal_stays_live_until_hook_receipt_is_replayed() {
    let observation = run_abort_consistency_scenario(
        AbortConsistencyScenario::WorkflowRemainsLiveBeforeHookReceipt,
    );
    assert!(!observation.workflow_aborted);
    assert!(observation.workflow_suspended || observation.workflow_completed);
}

#[test]
fn stream_only_abort_reaches_the_step() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::StreamOnlyAbortReachesStep);
    assert!(observation.step_received_abort);
    assert_eq!(observation.queued_stream_ops_after_abort, 1);
}

#[test]
fn missing_hook_receipt_leaves_workflow_replay_unaborted() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::MissingHookLeavesReplayLive);
    assert!(observation.workflow_suspended);
    assert!(!observation.workflow_aborted);
    assert!(!observation.hook_abort_requested);
}

#[test]
fn step_abort_keeps_stream_and_hook_identity_bound_to_the_signal() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::StepAbortCarriesStableSymbols);
    assert_eq!(observation.queued_stream_ops_after_abort, 1);
    assert!(observation.stream_name_bound);
    assert!(observation.hook_token_bound);
    assert!(observation.signal_symbols_match_controller);
}

#[test]
fn hook_only_abort_is_visible_during_workflow_replay() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::HookOnlyAbortReachesReplay);
    assert!(observation.hook_event_present);
    assert!(observation.workflow_aborted);
}

#[test]
fn stream_failure_degrades_without_an_unhandled_error() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::StreamFailureDegradesGracefully);
    assert!(observation.aborted);
    assert_eq!(observation.queued_stream_ops_after_abort, 1);
    assert_eq!(observation.unhandled_errors, 0);
}

#[test]
fn dual_failure_does_not_corrupt_abort_metadata() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::DualFailureDoesNotCorruptState);
    assert!(observation.aborted);
    assert_eq!(observation.reason.as_deref(), Some("both will fail"));
    assert!(observation.stream_name_bound);
    assert!(observation.hook_token_bound);
    assert!(!observation.corrupted);
    assert_eq!(observation.unhandled_errors, 0);
}

#[test]
fn abort_after_step_completion_is_idempotent() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::AbortAfterStepCompletion);
    assert!(observation.aborted);
    assert!(!observation.corrupted);
    assert_eq!(observation.unhandled_errors, 0);
}

#[test]
fn orphan_abort_packet_is_safe_and_preserves_local_reason() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::OrphanAbortPacket);
    assert_eq!(observation.queued_stream_ops_before_abort, 0);
    assert_eq!(observation.queued_stream_ops_after_abort, 1);
    assert!(observation.aborted);
    assert_eq!(observation.reason.as_deref(), Some("orphan abort"));
}

#[test]
fn double_abort_emits_at_most_one_packet_and_keeps_the_first_reason() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::DoubleAbortIsIdempotent);
    assert!(observation.queued_stream_ops_after_abort <= 1);
    assert!(observation.aborted);
    assert_eq!(observation.reason.as_deref(), Some("first"));
}

#[test]
fn first_run_abort_listener_fires_synchronously_at_the_call_site() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::FirstRunListenerOrder);
    assert_eq!(
        observation.listener_order,
        vec!["before-abort", "listener-fired", "after-abort"]
    );
}

#[test]
fn replay_abort_sets_state_fires_listeners_and_keeps_its_reason() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::ReplayAbortDelivery);
    assert!(observation.aborted);
    assert_eq!(observation.listener_calls, 1);
    assert_eq!(observation.reason.as_deref(), Some("replay-reason"));
}

#[test]
fn cross_execution_abort_is_a_durable_replay_fact() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::CrossExecutionAbortDelivery);
    assert!(observation.aborted);
    assert_eq!(observation.reason.as_deref(), Some("step-aborted"));
}

#[test]
fn listener_registered_after_replay_abort_fires_immediately() {
    let observation = run_abort_consistency_scenario(
        AbortConsistencyScenario::LateListenerObservesReplayAbort,
    );
    assert!(observation.aborted);
    assert_eq!(observation.listener_calls, 1);
}

#[test]
fn abort_after_last_suspension_point_does_not_block_completion() {
    let observation = run_abort_consistency_scenario(
        AbortConsistencyScenario::AbortAfterLastSuspensionPoint,
    );
    assert!(observation.workflow_completed);
    assert_eq!(observation.return_value.as_deref(), Some("done"));
}

#[test]
fn fire_and_forget_sleep_does_not_block_workflow_completion() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::FireAndForgetSleep);
    assert!(observation.workflow_completed);
    assert_eq!(observation.return_value.as_deref(), Some("done"));
}

#[test]
fn pending_step_is_preserved_at_suspension() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::PendingStepAtSuspension);
    assert!(observation.workflow_suspended);
    assert_eq!(observation.step_count, 1);
    assert_eq!(observation.pending_step_name.as_deref(), Some("add"));
    assert_eq!(observation.pending_step_args, vec![1, 2]);
}

#[test]
fn pending_hook_is_preserved_at_suspension() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::PendingHookAtSuspension);
    assert!(observation.workflow_suspended);
    assert!(observation.hook_count >= 1);
    assert_eq!(observation.pending_hook_token.as_deref(), Some("test-hook"));
}

#[test]
fn pending_wait_is_preserved_at_suspension() {
    let observation =
        run_abort_consistency_scenario(AbortConsistencyScenario::PendingWaitAtSuspension);
    assert!(observation.workflow_suspended);
    assert_eq!(observation.wait_count, 1);
    assert!(observation.wait_has_resume_at);
}

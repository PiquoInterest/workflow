use workflow_core_tdd::abort_controller::{
    AbortControllerObservation, AbortControllerScenario, AbortReason, InvocationKind,
    WorkflowErrorKind, observe_abort_controller,
};

fn observe(scenario: AbortControllerScenario) -> AbortControllerObservation {
    observe_abort_controller(scenario)
}

fn only_hook(
    observation: &AbortControllerObservation,
) -> &workflow_core_tdd::abort_controller::HookRecord {
    assert_eq!(observation.hooks_after_abort.len(), 1);
    &observation.hooks_after_abort[0]
}

#[test]
fn constructor_exposes_signal_and_abort_method() {
    let observation = observe(AbortControllerScenario::Construct);
    assert!(observation.signal_defined);
    assert!(observation.abort_method_defined);
    assert!(observation.abort_method_is_function);
}

#[test]
fn abort_without_reason_marks_the_durable_hook_for_resumption() {
    let observation = observe(AbortControllerScenario::AbortWithoutReason);
    let hook = only_hook(&observation);
    assert!(hook.abort_requested);
}

#[test]
fn abort_with_reason_preserves_the_exact_reason_on_the_hook() {
    let observation = observe(AbortControllerScenario::AbortWithReason);
    let hook = only_hook(&observation);
    assert_eq!(hook.abort_reason, AbortReason::error("custom reason"));
}

#[test]
fn abort_called_twice_is_idempotent_after_the_signal_is_aborted() {
    let observation = observe(AbortControllerScenario::DoubleAbort);
    assert!(observation.signal_aborted);
}

#[test]
fn replay_token_mismatch_reports_a_typed_divergence_error() {
    let observation = observe(AbortControllerScenario::ReplayTokenMismatch);
    assert_eq!(
        observation.workflow_error_kind,
        Some(WorkflowErrorKind::ReplayDivergence)
    );
    assert_eq!(
        observation.workflow_error_event_id.as_deref(),
        Some("evnt_0")
    );
    assert_eq!(observation.observed_token.as_deref(), Some("wrong-token"));
    let expected = observation.expected_token.as_deref().unwrap();
    let message = observation.workflow_error_message.as_deref().unwrap();
    assert!(message.contains("hook_received"));
    assert!(message.contains("wrong-token"));
    assert!(message.contains(expected));
}

#[test]
fn duplicate_matching_abort_receipts_are_consumed_idempotently() {
    let observation = observe(AbortControllerScenario::DuplicateReceipts);
    assert!(observation.signal_aborted);
    assert_eq!(observation.listener_calls, 1);
    assert_eq!(observation.event_index, 2);
    assert_eq!(observation.consumed_event_count, 2);
    assert_eq!(observation.unconsumed_event_count, 0);
}

#[test]
fn signal_is_not_aborted_initially() {
    let observation = observe(AbortControllerScenario::InitialSignal);
    assert!(!observation.signal_aborted);
    assert_eq!(observation.signal_reason, AbortReason::Undefined);
}

#[test]
fn abort_listener_fires_once_when_the_signal_is_aborted() {
    let observation = observe(AbortControllerScenario::ListenerFires);
    assert!(observation.signal_aborted);
    assert_eq!(observation.listener_calls, 1);
}

#[test]
fn removed_abort_listener_does_not_fire() {
    let observation = observe(AbortControllerScenario::RemovedListener);
    assert!(observation.signal_aborted);
    assert_eq!(observation.listener_calls, 0);
}

#[test]
fn throw_if_aborted_throws_the_default_abort_error() {
    let observation = observe(AbortControllerScenario::ThrowWhenAborted);
    assert!(observation.throw_if_aborted.threw);
    assert_eq!(
        observation.throw_if_aborted.message.as_deref(),
        Some("The operation was aborted.")
    );
}

#[test]
fn throw_if_aborted_is_a_noop_for_an_active_signal() {
    let observation = observe(AbortControllerScenario::ThrowWhenNotAborted);
    assert!(!observation.throw_if_aborted.threw);
    assert_eq!(observation.throw_if_aborted.message, None);
}

#[test]
fn multiple_controllers_keep_independent_state_and_reason() {
    let observation = observe(AbortControllerScenario::IndependentControllers);
    assert!(observation.signal_aborted);
    assert_eq!(observation.signal_reason, AbortReason::error("c1 reason"));
    assert_eq!(observation.secondary_signal_aborted, Some(false));
    assert_eq!(
        observation.secondary_signal_reason,
        Some(AbortReason::Undefined)
    );
}

#[test]
fn static_abort_returns_a_pre_aborted_signal_with_abort_error() {
    let observation = observe(AbortControllerScenario::StaticAbortDefault);
    assert!(observation.signal_aborted);
    assert_eq!(
        observation.signal_reason,
        AbortReason::DomException {
            name: "AbortError".to_owned(),
            message: "The operation was aborted.".to_owned(),
        }
    );
}

#[test]
fn static_abort_preserves_a_custom_reason() {
    let observation = observe(AbortControllerScenario::StaticAbortReason);
    assert!(observation.signal_aborted);
    assert_eq!(observation.signal_reason, AbortReason::error("custom"));
}

#[test]
fn any_aborts_when_a_later_input_aborts() {
    let observation = observe(AbortControllerScenario::AnyLaterAbort);
    assert!(observation.signal_aborted);
    assert_eq!(observation.signal_reason, AbortReason::error("c2 aborted"));
    assert_eq!(observation.listener_calls, 1);
}

#[test]
fn any_is_immediately_aborted_by_a_pre_aborted_input() {
    let observation = observe(AbortControllerScenario::AnyPreAborted);
    assert!(observation.signal_aborted);
    assert_eq!(
        observation.signal_reason,
        AbortReason::error("already aborted")
    );
}

#[test]
fn any_materializes_a_single_shot_iterable_only_once() {
    let observation = observe(AbortControllerScenario::AnySingleShotIterable);
    assert_eq!(observation.iterable_passes, 1);
    assert!(observation.signal_aborted);
    assert_eq!(
        observation.signal_reason,
        AbortReason::error("after-iterable")
    );
}

#[test]
fn any_removes_listeners_from_every_input_after_aborting() {
    let observation = observe(AbortControllerScenario::AnyListenerCleanup);
    assert!(observation.signal_aborted);
    assert_eq!(observation.input_listener_removals, vec![1, 1]);
}

#[test]
fn timeout_is_rejected_inside_workflow_functions() {
    let observation = observe(AbortControllerScenario::TimeoutUnsupported);
    let error = observation.timeout_error.as_deref().unwrap();
    assert!(error.contains("AbortSignal.timeout() is not supported in workflow functions"));
}

#[test]
fn constructing_a_controller_registers_one_internal_system_hook() {
    let observation = observe(AbortControllerScenario::HookCreation);
    assert_eq!(observation.queue_size_before, 0);
    assert_eq!(observation.queue_size_after, 1);
    let hook = only_hook(&observation);
    assert_eq!(hook.kind, InvocationKind::Hook);
    assert!(hook.is_system);
    assert!(!hook.is_webhook);
    assert!(hook.token.starts_with("abrt_"));
}

#[test]
fn abort_marks_the_matching_hook_and_preserves_text_reason() {
    let observation = observe(AbortControllerScenario::HookAbortQueue);
    assert_eq!(observation.hooks_before_abort.len(), 1);
    assert!(!observation.hooks_before_abort[0].abort_requested);
    let hook = only_hook(&observation);
    assert!(hook.abort_requested);
    assert_eq!(hook.abort_reason, AbortReason::text("test-reason"));
}

#[test]
fn deterministic_seed_reuses_the_abort_hook_token_across_replays() {
    let observation = observe(AbortControllerScenario::ReplayStableToken);
    assert_eq!(observation.replay_tokens.len(), 2);
    assert!(!observation.replay_tokens[0].is_empty());
    assert_eq!(observation.replay_tokens[0], observation.replay_tokens[1]);
}

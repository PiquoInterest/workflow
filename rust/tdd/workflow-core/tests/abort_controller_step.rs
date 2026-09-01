use workflow_core_tdd::abort_controller_step::{AbortStepScenario, run_abort_step_scenario};

#[test]
fn deserialized_signal_registers_a_stream_reader_operation() {
    let observation = run_abort_step_scenario(AbortStepScenario::ReaderRegistered);
    assert!(observation.reader_ops > 0);
}

#[test]
fn already_aborted_signal_does_not_register_a_stream_reader() {
    let observation = run_abort_step_scenario(AbortStepScenario::PreAbortedSkipsReader);
    assert_eq!(observation.reader_ops, 0);
}

#[test]
fn already_aborted_signal_is_visible_immediately() {
    let observation = run_abort_step_scenario(AbortStepScenario::PreAbortedImmediatelyVisible);
    assert!(observation.aborted);
    assert_eq!(observation.reason.as_deref(), Some("already-done"));
}

#[test]
fn stream_packet_aborts_the_deserialized_signal() {
    let observation = run_abort_step_scenario(AbortStepScenario::StreamPacketAborts);
    assert!(observation.aborted);
}

#[test]
fn stream_packet_preserves_the_abort_reason() {
    let observation = run_abort_step_scenario(AbortStepScenario::StreamReasonPropagates);
    assert!(observation.aborted);
    assert_eq!(observation.reason.as_deref(), Some("custom-abort-reason"));
}

#[test]
fn abort_listener_fires_when_the_stream_packet_arrives() {
    let observation = run_abort_step_scenario(AbortStepScenario::ListenerFires);
    assert_eq!(observation.listener_calls, 1);
}

#[test]
fn throw_if_aborted_throws_after_stream_delivery() {
    let observation = run_abort_step_scenario(AbortStepScenario::ThrowIfAborted);
    assert!(observation.throws_when_checked);
}

#[test]
fn step_abort_queues_the_stream_write_in_background_operations() {
    let observation = run_abort_step_scenario(AbortStepScenario::StepAbortQueuesStreamWrite);
    assert!(observation.background_ops >= 1);
    assert_eq!(observation.stream_writes, 1);
}

#[test]
fn step_abort_queues_the_durable_hook_resume() {
    let observation = run_abort_step_scenario(AbortStepScenario::StepAbortQueuesDurableHookResume);
    assert_eq!(observation.durable_hook_resumes, 1);
    assert_eq!(
        observation.durable_hook_token.as_deref(),
        Some("abrt_test9")
    );
    assert_eq!(
        observation.durable_hook_reason.as_deref(),
        Some("hook-resume-test")
    );
}

#[test]
fn step_abort_sets_aborted_synchronously() {
    let observation = run_abort_step_scenario(AbortStepScenario::StepAbortIsSynchronous);
    assert!(observation.aborted);
}

#[test]
fn abort_after_step_context_exit_does_not_crash() {
    let observation = run_abort_step_scenario(AbortStepScenario::AbortOutsideStepContext);
    assert!(!observation.crashed);
    assert!(observation.aborted);
}

#[test]
fn every_consumer_of_one_abort_stream_observes_the_abort() {
    let observation = run_abort_step_scenario(AbortStepScenario::SharedStreamAbortsEveryConsumer);
    assert_eq!(observation.consumer_abort_states, vec![true, true]);
}

#[test]
fn composite_signal_aborts_when_its_local_member_aborts() {
    let observation = run_abort_step_scenario(AbortStepScenario::CompositeSignalWithLocalAbort);
    assert!(observation.composite_aborted);
}

#[test]
fn fetch_abort_error_is_wrapped_as_fatal() {
    let observation = run_abort_step_scenario(AbortStepScenario::FetchAbortIsFatal);
    assert!(observation.fatal);
    assert_eq!(
        observation.error_message.as_deref(),
        Some("The operation was aborted")
    );
}

#[test]
fn throw_if_aborted_error_is_wrapped_as_fatal() {
    let observation = run_abort_step_scenario(AbortStepScenario::ThrowIfAbortedIsFatal);
    assert!(observation.fatal);
}

#[test]
fn custom_abort_reason_survives_the_fatal_wrapper() {
    let observation = run_abort_step_scenario(AbortStepScenario::CustomAbortReasonIsFatal);
    assert!(observation.fatal);
    assert_eq!(observation.error_message.as_deref(), Some("user-cancelled"));
}

#[test]
fn fatal_abort_skips_retry_policy() {
    let observation = run_abort_step_scenario(AbortStepScenario::FatalAbortSkipsRetries);
    assert!(observation.fatal);
    assert!(observation.retries_skipped);
}

#[test]
fn unrelated_step_error_is_not_reclassified_as_fatal() {
    let observation = run_abort_step_scenario(AbortStepScenario::RegularErrorRemainsNonFatal);
    assert!(!observation.fatal);
    assert_eq!(
        observation.error_message.as_deref(),
        Some("network timeout")
    );
}

#[test]
fn durable_hook_resume_is_committed_before_step_completion() {
    let observation =
        run_abort_step_scenario(AbortStepScenario::DurableHookResumePrecedesStepCompletion);
    assert!(observation.aborted);
    assert_eq!(observation.pre_completion_ops, 1);
    assert_eq!(observation.durable_hook_resumes, 1);
    assert_eq!(observation.lazy_hook_resumes, 0);
    assert_eq!(
        observation.durable_hook_token.as_deref(),
        Some("abrt_pre_completion")
    );
    assert_eq!(
        observation.durable_hook_reason.as_deref(),
        Some("aborted from step")
    );
}

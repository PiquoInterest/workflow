use workflow_core_tdd::duplicate_events::{
    DuplicateNotification, DuplicateReplayScenario, replay_duplicate_scenario,
};

#[test]
fn ignores_step_started_after_the_step_completed() {
    let observation =
        replay_duplicate_scenario(DuplicateReplayScenario::StepStartedAfterCompletion);
    assert!(observation.suspended);
    assert_eq!(observation.observed_values, vec!["a-result"]);
    assert_eq!(observation.pending_step_names, vec!["stepB"]);
    assert_eq!(
        observation.duplicate_notifications,
        vec![DuplicateNotification {
            event_index: 3,
            event_class: "step_started".to_owned(),
        }]
    );
}

#[test]
fn ignores_wait_created_after_the_wait_completed() {
    let observation =
        replay_duplicate_scenario(DuplicateReplayScenario::WaitCreatedAfterCompletion);
    assert!(observation.suspended);
    assert_eq!(observation.pending_step_names, vec!["afterSleep"]);
    assert_eq!(
        observation.duplicate_notifications,
        vec![DuplicateNotification {
            event_index: 2,
            event_class: "wait_created".to_owned(),
        }]
    );
}

#[test]
fn ignores_a_second_disposal_for_an_already_disposed_hook() {
    let observation = replay_duplicate_scenario(DuplicateReplayScenario::SecondHookDisposal);
    assert!(observation.suspended);
    assert_eq!(observation.pending_step_names, vec!["afterHook"]);
    assert_eq!(
        observation.duplicate_notifications,
        vec![DuplicateNotification {
            event_index: 2,
            event_class: "hook_disposed".to_owned(),
        }]
    );
}

#[test]
fn ignores_hook_creation_after_the_hook_was_disposed() {
    let observation = replay_duplicate_scenario(DuplicateReplayScenario::HookCreatedAfterDisposal);
    assert!(observation.suspended);
    assert_eq!(observation.pending_step_names, vec!["afterHook"]);
    assert_eq!(
        observation.duplicate_notifications,
        vec![DuplicateNotification {
            event_index: 2,
            event_class: "hook_created".to_owned(),
        }]
    );
}

#[test]
fn ignores_a_second_attribute_write_and_leaves_no_stranded_event() {
    let observation = replay_duplicate_scenario(DuplicateReplayScenario::SecondAttributeWrite);
    assert!(!observation.suspended);
    assert_eq!(observation.result.as_deref(), Some("done"));
    assert_eq!(observation.observed_values, vec!["a-result"]);
    assert_eq!(
        observation.duplicate_notifications,
        vec![DuplicateNotification {
            event_index: 1,
            event_class: "attr_set".to_owned(),
        }]
    );
    assert_eq!(observation.stranded_event, None);
    assert_eq!(observation.error, None);
}

#[test]
fn unrelated_unconsumed_events_still_fail_as_divergence() {
    let observation = replay_duplicate_scenario(DuplicateReplayScenario::UnrelatedUnconsumedEvent);
    assert!(!observation.suspended);
    assert!(
        observation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Unconsumed event in event log")
    );
    assert!(observation.duplicate_notifications.is_empty());
}

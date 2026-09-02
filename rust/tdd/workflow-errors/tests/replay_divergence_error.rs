use std::collections::BTreeSet;

use workflow_errors_tdd::{
    GuardCandidate, PropertyValue, is_replay_divergence, replay_divergence_error,
};

#[test]
fn is_a_retryable_replay_signal_with_its_own_documentation_link() {
    let error = replay_divergence_error("consumer mismatch", "event-1");

    assert_eq!(error.name, "ReplayDivergenceError");
    assert!(error.hierarchy.contains("WorkflowRuntimeError"));
    assert_eq!(error.event_id.as_deref(), Some("event-1"));
    assert!(error.message.contains("replay-divergence"));
    assert!(is_replay_divergence(&GuardCandidate::Workflow(error)));
}

#[test]
fn does_not_treat_an_error_without_event_id_as_replay_divergence() {
    assert!(!is_replay_divergence(&GuardCandidate::ForeignError {
        name: "ReplayDivergenceError".to_owned(),
        fatal: PropertyValue::Missing,
        fields: BTreeSet::new(),
    }));
}

use workflow_errors_tdd::{CauseToken, GuardCandidate, corrupted_event_log_error, is_named_error};

#[test]
fn sets_the_name_and_extends_workflow_runtime_error() {
    let error = corrupted_event_log_error("event mismatch", None);

    assert_eq!(error.name, "CorruptedEventLogError");
    assert!(error.hierarchy.contains("WorkflowError"));
    assert!(error.hierarchy.contains("WorkflowRuntimeError"));
    assert!(error.hierarchy.contains("CorruptedEventLogError"));
}

#[test]
fn adds_the_corrupted_event_log_docs_link() {
    let error = corrupted_event_log_error("event mismatch", None);
    assert!(
        error
            .message
            .contains("https://workflow-sdk.dev/err/corrupted-event-log")
    );
}

#[test]
fn preserves_cause_for_debugging() {
    let cause = CauseToken(2);
    let error = corrupted_event_log_error("event mismatch", Some(cause.clone()));
    assert_eq!(error.cause, Some(cause));
}

#[test]
fn type_guard_discriminates_by_name() {
    let error = corrupted_event_log_error("event mismatch", None);
    assert!(is_named_error(
        &GuardCandidate::Workflow(error),
        "CorruptedEventLogError"
    ));
    assert!(!is_named_error(
        &GuardCandidate::NonError,
        "CorruptedEventLogError"
    ));
}

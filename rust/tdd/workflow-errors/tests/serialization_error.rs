use workflow_errors_tdd::{
    CauseToken, GuardCandidate, is_fatal, is_named_error, serialization_error,
};

#[test]
fn sets_the_name_and_extends_workflow_error() {
    let error = serialization_error("boom", None, None);

    assert_eq!(error.name, "SerializationError");
    assert!(error.hierarchy.contains("WorkflowError"));
    assert!(error.hierarchy.contains("SerializationError"));
}

#[test]
fn renders_just_the_title_when_no_hint_is_provided() {
    let error = serialization_error("boom", None, None);
    assert_eq!(error.message, "boom");
}

#[test]
fn renders_the_hint_as_a_framed_branch() {
    let error = serialization_error(
        "boom",
        Some("Register the class with WORKFLOW_SERIALIZE."),
        None,
    );

    assert_eq!(
        error.hint.as_deref(),
        Some("Register the class with WORKFLOW_SERIALIZE.")
    );
    assert_eq!(
        error.message,
        "boom\n╰▶ hint: Register the class with WORKFLOW_SERIALIZE."
    );
}

#[test]
fn preserves_cause_for_debugging() {
    let cause = CauseToken(4);
    let error = serialization_error("boom", None, Some(cause.clone()));
    assert_eq!(error.cause, Some(cause));
}

#[test]
fn type_guard_discriminates_by_name() {
    let error = serialization_error("boom", None, None);
    assert!(is_named_error(
        &GuardCandidate::Workflow(error),
        "SerializationError"
    ));
    assert!(!is_named_error(
        &GuardCandidate::NonError,
        "SerializationError"
    ));
}

#[test]
fn is_fatal_and_short_circuits_the_retry_loop() {
    let error = serialization_error("boom", None, None);
    assert!(error.fatal);
    assert!(is_fatal(&GuardCandidate::Workflow(error)));
}

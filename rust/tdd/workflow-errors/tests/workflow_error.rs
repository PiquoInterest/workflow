use workflow_errors_tdd::{GuardCandidate, is_named_error, workflow_error};

#[test]
fn uses_its_public_class_name_and_passes_its_own_type_guard() {
    let error = workflow_error("boom", None, None);

    assert_eq!(error.name, "WorkflowError");
    assert!(error.hierarchy.contains("WorkflowError"));
    assert!(is_named_error(
        &GuardCandidate::Workflow(error),
        "WorkflowError"
    ));
}

#[test]
fn keeps_documentation_framing_stable() {
    let error = workflow_error(
        "event history is invalid",
        Some("corrupted-event-log"),
        None,
    );

    assert_eq!(
        error.message,
        "event history is invalid\n╰▶ docs: https://workflow-sdk.dev/err/corrupted-event-log"
    );
}

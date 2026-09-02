use workflow_errors_tdd::{CauseToken, GuardCandidate, is_named_error, workflow_build_error};

#[test]
fn sets_the_name_and_extends_workflow_error() {
    let error = workflow_build_error("boom", None, None);

    assert_eq!(error.name, "WorkflowBuildError");
    assert!(error.hierarchy.contains("WorkflowError"));
    assert!(error.hierarchy.contains("WorkflowBuildError"));
}

#[test]
fn appends_hint_with_a_blank_line_separator() {
    let error = workflow_build_error(
        "Build failed during steps",
        Some("run `pnpm install workflow` and try again"),
        None,
    );

    assert_eq!(
        error.hint.as_deref(),
        Some("run `pnpm install workflow` and try again")
    );
    assert_eq!(
        error.message,
        "Build failed during steps\n╰▶ hint: run `pnpm install workflow` and try again"
    );
}

#[test]
fn preserves_cause_for_debugging() {
    let cause = CauseToken(1);
    let error = workflow_build_error("boom", None, Some(cause.clone()));
    assert_eq!(error.cause, Some(cause));
}

#[test]
fn type_guard_discriminates_by_name() {
    let error = workflow_build_error("boom", None, None);
    assert!(is_named_error(
        &GuardCandidate::Workflow(error),
        "WorkflowBuildError"
    ));
    assert!(!is_named_error(
        &GuardCandidate::ForeignError {
            name: "Error".to_owned(),
            fatal: workflow_errors_tdd::PropertyValue::Missing,
            fields: Default::default(),
        },
        "WorkflowBuildError"
    ));
    assert!(!is_named_error(
        &GuardCandidate::NonError,
        "WorkflowBuildError"
    ));
}

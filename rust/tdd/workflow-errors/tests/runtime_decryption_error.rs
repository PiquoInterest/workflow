use std::collections::BTreeMap;

use workflow_errors_tdd::{
    CauseToken, DiagnosticValue, GuardCandidate, is_named_error, runtime_decryption_error,
};

#[test]
fn sets_the_name_and_extends_workflow_runtime_error() {
    let error = runtime_decryption_error("decrypt failed", None, None);

    assert_eq!(error.name, "RuntimeDecryptionError");
    assert!(error.hierarchy.contains("WorkflowError"));
    assert!(error.hierarchy.contains("WorkflowRuntimeError"));
    assert!(error.hierarchy.contains("RuntimeDecryptionError"));
}

#[test]
fn adds_the_runtime_decryption_failed_docs_link() {
    let error = runtime_decryption_error("decrypt failed", None, None);
    assert!(
        error
            .message
            .contains("https://workflow-sdk.dev/err/runtime-decryption-failed")
    );
}

#[test]
fn preserves_cause_for_debugging() {
    let cause = CauseToken(3);
    let error = runtime_decryption_error("decrypt failed", Some(cause.clone()), None);
    assert_eq!(error.cause, Some(cause));
}

#[test]
fn records_optional_diagnostic_context() {
    let context = BTreeMap::from([
        (
            "operation".to_owned(),
            DiagnosticValue::Text("decrypt".to_owned()),
        ),
        ("byteLength".to_owned(), DiagnosticValue::Integer(42)),
        (
            "formatPrefix".to_owned(),
            DiagnosticValue::Text("encr".to_owned()),
        ),
    ]);
    let error = runtime_decryption_error("decrypt failed", None, Some(context.clone()));
    assert_eq!(error.context, Some(context));
}

#[test]
fn omits_context_when_not_provided() {
    let error = runtime_decryption_error("decrypt failed", None, None);
    assert!(error.context.is_none());
}

#[test]
fn type_guard_discriminates_by_name() {
    let error = runtime_decryption_error("decrypt failed", None, None);
    assert!(is_named_error(
        &GuardCandidate::Workflow(error),
        "RuntimeDecryptionError"
    ));
    assert!(!is_named_error(
        &GuardCandidate::NonError,
        "RuntimeDecryptionError"
    ));
}

use std::collections::BTreeSet;

use workflow_errors_tdd::{
    GuardCandidate, PropertyValue, fatal_error, is_fatal,
};

#[test]
fn returns_true_for_direct_fatal_error_instances() {
    let error = fatal_error("boom");
    assert!(is_fatal(&GuardCandidate::Workflow(error)));
}

#[test]
fn returns_true_for_any_error_with_fatal_true() {
    assert!(is_fatal(&GuardCandidate::ForeignError {
        name: "ContextViolation".to_owned(),
        fatal: PropertyValue::Bool(true),
        fields: BTreeSet::new(),
    }));
}

#[test]
fn returns_false_for_plain_errors() {
    assert!(!is_fatal(&GuardCandidate::ForeignError {
        name: "Error".to_owned(),
        fatal: PropertyValue::Missing,
        fields: BTreeSet::new(),
    }));
}

#[test]
fn returns_false_for_non_error_values() {
    assert!(!is_fatal(&GuardCandidate::NonError));
}

#[test]
fn returns_false_when_fatal_is_not_strictly_true() {
    for fatal in [
        PropertyValue::Integer(1),
        PropertyValue::Text("yes".to_owned()),
        PropertyValue::Bool(false),
    ] {
        assert!(!is_fatal(&GuardCandidate::ForeignError {
            name: "Weird".to_owned(),
            fatal,
            fields: BTreeSet::new(),
        }));
    }
}

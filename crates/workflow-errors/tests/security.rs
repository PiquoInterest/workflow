use std::collections::BTreeSet;

use workflow_errors::{
    GuardCandidate, PropertyValue, ansi, is_fatal, is_named_error, is_replay_divergence,
};

#[test]
fn inline_annotation_clamps_untrusted_offsets_before_allocating() {
    assert_eq!(
        ansi::inline_annotation("abc", usize::MAX, usize::MAX, "bounded"),
        "abc\n   ┬\n   ╰▶ bounded"
    );
}

#[test]
fn named_error_guard_requires_an_exact_public_name() {
    assert!(!is_named_error(
        &GuardCandidate::ForeignError {
            name: "WorkflowErrorExtra".to_owned(),
            fatal: PropertyValue::Missing,
            fields: BTreeSet::new(),
        },
        "WorkflowError"
    ));
}

#[test]
fn replay_guard_requires_both_name_and_event_id_field() {
    assert!(!is_replay_divergence(&GuardCandidate::ForeignError {
        name: "ReplayDivergenceError".to_owned(),
        fatal: PropertyValue::Missing,
        fields: BTreeSet::new(),
    }));
    assert!(is_replay_divergence(&GuardCandidate::ForeignError {
        name: "ReplayDivergenceError".to_owned(),
        fatal: PropertyValue::Missing,
        fields: BTreeSet::from(["eventId".to_owned()]),
    }));
}

#[test]
fn fatal_guard_rejects_truthy_non_boolean_properties() {
    for fatal in [
        PropertyValue::Integer(1),
        PropertyValue::Text("true".to_owned()),
        PropertyValue::Bool(false),
    ] {
        assert!(!is_fatal(&GuardCandidate::ForeignError {
            name: "Error".to_owned(),
            fatal,
            fields: BTreeSet::new(),
        }));
    }
}

use std::collections::{BTreeMap, BTreeSet};

use workflow_world::attributes::{
    ATTRIBUTE_KEY_MAX_LENGTH, ATTRIBUTE_MAX_PER_RUN, ATTRIBUTE_VALUE_MAX_BYTES, AttributeChange,
    AttributeValidationOptions, apply_attribute_changes, validate_attribute_changes,
    validate_attribute_key, validate_attribute_value,
};

fn change(key: &str, value: Option<&str>) -> AttributeChange {
    AttributeChange {
        key: key.to_owned(),
        value: value.map(ToOwned::to_owned),
    }
}

#[test]
fn accepts_a_normal_attribute_key() {
    assert!(validate_attribute_key("phase", true).is_ok());
}

#[test]
fn rejects_an_empty_attribute_key() {
    assert!(validate_attribute_key("", true).is_err());
}

#[test]
fn rejects_an_attribute_key_over_the_cap() {
    assert!(validate_attribute_key(&"k".repeat(ATTRIBUTE_KEY_MAX_LENGTH + 1), true).is_err());
}

#[test]
fn accepts_an_attribute_key_exactly_at_the_cap() {
    assert!(validate_attribute_key(&"k".repeat(ATTRIBUTE_KEY_MAX_LENGTH), true).is_ok());
}

#[test]
fn accepts_null_as_an_attribute_deletion() {
    assert!(validate_attribute_value(None).is_ok());
}

#[test]
fn accepts_a_normal_attribute_value() {
    assert!(validate_attribute_value(Some("running")).is_ok());
}

#[test]
fn rejects_an_attribute_value_over_the_utf8_byte_cap() {
    assert!(validate_attribute_value(Some(&"v".repeat(ATTRIBUTE_VALUE_MAX_BYTES + 1))).is_err());
}

#[test]
fn measures_attribute_values_in_utf8_bytes() {
    assert!(validate_attribute_value(Some(&"é".repeat(129))).is_err());
    assert!(validate_attribute_value(Some(&"💥".repeat(65))).is_err());
    assert!(validate_attribute_value(Some(&"💥".repeat(64))).is_ok());
}

#[test]
fn accepts_complete_attribute_changes_and_batches() {
    let changes = [change("phase", Some("running")), change("stale", None)];
    assert!(validate_attribute_changes(&changes, &AttributeValidationOptions::default()).is_ok());
}

#[test]
fn rejects_a_batch_that_exceeds_the_attribute_cap() {
    let changes: Vec<_> = (0..=ATTRIBUTE_MAX_PER_RUN)
        .map(|index| change(&format!("k{index}"), Some("v")))
        .collect();
    assert!(validate_attribute_changes(&changes, &AttributeValidationOptions::default()).is_err());
}

#[test]
fn keeps_reserved_key_policy_contextual() {
    assert!(validate_attribute_key("$rootRunId", true).is_ok());
    assert!(validate_attribute_key("$rootRunId", false).is_err());
}

#[test]
fn validates_a_small_attribute_batch() {
    let changes = [change("phase", Some("running")), change("stale", None)];
    assert!(validate_attribute_changes(&changes, &AttributeValidationOptions::default()).is_ok());
}

#[test]
fn rejects_duplicate_keys_in_one_batch() {
    let changes = [change("phase", Some("one")), change("phase", Some("two"))];
    assert!(validate_attribute_changes(&changes, &AttributeValidationOptions::default()).is_err());
}

#[test]
fn rejects_an_exact_post_merge_count_above_the_cap() {
    let existing_keys = (0..(ATTRIBUTE_MAX_PER_RUN - 1))
        .map(|index| format!("k{index}"))
        .collect::<BTreeSet<_>>();
    let changes = [change("new-a", Some("a")), change("new-b", Some("b"))];
    let options = AttributeValidationOptions {
        existing_keys: Some(existing_keys),
        allow_reserved_attributes: false,
    };
    assert!(validate_attribute_changes(&changes, &options).is_err());
}

#[test]
fn an_unknown_deletion_does_not_reduce_the_post_merge_count() {
    let mut changes: Vec<_> = (0..=ATTRIBUTE_MAX_PER_RUN)
        .map(|index| change(&format!("k{index}"), Some("v")))
        .collect();
    changes.push(change("missing", None));
    assert!(validate_attribute_changes(&changes, &AttributeValidationOptions::default()).is_err());
}

#[test]
fn upserting_an_existing_key_does_not_grow_the_count() {
    let existing_keys = (0..ATTRIBUTE_MAX_PER_RUN)
        .map(|index| format!("k{index}"))
        .collect::<BTreeSet<_>>();
    let options = AttributeValidationOptions {
        existing_keys: Some(existing_keys),
        allow_reserved_attributes: false,
    };
    assert!(validate_attribute_changes(&[change("k0", Some("updated"))], &options).is_ok());
}

#[test]
fn reserved_attribute_keys_are_rejected_by_default() {
    assert!(
        validate_attribute_changes(
            &[change("$framework.kind", Some("agent"))],
            &AttributeValidationOptions::default(),
        )
        .is_err()
    );
}

#[test]
fn framework_callers_can_explicitly_allow_reserved_keys() {
    let options = AttributeValidationOptions {
        existing_keys: None,
        allow_reserved_attributes: true,
    };
    assert!(
        validate_attribute_changes(&[change("$framework.kind", Some("agent"))], &options).is_ok()
    );
}

#[test]
fn applies_upserts() {
    let existing = BTreeMap::from([("phase".to_owned(), "pending".to_owned())]);
    let result = apply_attribute_changes(Some(&existing), &[change("region", Some("iad1"))]);
    assert_eq!(
        result,
        BTreeMap::from([
            ("phase".to_owned(), "pending".to_owned()),
            ("region".to_owned(), "iad1".to_owned()),
        ])
    );
}

#[test]
fn overwrites_an_existing_value() {
    let existing = BTreeMap::from([("phase".to_owned(), "pending".to_owned())]);
    let result = apply_attribute_changes(Some(&existing), &[change("phase", Some("running"))]);
    assert_eq!(
        result,
        BTreeMap::from([("phase".to_owned(), "running".to_owned())])
    );
}

#[test]
fn removes_a_value_when_the_change_is_null() {
    let existing = BTreeMap::from([
        ("phase".to_owned(), "pending".to_owned()),
        ("stale".to_owned(), "yes".to_owned()),
    ]);
    let result = apply_attribute_changes(Some(&existing), &[change("stale", None)]);
    assert_eq!(
        result,
        BTreeMap::from([("phase".to_owned(), "pending".to_owned())])
    );
}

#[test]
fn handles_mixed_set_and_unset_changes() {
    let existing = BTreeMap::from([
        ("a".to_owned(), "1".to_owned()),
        ("b".to_owned(), "2".to_owned()),
    ]);
    let result = apply_attribute_changes(
        Some(&existing),
        &[
            change("a", None),
            change("b", Some("updated")),
            change("c", Some("3")),
        ],
    );
    assert_eq!(
        result,
        BTreeMap::from([
            ("b".to_owned(), "updated".to_owned()),
            ("c".to_owned(), "3".to_owned()),
        ])
    );
}

#[test]
fn returns_a_new_map_without_mutating_the_existing_map() {
    let existing = BTreeMap::from([("a".to_owned(), "1".to_owned())]);
    let snapshot = existing.clone();
    let result = apply_attribute_changes(Some(&existing), &[change("b", Some("2"))]);
    assert_eq!(existing, snapshot);
    assert_eq!(result.get("b").map(String::as_str), Some("2"));
}

#[test]
fn starts_from_an_empty_map_when_existing_attributes_are_absent() {
    let result = apply_attribute_changes(None, &[change("x", Some("y"))]);
    assert_eq!(
        result,
        BTreeMap::from([("x".to_owned(), "y".to_owned())])
    );
}

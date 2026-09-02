use std::panic::catch_unwind;

use workflow_ai::{ErrorValue, get_error_message};

#[test]
fn to_json_named_fields_are_serialized_as_inert_data() {
    let value = ErrorValue::Object(vec![
        (
            "toJSON".to_owned(),
            ErrorValue::String("not executable".to_owned()),
        ),
        ("rewritten".to_owned(), ErrorValue::Bool(false)),
    ]);

    assert_eq!(
        get_error_message(&value),
        r#"{"toJSON":"not executable","rewritten":false}"#
    );
}

#[test]
fn every_representable_variant_normalizes_without_panicking() {
    let values = vec![
        ErrorValue::Error("error".to_owned()),
        ErrorValue::CustomError("custom".to_owned()),
        ErrorValue::String("string".to_owned()),
        ErrorValue::Object(vec![(
            "nested".to_owned(),
            ErrorValue::Array(vec![ErrorValue::Undefined, ErrorValue::Null]),
        )]),
        ErrorValue::Null,
        ErrorValue::Undefined,
        ErrorValue::Number(i64::MIN),
        ErrorValue::Number(i64::MAX),
        ErrorValue::Bool(false),
        ErrorValue::Array(vec![ErrorValue::Object(Vec::new())]),
    ];

    for value in values {
        let normalized = catch_unwind(|| get_error_message(&value));
        assert!(normalized.is_ok(), "representable values must normalize safely");
    }
}

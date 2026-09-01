use workflow_ai_tdd::{ErrorValue, get_error_message};

#[test]
fn returns_message_from_error_instances() {
    assert_eq!(
        get_error_message(&ErrorValue::Error("something broke".to_owned())),
        "something broke"
    );
}

#[test]
fn returns_string_errors_as_is() {
    assert_eq!(
        get_error_message(&ErrorValue::String("plain string error".to_owned())),
        "plain string error"
    );
}

#[test]
fn serializes_plain_objects_instead_of_object_object() {
    let value = ErrorValue::Object(vec![
        (
            "code".to_owned(),
            ErrorValue::String("STREAM_FAILED".to_owned()),
        ),
        (
            "detail".to_owned(),
            ErrorValue::String("token limit".to_owned()),
        ),
    ]);
    let message = get_error_message(&value);

    assert_ne!(message, "[object Object]");
    assert_eq!(
        message,
        r#"{"code":"STREAM_FAILED","detail":"token limit"}"#
    );
}

#[test]
fn serializes_nested_objects() {
    let value = ErrorValue::Object(vec![(
        "outer".to_owned(),
        ErrorValue::Object(vec![(
            "inner".to_owned(),
            ErrorValue::String("value".to_owned()),
        )]),
    )]);

    assert_eq!(get_error_message(&value), r#"{"outer":{"inner":"value"}}"#);
}

#[test]
fn returns_unknown_error_for_null() {
    assert_eq!(get_error_message(&ErrorValue::Null), "unknown error");
}

#[test]
fn returns_unknown_error_for_undefined() {
    assert_eq!(get_error_message(&ErrorValue::Undefined), "unknown error");
}

#[test]
fn handles_number_errors() {
    assert_eq!(get_error_message(&ErrorValue::Number(42)), "42");
}

#[test]
fn handles_boolean_errors() {
    assert_eq!(get_error_message(&ErrorValue::Bool(true)), "true");
}

#[test]
fn handles_array_errors() {
    assert_eq!(
        get_error_message(&ErrorValue::Array(vec![
            ErrorValue::String("a".to_owned()),
            ErrorValue::String("b".to_owned()),
        ])),
        r#"["a","b"]"#
    );
}

#[test]
fn handles_empty_strings() {
    assert_eq!(get_error_message(&ErrorValue::String(String::new())), "");
}

#[test]
fn handles_error_subclasses_by_their_message() {
    assert_eq!(
        get_error_message(&ErrorValue::CustomError("custom msg".to_owned())),
        "custom msg"
    );
}

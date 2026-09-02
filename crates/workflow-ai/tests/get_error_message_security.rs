use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use workflow_ai::{CallableProbe, ErrorValue, SharedErrorObject, get_error_message};

#[test]
fn object_supplied_to_json_callable_is_never_invoked() {
    let callback = CallableProbe::new("toJSON");
    let value = ErrorValue::Object(vec![
        ("toJSON".to_owned(), ErrorValue::Callable(callback.clone())),
        ("rewritten".to_owned(), ErrorValue::Bool(false)),
    ]);

    assert_eq!(
        get_error_message(&value),
        r#"{"toJSON":"[Function toJSON]","rewritten":false}"#
    );
    assert_eq!(callback.call_count(), 0);

    callback.invoke();
    assert_eq!(callback.call_count(), 1);
}

#[test]
fn cyclic_objects_normalize_without_panicking() {
    let object: Arc<SharedErrorObject> = Arc::new_cyclic(|reference| {
        SharedErrorObject::new(vec![(
            "self".to_owned(),
            ErrorValue::ObjectReference(reference.clone()),
        )])
    });
    let value = ErrorValue::SharedObject(object);

    let normalized = catch_unwind(AssertUnwindSafe(|| get_error_message(&value)));
    assert_eq!(normalized.unwrap(), r#"{"self":"[Circular]"}"#);
}

#[test]
fn bigints_normalize_without_panicking() {
    let value = ErrorValue::BigInt("1".to_owned());

    let normalized = catch_unwind(AssertUnwindSafe(|| get_error_message(&value)));
    assert_eq!(normalized.unwrap(), "1n");
}

#[test]
fn released_references_and_excessive_depth_have_stable_fallbacks() {
    let released = {
        let object = Arc::new(SharedErrorObject::new(Vec::new()));
        Arc::downgrade(&object)
    };
    assert_eq!(
        get_error_message(&ErrorValue::ObjectReference(released)),
        "[Released reference]"
    );

    let mut value = ErrorValue::String("leaf".to_owned());
    for _ in 0..128 {
        value = ErrorValue::Array(vec![value]);
    }
    let normalized = catch_unwind(AssertUnwindSafe(|| get_error_message(&value)));
    assert!(
        normalized.unwrap().contains("[Max depth exceeded]"),
        "deep graphs must terminate at the configured bound"
    );
}

#[test]
fn invalid_bigint_text_cannot_escape_into_diagnostics() {
    assert_eq!(
        get_error_message(&ErrorValue::BigInt("1\nforged".to_owned())),
        "[Invalid BigInt]"
    );
}

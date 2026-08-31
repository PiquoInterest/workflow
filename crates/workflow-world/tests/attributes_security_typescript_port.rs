use serde_json::json;
use workflow_world::attributes::{AttributeChange, apply_attribute_changes};

#[test]
fn prototype_named_attributes_are_ordinary_map_data() {
    let result = apply_attribute_changes(
        None,
        &[AttributeChange {
            key: "__proto__".to_owned(),
            value: Some("ordinary-data".to_owned()),
        }],
    );

    assert_eq!(result.get("__proto__").map(String::as_str), Some("ordinary-data"));
    assert_eq!(result.len(), 1);
}

#[test]
fn object_shaped_values_cannot_reach_attribute_materialization() {
    let value = json!({
        "key": "__proto__",
        "value": { "polluted": true },
    });

    let error = serde_json::from_value::<AttributeChange>(value).unwrap_err();
    assert!(error.to_string().contains("string or null"));

    let clean = apply_attribute_changes(None, &[]);
    assert!(clean.is_empty());
}

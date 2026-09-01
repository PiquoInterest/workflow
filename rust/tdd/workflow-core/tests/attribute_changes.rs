use workflow_core_tdd::attribute_changes::{
    ATTRIBUTE_KEY_MAX_LENGTH, ATTRIBUTE_MAX_PER_RUN, ATTRIBUTE_VALUE_MAX_BYTES, AttributeChange,
    AttributeField, AttributeInput, NormalizeAttributeOptions, normalize_attribute_changes,
};

fn record(entries: Vec<AttributeField>) -> AttributeInput {
    AttributeInput::Record(entries)
}

fn normalize(input: AttributeInput) -> Result<Vec<AttributeChange>, String> {
    normalize_attribute_changes(input, NormalizeAttributeOptions::default())
        .map_err(|error| error.message)
}

#[test]
fn converts_a_record_to_ordered_changes_and_maps_undefined_to_null() {
    assert_eq!(
        normalize(record(vec![
            AttributeField::new("phase", Some("init")),
            AttributeField::new("stale", None::<String>),
        ])),
        Ok(vec![
            AttributeChange {
                key: "phase".to_owned(),
                value: Some("init".to_owned()),
            },
            AttributeChange {
                key: "stale".to_owned(),
                value: None,
            },
        ])
    );
}

#[test]
fn returns_an_empty_vector_for_an_empty_record() {
    assert_eq!(normalize(record(Vec::new())), Ok(Vec::new()));
}

fn assert_plain_object_error(input: AttributeInput, expected_type: &str) {
    let error = normalize(input).unwrap_err();
    assert!(error.contains("setAttributes requires a plain object"));
    assert!(error.contains(expected_type));
}

#[test]
fn rejects_null_and_names_the_type() {
    assert_plain_object_error(AttributeInput::Null, "got null");
}

#[test]
fn rejects_arrays_and_names_the_type() {
    assert_plain_object_error(
        AttributeInput::Array(vec!["phase".to_owned(), "init".to_owned()]),
        "got array",
    );
}

#[test]
fn rejects_strings_and_names_the_type() {
    assert_plain_object_error(
        AttributeInput::String("phase=init".to_owned()),
        "got string",
    );
}

#[test]
fn rejects_numbers_and_names_the_type() {
    assert_plain_object_error(AttributeInput::Number(42.0), "got number");
}

#[test]
fn rejects_keys_over_the_length_cap_and_names_the_limit() {
    let error = normalize(record(vec![AttributeField::new(
        "k".repeat(ATTRIBUTE_KEY_MAX_LENGTH + 1),
        Some("v"),
    )]))
    .unwrap_err();
    assert!(error.contains(&format!("exceeds limit {ATTRIBUTE_KEY_MAX_LENGTH}")));
}

#[test]
fn counts_key_length_with_javascript_utf16_semantics() {
    let at_cap = "💥".repeat(ATTRIBUTE_KEY_MAX_LENGTH / 2);
    assert_eq!(
        normalize(record(vec![AttributeField::new(
            at_cap.clone(),
            Some("v"),
        )])),
        Ok(vec![AttributeChange {
            key: at_cap,
            value: Some("v".to_owned()),
        }])
    );

    let error = normalize(record(vec![AttributeField::new(
        "💥".repeat(ATTRIBUTE_KEY_MAX_LENGTH / 2 + 1),
        Some("v"),
    )]))
    .unwrap_err();
    assert!(error.contains(&format!(
        "Attribute key length {} exceeds limit {ATTRIBUTE_KEY_MAX_LENGTH}",
        ATTRIBUTE_KEY_MAX_LENGTH + 2
    )));
}

#[test]
fn rejects_values_over_the_utf8_byte_cap_and_names_the_byte_length() {
    let error = normalize(record(vec![AttributeField::new(
        "note",
        Some("é".repeat(200)),
    )]))
    .unwrap_err();
    assert!(error.contains(&format!("exceeds limit {ATTRIBUTE_VALUE_MAX_BYTES}")));
    assert!(error.contains("byte length 400"));
}

#[test]
fn rejects_reserved_prefixes_with_opt_in_guidance() {
    let error = normalize(record(vec![AttributeField::new("$system", Some("x"))])).unwrap_err();
    assert!(error.contains("reserved prefix"));
    assert!(error.contains("allowReservedAttributes"));
}

#[test]
fn accepts_reserved_prefixes_when_explicitly_allowed() {
    let result = normalize_attribute_changes(
        record(vec![AttributeField::new("$agent.kind", Some("durable"))]),
        NormalizeAttributeOptions {
            allow_reserved_attributes: true,
        },
    );
    assert_eq!(
        result,
        Ok(vec![AttributeChange {
            key: "$agent.kind".to_owned(),
            value: Some("durable".to_owned()),
        }])
    );
}

#[test]
fn rejects_one_batch_above_the_per_run_cap() {
    let entries = (0..=ATTRIBUTE_MAX_PER_RUN)
        .map(|index| AttributeField::new(format!("key_{index}"), Some("v")))
        .collect();
    let error = normalize(record(entries)).unwrap_err();
    assert!(error.contains(&format!("exceed limit {ATTRIBUTE_MAX_PER_RUN}")));
}

#[test]
fn accepts_a_batch_exactly_at_the_per_run_cap() {
    let entries = (0..ATTRIBUTE_MAX_PER_RUN)
        .map(|index| AttributeField::new(format!("key_{index}"), Some("v")))
        .collect();
    assert_eq!(
        normalize(record(entries)).unwrap().len(),
        ATTRIBUTE_MAX_PER_RUN
    );
}

#[test]
fn accepts_boundary_length_keys_and_values() {
    let key = "k".repeat(ATTRIBUTE_KEY_MAX_LENGTH);
    let value = "v".repeat(ATTRIBUTE_VALUE_MAX_BYTES);
    assert_eq!(
        normalize(record(vec![AttributeField::new(
            key.clone(),
            Some(value.clone()),
        )])),
        Ok(vec![AttributeChange {
            key,
            value: Some(value),
        }])
    );
}

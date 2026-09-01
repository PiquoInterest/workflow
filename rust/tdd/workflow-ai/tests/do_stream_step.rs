use workflow_ai_tdd::{
    DoStreamStepCase, FinishReasonInput, FinishReasonTypeField, ResponseMetadataObservation,
    ToolCallInput, UiMessageChunkObservation, exercise_do_stream_step, normalize_finish_reason,
    safe_parse_tool_call_input,
};

fn string_reason(value: &str) -> FinishReasonInput {
    FinishReasonInput::String(value.to_owned())
}

fn object_reason(type_field: FinishReasonTypeField) -> FinishReasonInput {
    FinishReasonInput::Object {
        type_field,
        has_additional_properties: false,
    }
}

#[test]
fn passes_through_stop_string_finish_reason() {
    assert_eq!(normalize_finish_reason(&string_reason("stop")), "stop");
}

#[test]
fn passes_through_tool_calls_string_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&string_reason("tool-calls")),
        "tool-calls"
    );
}

#[test]
fn passes_through_length_string_finish_reason() {
    assert_eq!(normalize_finish_reason(&string_reason("length")), "length");
}

#[test]
fn passes_through_content_filter_string_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&string_reason("content-filter")),
        "content-filter"
    );
}

#[test]
fn passes_through_error_string_finish_reason() {
    assert_eq!(normalize_finish_reason(&string_reason("error")), "error");
}

#[test]
fn passes_through_other_string_finish_reason() {
    assert_eq!(normalize_finish_reason(&string_reason("other")), "other");
}

#[test]
fn extracts_stop_from_object_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::String(
            "stop".to_owned()
        ))),
        "stop"
    );
}

#[test]
fn extracts_tool_calls_from_object_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::String(
            "tool-calls".to_owned()
        ))),
        "tool-calls"
    );
}

#[test]
fn extracts_length_from_object_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::String(
            "length".to_owned()
        ))),
        "length"
    );
}

#[test]
fn extracts_content_filter_from_object_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::String(
            "content-filter".to_owned()
        ))),
        "content-filter"
    );
}

#[test]
fn extracts_error_from_object_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::String(
            "error".to_owned()
        ))),
        "error"
    );
}

#[test]
fn extracts_other_from_object_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::String(
            "other".to_owned()
        ))),
        "other"
    );
}

#[test]
fn returns_other_for_object_with_unrecognized_type() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::String(
            "unknown".to_owned()
        ))),
        "other"
    );
}

#[test]
fn returns_other_for_object_without_type_property() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::Missing)),
        "other"
    );
}

#[test]
fn returns_other_for_object_with_null_type() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::Null)),
        "other"
    );
}

#[test]
fn returns_other_for_object_with_undefined_type() {
    assert_eq!(
        normalize_finish_reason(&object_reason(FinishReasonTypeField::Undefined)),
        "other"
    );
}

#[test]
fn handles_object_with_additional_properties() {
    let input = FinishReasonInput::Object {
        type_field: FinishReasonTypeField::String("stop".to_owned()),
        has_additional_properties: true,
    };
    assert_eq!(normalize_finish_reason(&input), "stop");
}

#[test]
fn returns_other_for_undefined_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&FinishReasonInput::Undefined),
        "other"
    );
}

#[test]
fn returns_other_for_null_finish_reason() {
    assert_eq!(normalize_finish_reason(&FinishReasonInput::Null), "other");
}

#[test]
fn returns_other_for_number_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&FinishReasonInput::Number(42)),
        "other"
    );
}

#[test]
fn returns_other_for_boolean_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&FinishReasonInput::Bool(true)),
        "other"
    );
}

#[test]
fn returns_other_for_array_finish_reason() {
    assert_eq!(
        normalize_finish_reason(&FinishReasonInput::Array(vec![string_reason("stop")])),
        "other"
    );
}

#[test]
fn preserves_empty_string_finish_reason() {
    assert_eq!(normalize_finish_reason(&string_reason("")), "");
}

#[test]
fn normalizes_object_format_that_previously_stringified_as_object_object() {
    let normalized: String = normalize_finish_reason(&object_reason(
        FinishReasonTypeField::String("stop".to_owned()),
    ));
    assert_eq!(normalized, "stop");
}

#[test]
fn normalizes_tool_calls_object_format_to_a_string() {
    let normalized: String = normalize_finish_reason(&object_reason(
        FinishReasonTypeField::String("tool-calls".to_owned()),
    ));
    assert_eq!(normalized, "tool-calls");
}

#[test]
fn parses_valid_json_tool_call_input() {
    assert_eq!(
        safe_parse_tool_call_input(Some(r#"{"city":"San Francisco"}"#)),
        ToolCallInput::Object(vec![(
            "city".to_owned(),
            "San Francisco".to_owned()
        )])
    );
}

#[test]
fn returns_empty_object_for_undefined_tool_call_input() {
    assert_eq!(
        safe_parse_tool_call_input(None),
        ToolCallInput::Object(Vec::new())
    );
}

#[test]
fn preserves_malformed_tool_call_input_as_a_string() {
    let malformed = r#"{"city":"San Francisco""#;
    assert_eq!(
        safe_parse_tool_call_input(Some(malformed)),
        ToolCallInput::RawString(malformed.to_owned())
    );
}

#[test]
fn streamed_malformed_tool_call_input_does_not_abort_the_step() {
    let result = exercise_do_stream_step(DoStreamStepCase::MalformedToolCallInput);
    let malformed = ToolCallInput::RawString(r#"{"city":"San Francisco""#.to_owned());

    assert_eq!(result.tool_calls.len(), 1);
    assert_eq!(result.tool_calls[0].input, malformed);
    assert!(result.written_chunks.contains(
        &UiMessageChunkObservation::ToolInputAvailable {
            tool_call_id: "call-1".to_owned(),
            tool_name: "getWeather".to_owned(),
            input: malformed,
        }
    ));
}

#[test]
fn partial_response_metadata_chunks_are_merged_instead_of_overwritten() {
    let result = exercise_do_stream_step(DoStreamStepCase::PartialResponseMetadata);

    assert_eq!(
        result.response_metadata,
        Some(ResponseMetadataObservation {
            id: Some("resp-1".to_owned()),
            model_id: Some("resolved-model".to_owned()),
            timestamp: Some("2026-06-24T00:00:00.000Z".to_owned()),
        })
    );
}

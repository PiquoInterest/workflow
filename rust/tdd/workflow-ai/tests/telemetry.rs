use workflow_ai_tdd::ModelValue;
use workflow_ai_tdd::telemetry::{
    TelemetryCase, TelemetryOptionsObservation, TelemetryValue, exercise_telemetry,
};

fn string(value: &str) -> TelemetryValue {
    TelemetryValue::String(value.to_owned())
}

fn number(value: i64) -> TelemetryValue {
    TelemetryValue::Number(value)
}

fn model_object(entries: &[(&str, ModelValue)]) -> ModelValue {
    ModelValue::Object(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect(),
    )
}

#[test]
fn records_response_time_attributes_on_the_do_stream_span() {
    let observation = exercise_telemetry(TelemetryCase::DoStreamResponseAttributes);

    assert!(
        observation
            .record_span_names
            .contains(&"ai.streamText.doStream".to_owned())
    );
    assert_eq!(
        observation.initial_attributes.get("ai.model.provider"),
        Some(&string("test-provider"))
    );
    assert_eq!(
        observation.initial_attributes.get("ai.model.id"),
        Some(&string("test-model-id"))
    );
    assert_eq!(
        observation.initial_attributes.get("gen_ai.system"),
        Some(&string("test-provider"))
    );
    assert_eq!(
        observation.initial_attributes.get("gen_ai.request.model"),
        Some(&string("test-model-id"))
    );
    assert_eq!(
        observation.initial_attributes.get("ai.prompt.messages"),
        Some(&string(
            r#"[{"role":"user","content":[{"type":"text","text":"hi"}]}]"#
        ))
    );

    assert_eq!(
        observation
            .emitted_attributes
            .get("ai.response.finishReason"),
        Some(&string("stop"))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.response.id"),
        Some(&string("resp-123"))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.response.model"),
        Some(&string("test-model-id"))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.usage.inputTokens"),
        Some(&number(10))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.usage.outputTokens"),
        Some(&number(20))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.usage.totalTokens"),
        Some(&number(30))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.response.text"),
        Some(&string("Hello world"))
    );
    assert_eq!(
        observation
            .emitted_attributes
            .get("gen_ai.response.finish_reasons"),
        Some(&TelemetryValue::StringList(vec!["stop".to_owned()]))
    );
    assert_eq!(
        observation
            .emitted_attributes
            .get("gen_ai.usage.input_tokens"),
        Some(&number(10))
    );
    assert_eq!(
        observation
            .emitted_attributes
            .get("gen_ai.usage.output_tokens"),
        Some(&number(20))
    );
    assert!(matches!(
        observation
            .emitted_attributes
            .get("ai.response.msToFirstChunk"),
        Some(TelemetryValue::Number(_))
    ));
    assert!(matches!(
        observation
            .emitted_attributes
            .get("ai.response.msToFinish"),
        Some(TelemetryValue::Number(_))
    ));
}

#[test]
fn records_tool_call_attributes_in_the_response() {
    let observation = exercise_telemetry(TelemetryCase::DoStreamToolCalls);

    assert_eq!(
        observation
            .emitted_attributes
            .get("ai.response.finishReason"),
        Some(&string("tool-calls"))
    );
    assert!(
        observation
            .emitted_attributes
            .contains_key("ai.response.toolCalls")
    );
    assert_eq!(observation.tool_call_names, vec!["getWeather"]);
}

#[test]
fn record_inputs_false_omits_prompt_attributes() {
    let observation = exercise_telemetry(TelemetryCase::DoStreamWithoutRecordedInputs);

    for key in [
        "ai.prompt.messages",
        "ai.prompt.tools",
        "ai.prompt.toolChoice",
    ] {
        assert!(!observation.initial_attributes.contains_key(key));
    }
}

#[test]
fn record_outputs_false_omits_response_payloads_but_keeps_usage() {
    let observation = exercise_telemetry(TelemetryCase::DoStreamWithoutRecordedOutputs);

    assert_eq!(
        observation.emitted_attributes.get("ai.usage.inputTokens"),
        Some(&number(1))
    );
    assert_eq!(
        observation
            .emitted_attributes
            .get("ai.response.finishReason"),
        Some(&string("stop"))
    );
    assert!(!observation.emitted_attributes.contains_key("ai.response.text"));
    assert!(
        !observation
            .emitted_attributes
            .contains_key("ai.response.toolCalls")
    );
}

#[test]
fn includes_reasoning_and_cache_token_attributes_when_present() {
    let observation = exercise_telemetry(TelemetryCase::DoStreamReasoningAndCacheTokens);

    assert_eq!(
        observation.emitted_attributes.get("ai.usage.inputTokens"),
        Some(&number(100))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.usage.outputTokens"),
        Some(&number(50))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.usage.totalTokens"),
        Some(&number(150))
    );
    assert_eq!(
        observation
            .emitted_attributes
            .get("ai.usage.reasoningTokens"),
        Some(&number(30))
    );
    assert_eq!(
        observation
            .emitted_attributes
            .get("ai.usage.cachedInputTokens"),
        Some(&number(80))
    );
    assert_eq!(
        observation.emitted_attributes.get("ai.response.reasoning"),
        Some(&string("thinking..."))
    );
}

#[test]
fn records_tool_results_on_the_tool_span() {
    let observation = exercise_telemetry(TelemetryCase::ExecuteToolResult);

    assert!(
        observation
            .record_span_names
            .contains(&"ai.toolCall".to_owned())
    );
    assert_eq!(
        observation.tool_span_attributes.get("ai.toolCall.name"),
        Some(&string("getWeather"))
    );
    assert_eq!(
        observation.tool_span_attributes.get("ai.toolCall.id"),
        Some(&string("tc-1"))
    );
    assert_eq!(
        observation.tool_span_attributes.get("ai.toolCall.args"),
        Some(&string("{}"))
    );
    assert_eq!(
        observation.tool_result,
        Some(model_object(&[
            ("type", ModelValue::String("json".to_owned())),
            (
                "value",
                model_object(&[
                    ("temperature", ModelValue::Number(72)),
                    ("unit", ModelValue::String("F".to_owned())),
                ]),
            ),
        ]))
    );
}

#[test]
fn record_outputs_false_omits_tool_arguments_and_result() {
    let observation = exercise_telemetry(TelemetryCase::ExecuteToolWithoutRecordedOutputs);

    assert!(
        observation
            .record_span_names
            .contains(&"ai.toolCall".to_owned())
    );
    assert!(
        !observation
            .tool_span_attributes
            .contains_key("ai.toolCall.args")
    );
    assert_eq!(observation.tool_result, None);
}

#[test]
fn executes_tools_inside_the_outer_stream_span_context() {
    let observation = exercise_telemetry(TelemetryCase::ExecuteToolSpanContext);

    assert!(
        observation
            .run_in_context_handles
            .contains(&"test-trace".to_owned())
    );
}

#[test]
fn forwards_outer_stream_telemetry_options_to_the_iterator() {
    let observation = exercise_telemetry(TelemetryCase::StreamTextIteratorOptions);

    assert_eq!(
        observation.iterator_telemetry,
        Some(TelemetryOptionsObservation {
            is_enabled: true,
            function_id: Some("outer-test".to_owned()),
            record_inputs: None,
            record_outputs: None,
        })
    );
}

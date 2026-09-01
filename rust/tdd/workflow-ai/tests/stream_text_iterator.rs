use workflow_ai_tdd::ModelValue;
use workflow_ai_tdd::stream_text_iterator::{
    AssistantPart, PromptMessage, StreamTextIteratorCase, StreamTextIteratorObservation,
    exercise_stream_text_iterator,
};

fn object(entries: &[(&str, ModelValue)]) -> ModelValue {
    ModelValue::Object(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect(),
    )
}

fn provider_option(provider: &str, key: &str, value: &str) -> ModelValue {
    object(&[(
        provider,
        object(&[(key, ModelValue::String(value.to_owned()))]),
    )])
}

fn assistant_parts(
    observation: &StreamTextIteratorObservation,
    prompt_index: usize,
) -> &[AssistantPart] {
    observation.captured_prompts[prompt_index]
        .iter()
        .find_map(|message| match message {
            PromptMessage::Assistant(parts) => Some(parts.as_slice()),
            PromptMessage::System(_) | PromptMessage::User(_) => None,
        })
        .expect("translated fixture must contain an assistant message")
}

fn tool_call<'a>(parts: &'a [AssistantPart], tool_name: &str) -> &'a AssistantPart {
    parts
        .iter()
        .find(|part| {
            matches!(
                part,
                AssistantPart::ToolCall {
                    tool_name: actual,
                    ..
                } if actual == tool_name
            )
        })
        .expect("translated fixture must contain the requested tool call")
}

#[test]
fn preserves_provider_metadata_as_provider_options() {
    let observation = exercise_stream_text_iterator(StreamTextIteratorCase::ProviderMetadata);

    assert!(!observation.first_done);
    assert_eq!(observation.first_tool_call_count, 1);
    let part = tool_call(assistant_parts(&observation, 0), "testTool");
    assert!(matches!(
        part,
        AssistantPart::ToolCall {
            provider_options: Some(actual),
            ..
        } if actual == &provider_option(
            "google",
            "thoughtSignature",
            "sig_abc123_test_signature"
        )
    ));
}

#[test]
fn omits_provider_options_when_metadata_is_absent() {
    let observation = exercise_stream_text_iterator(StreamTextIteratorCase::NoProviderMetadata);
    let part = tool_call(assistant_parts(&observation, 0), "testTool");

    assert!(matches!(
        part,
        AssistantPart::ToolCall {
            provider_options: None,
            ..
        }
    ));
}

#[test]
fn preserves_metadata_for_parallel_tool_calls() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::ParallelProviderMetadata);

    assert!(!observation.first_done);
    assert_eq!(observation.first_tool_call_count, 2);
    let parts = assistant_parts(&observation, 0);
    assert!(matches!(
        tool_call(parts, "weatherTool"),
        AssistantPart::ToolCall {
            provider_options: Some(actual),
            ..
        } if actual == &provider_option("google", "thoughtSignature", "sig_weather_123")
    ));
    assert!(matches!(
        tool_call(parts, "newsTool"),
        AssistantPart::ToolCall {
            provider_options: Some(actual),
            ..
        } if actual == &provider_option("google", "thoughtSignature", "sig_news_456")
    ));
}

#[test]
fn handles_mixed_tool_calls_with_and_without_metadata() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::MixedProviderMetadata);
    let parts = assistant_parts(&observation, 0);

    assert!(matches!(
        tool_call(parts, "toolWithMeta"),
        AssistantPart::ToolCall {
            provider_options: Some(actual),
            ..
        } if actual == &provider_option("vertex", "thoughtSignature", "sig_vertex_789")
    ));
    assert!(matches!(
        tool_call(parts, "toolWithoutMeta"),
        AssistantPart::ToolCall {
            provider_options: None,
            ..
        }
    ));
}

#[test]
fn preserves_openai_item_id_metadata() {
    let observation = exercise_stream_text_iterator(StreamTextIteratorCase::OpenAiItemId);
    let part = tool_call(assistant_parts(&observation, 0), "testTool");

    assert!(matches!(
        part,
        AssistantPart::ToolCall {
            provider_options: Some(actual),
            ..
        } if actual == &provider_option(
            "openai",
            "itemId",
            "fc_0402bf2d292dd7ed00697a35fb10e0819ab0098545c4d0d7f5"
        )
    ));
}

#[test]
fn preserves_all_openai_metadata_fields() {
    let observation = exercise_stream_text_iterator(StreamTextIteratorCase::OpenAiAllFields);
    let part = tool_call(assistant_parts(&observation, 0), "testTool");
    let expected = object(&[(
        "openai",
        object(&[
            (
                "itemId",
                ModelValue::String(
                    "fc_0402bf2d292dd7ed00697a35fb10e0819ab0098545c4d0d7f5".to_owned(),
                ),
            ),
            (
                "someOtherField",
                ModelValue::String("should-be-preserved".to_owned()),
            ),
        ]),
    )]);

    assert!(matches!(
        part,
        AssistantPart::ToolCall {
            provider_options: Some(actual),
            ..
        } if actual == &expected
    ));
}

#[test]
fn preserves_mixed_gemini_and_openai_metadata() {
    let observation = exercise_stream_text_iterator(StreamTextIteratorCase::MixedProviders);
    let part = tool_call(assistant_parts(&observation, 0), "testTool");
    let expected = object(&[
        (
            "google",
            object(&[(
                "thoughtSignature",
                ModelValue::String("sig_gemini_preserved".to_owned()),
            )]),
        ),
        (
            "openai",
            object(&[(
                "itemId",
                ModelValue::String("fc_should_also_be_preserved".to_owned()),
            )]),
        ),
    ]);

    assert!(matches!(
        part,
        AssistantPart::ToolCall {
            provider_options: Some(actual),
            ..
        } if actual == &expected
    ));
}

#[test]
fn places_reasoning_before_tool_calls() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::ReasoningBeforeToolCall);
    let parts = assistant_parts(&observation, 0);

    assert_eq!(parts.len(), 3);
    assert!(matches!(
        &parts[0],
        AssistantPart::Reasoning {
            text,
            provider_options: None,
        } if text == "Let me think about this..."
    ));
    assert!(matches!(
        &parts[1],
        AssistantPart::Reasoning {
            text,
            provider_options: None,
        } if text == "I should use the test tool."
    ));
    assert!(matches!(
        &parts[2],
        AssistantPart::ToolCall { tool_call_id, .. } if tool_call_id == "call-1"
    ));
}

#[test]
fn preserves_reasoning_provider_options() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::ReasoningProviderOptions);
    let parts = assistant_parts(&observation, 0);
    let expected = object(&[(
        "anthropic",
        object(&[(
            "cacheControl",
            object(&[("type", ModelValue::String("ephemeral".to_owned()))]),
        )]),
    )]);

    assert!(matches!(
        &parts[0],
        AssistantPart::Reasoning {
            text,
            provider_options: Some(actual),
        } if text == "thinking..." && actual == &expected
    ));
}

#[test]
fn omits_reasoning_parts_when_the_step_has_none() {
    let observation = exercise_stream_text_iterator(StreamTextIteratorCase::NoReasoning);
    let parts = assistant_parts(&observation, 0);

    assert_eq!(parts.len(), 1);
    assert!(matches!(&parts[0], AssistantPart::ToolCall { .. }));
}

#[test]
fn prepare_step_can_prepend_only_a_system_message() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::PrepareSystemOnly);

    assert_eq!(
        observation.captured_prompts[0],
        vec![
            PromptMessage::System("You are a helpful assistant.".to_owned()),
            PromptMessage::User("hello".to_owned()),
        ]
    );
}

#[test]
fn prepare_step_preserves_system_when_replacing_messages() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::PrepareSystemAndMessages);

    assert_eq!(
        observation.captured_prompts[0],
        vec![
            PromptMessage::System("Dynamic system prompt.".to_owned()),
            PromptMessage::User("modified message".to_owned()),
        ]
    );
}

#[test]
fn prepare_step_replaces_an_existing_system_message() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::ReplaceExistingSystem);

    assert_eq!(
        observation.captured_prompts[0],
        vec![
            PromptMessage::System("New system prompt.".to_owned()),
            PromptMessage::User("hello".to_owned()),
        ]
    );
}

#[test]
fn prepare_step_updates_the_system_message_on_subsequent_steps() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::UpdateSystemOnSubsequentStep);

    assert_eq!(observation.captured_prompts.len(), 2);
    assert_eq!(
        observation.captured_prompts[0][0],
        PromptMessage::System("System prompt v0".to_owned())
    );
    assert_eq!(
        observation.captured_prompts[1][0],
        PromptMessage::System("System prompt v1".to_owned())
    );
}

#[test]
fn preserves_malformed_tool_call_input_instead_of_throwing() {
    let observation =
        exercise_stream_text_iterator(StreamTextIteratorCase::MalformedToolCallInput);

    assert!(!observation.first_done);
    assert_eq!(observation.first_tool_call_count, 1);
    let part = tool_call(assistant_parts(&observation, 0), "testTool");
    assert!(matches!(
        part,
        AssistantPart::ToolCall {
            input: workflow_ai_tdd::ToolCallInput::RawString(input),
            ..
        } if input == r#"{"query":"test""#
    ));
}

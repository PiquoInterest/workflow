use workflow_ai_tdd::ModelValue;
use workflow_ai_tdd::durable_agent::{
    DurableAgentCase, GenerationSettingsObservation, PrepareStepObservation,
    TelemetrySettingsObservation, ToolExecutionObservation, exercise_durable_agent,
};

fn object(entries: &[(&str, ModelValue)]) -> ModelValue {
    ModelValue::Object(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect(),
    )
}

fn string(value: &str) -> ModelValue {
    ModelValue::String(value.to_owned())
}

#[test]
fn forwards_stream_prepare_step_to_the_iterator() {
    let observation = exercise_durable_agent(DurableAgentCase::PrepareStepForwarded);

    assert!(observation.prepare_step_forwarded);
}

#[test]
fn uses_constructor_prepare_step_by_default() {
    let observation = exercise_durable_agent(DurableAgentCase::PrepareStepConstructorDefault);

    assert_eq!(
        observation
            .prepare_step
            .expect("translated fixture must expose prepareStep")
            .source,
        "constructor"
    );
}

#[test]
fn stream_prepare_step_overrides_constructor_prepare_step() {
    let observation = exercise_durable_agent(DurableAgentCase::PrepareStepStreamOverride);

    assert_eq!(
        observation
            .prepare_step
            .expect("translated fixture must expose prepareStep")
            .source,
        "stream"
    );
}

#[test]
fn prepare_step_can_modify_messages() {
    let observation = exercise_durable_agent(DurableAgentCase::PrepareStepModifyMessages);

    assert!(
        observation
            .prepare_step
            .expect("translated fixture must expose prepareStep")
            .can_modify_messages
    );
}

#[test]
fn prepare_step_can_change_the_model_dynamically() {
    let observation = exercise_durable_agent(DurableAgentCase::PrepareStepDynamicModel);

    assert!(
        observation
            .prepare_step
            .expect("translated fixture must expose prepareStep")
            .can_change_model
    );
}

#[test]
fn prepare_step_receives_model_step_history_and_messages() {
    let observation = exercise_durable_agent(DurableAgentCase::PrepareStepReceivesInfo);

    assert_eq!(
        observation.prepare_step,
        Some(PrepareStepObservation {
            source: "stream".to_owned(),
            can_modify_messages: false,
            can_change_model: false,
            model_id: "mock-model-id".to_owned(),
            step_number: 0,
            steps_length: 0,
            messages_id: "initialMessages".to_owned(),
        })
    );
}

#[test]
fn passes_conversation_messages_and_call_id_to_tool_execution() {
    let observation = exercise_durable_agent(DurableAgentCase::ToolExecuteMessages);

    assert_eq!(
        observation.tool_executions,
        vec![ToolExecutionObservation {
            tool_name: "testTool".to_owned(),
            tool_call_id: "test-call-id".to_owned(),
            input: object(&[("query", string("weather"))]),
            messages_id: "conversationMessages".to_owned(),
            context: None,
        }]
    );
}

#[test]
fn passes_the_same_conversation_to_parallel_tools() {
    let observation = exercise_durable_agent(DurableAgentCase::ParallelToolMessages);

    assert_eq!(observation.tool_executions.len(), 2);
    assert_eq!(
        observation.tool_executions[0],
        ToolExecutionObservation {
            tool_name: "weatherTool".to_owned(),
            tool_call_id: "weather-call".to_owned(),
            input: object(&[("city", string("NYC"))]),
            messages_id: "conversationMessages".to_owned(),
            context: None,
        }
    );
    assert_eq!(
        observation.tool_executions[1],
        ToolExecutionObservation {
            tool_name: "newsTool".to_owned(),
            tool_call_id: "news-call".to_owned(),
            input: object(&[("topic", string("tech"))]),
            messages_id: "conversationMessages".to_owned(),
            context: None,
        }
    );
}

#[test]
fn subsequent_tool_rounds_receive_the_updated_conversation() {
    let observation = exercise_durable_agent(DurableAgentCase::SubsequentToolMessages);

    assert_eq!(
        observation.messages_per_round,
        vec!["firstRoundMessages", "secondRoundMessages"]
    );
    assert_eq!(observation.message_lengths_per_round, vec![2, 4]);
    assert!(
        observation.message_lengths_per_round[1]
            > observation.message_lengths_per_round[0]
    );
}

#[test]
fn forwards_constructor_generation_settings() {
    let observation = exercise_durable_agent(DurableAgentCase::ConstructorGenerationSettings);

    assert_eq!(
        observation.generation_settings,
        Some(GenerationSettingsObservation {
            temperature_millis: Some(700),
            max_output_tokens: Some(1_000),
            top_p_millis: Some(900),
            seed: Some(42),
        })
    );
}

#[test]
fn stream_generation_settings_override_constructor_values() {
    let observation = exercise_durable_agent(DurableAgentCase::StreamGenerationOverride);

    assert_eq!(
        observation.generation_settings,
        Some(GenerationSettingsObservation {
            temperature_millis: Some(300),
            max_output_tokens: Some(500),
            top_p_millis: None,
            seed: None,
        })
    );
}

#[test]
fn forwards_max_steps_to_the_iterator() {
    let observation = exercise_durable_agent(DurableAgentCase::MaxSteps);

    assert_eq!(observation.max_steps, Some(5));
}

#[test]
fn forwards_constructor_tool_choice() {
    let observation = exercise_durable_agent(DurableAgentCase::ConstructorToolChoice);

    assert_eq!(observation.tool_choice.as_deref(), Some("required"));
}

#[test]
fn stream_tool_choice_overrides_constructor_tool_choice() {
    let observation = exercise_durable_agent(DurableAgentCase::StreamToolChoiceOverride);

    assert_eq!(observation.tool_choice.as_deref(), Some("none"));
}

#[test]
fn active_tools_filters_the_tool_set() {
    let observation = exercise_durable_agent(DurableAgentCase::ActiveTools);

    assert_eq!(observation.active_tools, vec!["tool1", "tool3"]);
}

#[test]
fn forwards_include_raw_chunks() {
    let observation = exercise_durable_agent(DurableAgentCase::IncludeRawChunks);

    assert!(observation.include_raw_chunks);
}

#[test]
fn forwards_constructor_telemetry_settings() {
    let observation = exercise_durable_agent(DurableAgentCase::ConstructorTelemetry);

    assert_eq!(
        observation.telemetry,
        Some(TelemetrySettingsObservation {
            is_enabled: Some(true),
            function_id: Some("test-agent".to_owned()),
            metadata: Some(object(&[("version", string("1.0"))])),
        })
    );
}

#[test]
fn stream_telemetry_overrides_constructor_telemetry() {
    let observation = exercise_durable_agent(DurableAgentCase::StreamTelemetryOverride);

    assert_eq!(
        observation.telemetry,
        Some(TelemetrySettingsObservation {
            is_enabled: Some(false),
            function_id: Some("stream-id".to_owned()),
            metadata: None,
        })
    );
}

#[test]
fn collect_ui_messages_false_returns_no_ui_messages() {
    let observation = exercise_durable_agent(DurableAgentCase::CollectUiFalse);

    assert_eq!(observation.ui_messages, None);
}

#[test]
fn unset_collect_ui_messages_returns_no_ui_messages() {
    let observation = exercise_durable_agent(DurableAgentCase::CollectUiUnset);

    assert_eq!(observation.ui_messages, None);
}

#[test]
fn collect_ui_messages_true_enables_chunk_collection() {
    let observation = exercise_durable_agent(DurableAgentCase::CollectUiTrue);

    assert_eq!(observation.collect_ui_chunks, Some(true));
    assert_eq!(observation.ui_messages, Some(Vec::new()));
}

#[test]
fn ui_collection_survives_send_finish_false_and_closes_the_writer() {
    let observation = exercise_durable_agent(DurableAgentCase::CollectUiNoFinishClose);

    assert_eq!(observation.ui_messages, Some(Vec::new()));
    assert!(observation.writer_closed);
    assert!(!observation.finish_chunk_written);
}

#[test]
fn ui_collection_survives_send_finish_false_with_prevent_close() {
    let observation = exercise_durable_agent(DurableAgentCase::CollectUiNoFinishPreventClose);

    assert_eq!(observation.ui_messages, Some(Vec::new()));
}

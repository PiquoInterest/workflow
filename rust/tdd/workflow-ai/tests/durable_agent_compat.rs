use workflow_ai_tdd::ModelValue;
use workflow_ai_tdd::durable_agent_compat::{
    CompatPromptMessage, DurableAgentCompatCase, FinishEvent, ModelIdentity, OnStartEvent,
    OnStepStartEvent, PendingToolCall, StepFinishEvent, ToolCallEvent, ToolCallFinishEvent,
    UsageObservation, exercise_durable_agent_compat,
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

fn model() -> ModelIdentity {
    ModelIdentity {
        provider: "mock-provider".to_owned(),
        model_id: "mock-model-id".to_owned(),
    }
}

fn user_context() -> ModelValue {
    object(&[("userId", string("test-user"))])
}

fn tool_input(value: &str) -> ModelValue {
    object(&[("value", string(value))])
}

fn simple_usage() -> UsageObservation {
    UsageObservation {
        input_tokens: 3,
        output_tokens: 10,
        total_tokens: 13,
        cached_input_tokens: None,
        reasoning_tokens: None,
        no_cache_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        text_tokens: None,
    }
}

fn expected_instruction_prompt(systems: Vec<CompatPromptMessage>) -> Vec<CompatPromptMessage> {
    systems
        .into_iter()
        .chain([
            CompatPromptMessage::User {
                text: "Hello, world!".to_owned(),
            },
            CompatPromptMessage::Assistant {
                text: "Hello, world!".to_owned(),
            },
        ])
        .collect()
}

#[test]
fn uses_prepare_call_to_transform_provider_options() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::PrepareCall);

    assert_eq!(
        observation.provider_options,
        Some(object(&[("test", object(&[("value", string("test"))]))]))
    );
}

#[test]
fn uses_prepare_step_from_the_constructor() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::PrepareStepConstructor);

    assert_eq!(observation.prepare_step_numbers, vec![0]);
}

#[test]
fn stream_prepare_step_overrides_constructor_prepare_step() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::PrepareStepOverride);

    assert_eq!(observation.prepare_step_sources, vec!["stream"]);
}

#[test]
fn constructor_prepare_step_runs_for_each_tool_loop_step() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::PrepareStepMultiStep);

    assert_eq!(observation.prepare_step_numbers, vec![0, 1]);
}

#[test]
fn forwards_the_caller_abort_signal_to_the_model_stream() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::AbortSignal);

    assert!(observation.abort_signal_forwarded);
}

#[test]
fn converts_timeout_to_an_abort_signal() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::Timeout);

    assert!(observation.timeout_signal_created);
}

#[test]
fn passes_string_instructions_as_a_system_message() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::StringInstructions);

    assert_eq!(
        observation.prompt,
        expected_instruction_prompt(vec![CompatPromptMessage::System {
            content: "INSTRUCTIONS".to_owned(),
            provider_options: None,
        }])
    );
}

#[test]
fn passes_structured_system_message_instructions() {
    let observation =
        exercise_durable_agent_compat(DurableAgentCompatCase::SystemMessageInstructions);

    assert_eq!(
        observation.prompt,
        expected_instruction_prompt(vec![CompatPromptMessage::System {
            content: "INSTRUCTIONS".to_owned(),
            provider_options: Some(object(&[("test", object(&[("value", string("test"))]),)])),
        }])
    );
}

#[test]
fn passes_an_array_of_structured_system_messages() {
    let observation =
        exercise_durable_agent_compat(DurableAgentCompatCase::SystemMessageArrayInstructions);

    assert_eq!(
        observation.prompt,
        expected_instruction_prompt(vec![
            CompatPromptMessage::System {
                content: "INSTRUCTIONS_1".to_owned(),
                provider_options: Some(object(&[("test", object(&[("value", string("test1"))]),)])),
            },
            CompatPromptMessage::System {
                content: "INSTRUCTIONS_2".to_owned(),
                provider_options: Some(object(&[("test", object(&[("value", string("test2"))]),)])),
            },
        ])
    );
}

#[test]
fn calls_constructor_on_start() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStartConstructor);

    assert_eq!(observation.callback_order, vec!["constructor"]);
}

#[test]
fn calls_stream_on_start() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStartMethod);

    assert_eq!(observation.callback_order, vec!["method"]);
}

#[test]
fn calls_both_on_start_callbacks_in_order() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStartBoth);

    assert_eq!(observation.callback_order, vec!["constructor", "method"]);
}

#[test]
fn passes_complete_on_start_event_information() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStartEvent);

    assert_eq!(
        observation.on_start_event,
        Some(OnStartEvent {
            model: model(),
            system: Some("You are a helpful assistant".to_owned()),
            prompt: Some("Hello, world!".to_owned()),
            messages_present: false,
            temperature: Some("0.7".to_owned()),
            max_output_tokens: Some(500),
            context: Some(user_context()),
        })
    );
}

#[test]
fn calls_constructor_on_step_start() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStepStartConstructor);

    assert_eq!(observation.callback_order, vec!["constructor"]);
}

#[test]
fn calls_stream_on_step_start() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStepStartMethod);

    assert_eq!(observation.callback_order, vec!["method"]);
}

#[test]
fn calls_both_on_step_start_callbacks_in_order() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStepStartBoth);

    assert_eq!(observation.callback_order, vec!["constructor", "method"]);
}

#[test]
fn passes_complete_on_step_start_event_information() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStepStartEvent);

    assert_eq!(
        observation.on_step_start_event,
        Some(OnStepStartEvent {
            step_number: 0,
            model: model(),
            system: Some("You are a helpful assistant".to_owned()),
            messages_length: 1,
            previous_steps: 0,
            context: Some(user_context()),
        })
    );
}

#[test]
fn calls_constructor_on_step_finish() {
    let observation =
        exercise_durable_agent_compat(DurableAgentCompatCase::OnStepFinishConstructor);

    assert_eq!(observation.callback_order, vec!["constructor"]);
}

#[test]
fn calls_stream_on_step_finish() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStepFinishMethod);

    assert_eq!(observation.callback_order, vec!["method"]);
}

#[test]
fn calls_both_on_step_finish_callbacks_in_order() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStepFinishBoth);

    assert_eq!(observation.callback_order, vec!["constructor", "method"]);
}

#[test]
fn passes_the_step_result_to_on_step_finish() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnStepFinishEvent);

    assert_eq!(
        observation.step_finish_event,
        Some(StepFinishEvent {
            finish_reason: "stop".to_owned(),
            step_number: 0,
            text: "Hello, world!".to_owned(),
            input_tokens: 3,
            output_tokens: 10,
            provider_metadata: Some(object(&[(
                "testProvider",
                object(&[("testKey", string("testValue"))]),
            )])),
        })
    );
}

#[test]
fn calls_constructor_on_tool_call_start() {
    let observation =
        exercise_durable_agent_compat(DurableAgentCompatCase::ToolCallStartConstructor);

    assert_eq!(observation.callback_order, vec!["constructor"]);
}

#[test]
fn calls_stream_on_tool_call_start() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::ToolCallStartMethod);

    assert_eq!(observation.callback_order, vec!["method"]);
}

#[test]
fn calls_both_on_tool_call_start_callbacks_in_order() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::ToolCallStartBoth);

    assert_eq!(observation.callback_order, vec!["constructor", "method"]);
}

#[test]
fn passes_complete_tool_call_start_event_information() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::ToolCallStartEvent);

    assert_eq!(
        observation.tool_call_start_event,
        Some(ToolCallEvent {
            tool_name: "testTool".to_owned(),
            tool_call_id: "call-1".to_owned(),
            input: tool_input("test"),
            messages_length: 1,
        })
    );
}

#[test]
fn calls_constructor_on_tool_call_finish() {
    let observation =
        exercise_durable_agent_compat(DurableAgentCompatCase::ToolCallFinishConstructor);

    assert_eq!(observation.callback_order, vec!["constructor"]);
}

#[test]
fn calls_stream_on_tool_call_finish() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::ToolCallFinishMethod);

    assert_eq!(observation.callback_order, vec!["method"]);
}

#[test]
fn calls_both_on_tool_call_finish_callbacks_in_order() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::ToolCallFinishBoth);

    assert_eq!(observation.callback_order, vec!["constructor", "method"]);
}

#[test]
fn passes_complete_successful_tool_call_finish_information() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::ToolCallFinishEvent);
    let event = observation
        .tool_call_finish_event
        .expect("translated fixture must produce a tool-call finish event");

    assert_eq!(
        event,
        ToolCallFinishEvent {
            tool_call: ToolCallEvent {
                tool_name: "testTool".to_owned(),
                tool_call_id: "call-1".to_owned(),
                input: tool_input("hello"),
                messages_length: 1,
            },
            duration_ms: event.duration_ms,
            success: true,
            output: Some("hello-result".to_owned()),
        }
    );
}

#[test]
fn calls_constructor_on_finish() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnFinishConstructor);

    assert_eq!(observation.callback_order, vec!["constructor"]);
}

#[test]
fn calls_stream_on_finish() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnFinishMethod);

    assert_eq!(observation.callback_order, vec!["method"]);
}

#[test]
fn calls_both_on_finish_callbacks_in_order() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnFinishBoth);

    assert_eq!(observation.callback_order, vec!["constructor", "method"]);
}

#[test]
fn passes_complete_on_finish_event_information() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::OnFinishEvent);

    assert_eq!(
        observation.finish_event,
        Some(FinishEvent {
            text: "Hello, world!".to_owned(),
            finish_reason: "stop".to_owned(),
            steps_length: 1,
            total_usage: simple_usage(),
        })
    );
}

#[test]
fn exposes_finish_reason_and_total_usage_on_the_result() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::StreamResultUsage);
    let usage = observation
        .stream_result_usage
        .expect("translated fixture must expose stream result usage");

    assert_eq!(usage.input_tokens, 3);
    assert_eq!(usage.output_tokens, 10);
    assert_eq!(usage.total_tokens, 13);
}

#[test]
fn aggregates_the_full_v6_usage_shape_across_steps() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::AggregateDetailedUsage);

    assert_eq!(
        observation.detailed_usage,
        Some(UsageObservation {
            input_tokens: 30,
            output_tokens: 20,
            total_tokens: 50,
            cached_input_tokens: Some(13),
            reasoning_tokens: Some(6),
            no_cache_tokens: Some(17),
            cache_read_tokens: Some(13),
            cache_write_tokens: Some(5),
            text_tokens: Some(14),
        })
    );
}

#[test]
fn calls_per_call_integrations_for_every_lifecycle_event() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::PerCallIntegrations);

    assert_eq!(
        observation.integration_events,
        vec![
            "onStart",
            "onStepStart",
            "onToolCallStart",
            "onToolCallFinish",
            "onStepFinish",
            "onStepStart",
            "onStepFinish",
            "onFinish",
        ]
    );
}

#[test]
fn calls_globally_registered_integration_listeners() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::GlobalIntegrations);

    assert_eq!(
        observation.integration_events,
        vec!["global-onStart", "global-onStepFinish", "global-onFinish",]
    );
}

#[test]
fn calls_integrations_alongside_agent_callbacks_in_order() {
    let observation =
        exercise_durable_agent_compat(DurableAgentCompatCase::IntegrationsAlongsideCallbacks);

    assert_eq!(
        observation.integration_events,
        vec![
            "agent-onStart",
            "integration-onStart",
            "agent-onStepFinish",
            "integration-onStepFinish",
            "agent-onFinish",
            "integration-onFinish",
        ]
    );
}

#[test]
fn integration_listener_failures_do_not_break_streaming() {
    let observation =
        exercise_durable_agent_compat(DurableAgentCompatCase::IntegrationListenerFailure);

    assert!(observation.stream_completed);
}

#[test]
fn static_tool_approval_pauses_without_a_tool_result() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::StaticToolApproval);

    assert_eq!(
        observation.tool_calls,
        vec![PendingToolCall {
            tool_name: "testTool".to_owned(),
            input: tool_input("test"),
        }]
    );
    assert_eq!(observation.tool_result_count, 0);
}

#[test]
fn dynamic_tool_approval_receives_the_validated_input() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::DynamicToolApproval);

    assert_eq!(observation.approval_input, Some(tool_input("test")));
}

#[test]
fn dynamic_tool_approval_pauses_without_a_tool_result() {
    let observation = exercise_durable_agent_compat(DurableAgentCompatCase::DynamicToolApproval);

    assert_eq!(observation.tool_calls.len(), 1);
    assert_eq!(observation.tool_calls[0].tool_name, "testTool");
    assert_eq!(observation.tool_result_count, 0);
}

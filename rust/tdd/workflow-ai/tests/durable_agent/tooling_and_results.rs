use workflow_ai_tdd::ModelValue;
use workflow_ai_tdd::durable_agent::{
    ContentPart, DurableAgentCase, FinishObservation, PublicToolResultObservation,
    RepairObservation, StepObservation, ToolCallObservation, ToolExecutionObservation,
    ToolResultObservation, ToolResultOutput, UiChunkObservation, exercise_durable_agent,
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

fn tool_result(
    tool_call_id: &str,
    tool_name: &str,
    output: ToolResultOutput,
) -> ToolResultObservation {
    ToolResultObservation {
        tool_call_id: tool_call_id.to_owned(),
        tool_name: tool_name.to_owned(),
        output,
    }
}

fn tool_call(tool_call_id: &str, tool_name: &str, input: ModelValue) -> ToolCallObservation {
    ToolCallObservation {
        tool_call_id: tool_call_id.to_owned(),
        tool_name: tool_name.to_owned(),
        input,
    }
}

#[test]
fn converts_fatal_errors_to_error_text_tool_results() {
    let observation = exercise_durable_agent(DurableAgentCase::FatalError);

    assert!(observation.stream_completed);
    assert_eq!(observation.iterator_next_calls, 2);
    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "test-call-id",
            "testTool",
            ToolResultOutput::ErrorText("This is a fatal error".to_owned()),
        )]
    );
}

#[test]
fn converts_generic_errors_to_error_text_tool_results() {
    let observation = exercise_durable_agent(DurableAgentCase::GenericError);

    assert!(observation.stream_completed);
    assert_eq!(observation.iterator_next_calls, 2);
    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "test-call-id",
            "testTool",
            ToolResultOutput::ErrorText("This is a generic error".to_owned()),
        )]
    );
}

#[test]
fn wraps_normal_object_results_as_json() {
    let observation = exercise_durable_agent(DurableAgentCase::NormalObjectResult);

    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "test-call-id",
            "testTool",
            ToolResultOutput::Json(object(&[
                ("success", ModelValue::Bool(true)),
                ("data", string("test result")),
            ])),
        )]
    );
}

#[test]
fn passes_preformatted_content_outputs_through_unchanged() {
    let observation = exercise_durable_agent(DurableAgentCase::PreformattedContent);

    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "vision-call-id",
            "visionTool",
            ToolResultOutput::Content(vec![
                ContentPart::Text {
                    text: "Here is the image".to_owned(),
                },
                ContentPart::FileData {
                    data: "base64data".to_owned(),
                    media_type: "image/png".to_owned(),
                },
            ]),
        )]
    );
}

#[test]
fn passes_preformatted_text_outputs_through_unchanged() {
    let observation = exercise_durable_agent(DurableAgentCase::PreformattedText);

    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "text-call-id",
            "textTool",
            ToolResultOutput::Text("pre-formatted result".to_owned()),
        )]
    );
}

#[test]
fn provider_executed_tools_never_invoke_a_local_tool() {
    let observation = exercise_durable_agent(DurableAgentCase::ProviderExecutedSkipLocal);

    assert!(observation.tool_executions.is_empty());
    assert_eq!(observation.iterator_next_calls, 2);
    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "provider-call-id",
            "WebSearch",
            ToolResultOutput::Text("Search results for: test query".to_owned()),
        )]
    );
}

#[test]
fn combines_local_and_provider_executed_results_without_cross_execution() {
    let observation = exercise_durable_agent(DurableAgentCase::MixedProviderLocal);

    assert_eq!(observation.tool_executions.len(), 1);
    assert_eq!(observation.tool_executions[0].tool_name, "localTool");
    assert_eq!(
        observation.iterator_tool_results,
        vec![
            tool_result(
                "local-call-id",
                "localTool",
                ToolResultOutput::Json(object(&[("local", string("result"))])),
            ),
            tool_result(
                "provider-call-id",
                "WebSearch",
                ToolResultOutput::Json(object(&[(
                    "searchResults",
                    ModelValue::Array(vec![string("result1"), string("result2")]),
                )])),
            ),
        ]
    );
}

#[test]
fn maps_provider_executed_errors_to_error_text() {
    let observation = exercise_durable_agent(DurableAgentCase::ProviderExecutedError);

    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "provider-call-id",
            "WebSearch",
            ToolResultOutput::ErrorText("Search failed: Rate limit exceeded".to_owned()),
        )]
    );
}

#[test]
fn warns_and_uses_an_empty_result_when_provider_output_is_missing() {
    let observation = exercise_durable_agent(DurableAgentCase::MissingProviderResult);

    assert!(observation.warnings.iter().any(|warning| {
        warning.contains("Provider-executed tool \"WebSearch\"")
            && warning.contains("missing-result-id")
    }));
    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "missing-result-id",
            "WebSearch",
            ToolResultOutput::Text(String::new()),
        )]
    );
}

#[test]
fn client_side_tools_pause_the_loop_and_return_unresolved_calls() {
    let observation = exercise_durable_agent(DurableAgentCase::ClientSideTool);

    assert_eq!(observation.iterator_next_calls, 1);
    assert_eq!(
        observation.public_tool_calls,
        vec![tool_call(
            "ask-user-call-id",
            "askUser",
            object(&[("question", string("What is your name?"))]),
        )]
    );
    assert!(observation.public_tool_results.is_empty());
    assert_eq!(
        observation.returned_messages_id.as_deref(),
        Some("mockMessages")
    );
}

#[test]
fn mixed_client_and_server_tools_execute_only_the_server_tool() {
    let observation = exercise_durable_agent(DurableAgentCase::MixedClientServer);

    assert_eq!(observation.tool_executions.len(), 1);
    assert_eq!(observation.tool_executions[0].tool_name, "serverTool");
    assert_eq!(observation.iterator_next_calls, 1);
    assert_eq!(
        observation.written_chunks,
        vec![UiChunkObservation {
            chunk_type: "tool-output-available".to_owned(),
            tool_call_id: Some("server-call-id".to_owned()),
        }]
    );
    assert_eq!(
        observation.public_tool_calls,
        vec![
            tool_call("server-call-id", "serverTool", object(&[])),
            tool_call(
                "client-call-id",
                "clientTool",
                object(&[("prompt", string("confirm action"))]),
            ),
        ]
    );
    assert_eq!(
        observation.public_tool_results,
        vec![PublicToolResultObservation {
            tool_call_id: "server-call-id".to_owned(),
            tool_name: "serverTool".to_owned(),
            output: object(&[("data", string("from-server"))]),
        }]
    );
    let unresolved: Vec<_> = observation
        .public_tool_calls
        .iter()
        .filter(|tool_call| {
            !observation
                .public_tool_results
                .iter()
                .any(|tool_result| tool_result.tool_call_id == tool_call.tool_call_id)
        })
        .collect();
    assert_eq!(unresolved.len(), 1);
    assert_eq!(unresolved[0].tool_name, "clientTool");
}

#[test]
fn client_side_tool_pause_still_calls_on_finish() {
    let observation = exercise_durable_agent(DurableAgentCase::ClientToolOnFinish);

    assert!(observation.on_finish.is_some());
}

#[test]
fn normal_completion_exposes_no_last_step_tool_calls_or_results() {
    let observation = exercise_durable_agent(DurableAgentCase::NormalCompletionEmptyToolCalls);

    assert!(observation.public_tool_calls.is_empty());
    assert!(observation.public_tool_results.is_empty());
}

#[test]
fn forwards_the_on_error_callback_to_the_iterator() {
    let observation = exercise_durable_agent(DurableAgentCase::OnErrorForwarded);

    assert!(observation.on_error_forwarded);
}

#[test]
fn tool_execution_errors_do_not_abort_the_stream() {
    let observation = exercise_durable_agent(DurableAgentCase::ToolExecutionError);

    assert!(observation.stream_completed);
    assert_eq!(
        observation.iterator_tool_results,
        vec![tool_result(
            "test-call-id",
            "failingTool",
            ToolResultOutput::ErrorText("Tool execution failed".to_owned()),
        )]
    );
}

#[test]
fn schema_invalid_tool_input_is_returned_as_an_error_without_execution() {
    let observation = exercise_durable_agent(DurableAgentCase::InvalidToolInput);

    assert!(observation.stream_completed);
    assert!(observation.tool_executions.is_empty());
    assert_eq!(observation.iterator_tool_results.len(), 1);
    assert!(matches!(
        &observation.iterator_tool_results[0].output,
        ToolResultOutput::ErrorText(message)
            if message.contains("Invalid input for tool \"strictTool\"")
    ));
}

#[test]
fn a_corrected_retry_executes_once_after_an_invalid_attempt() {
    let observation = exercise_durable_agent(DurableAgentCase::CorrectedRetry);

    assert!(observation.stream_completed);
    assert_eq!(observation.iterator_next_calls, 3);
    assert_eq!(observation.tool_executions.len(), 1);
    assert_eq!(
        observation.repaired_execute_input,
        Some(object(&[("requiredField", string("ok"))]))
    );
    assert!(matches!(
        &observation.iterator_tool_results[0].output,
        ToolResultOutput::ErrorText(_)
    ));
    assert_eq!(
        observation.iterator_tool_results[1].output,
        ToolResultOutput::Json(object(&[("ok", ModelValue::Bool(true))]))
    );
}

#[test]
fn on_finish_receives_steps_messages_and_context() {
    let observation = exercise_durable_agent(DurableAgentCase::OnFinishEvent);

    assert_eq!(
        observation.on_finish,
        Some(FinishObservation {
            steps_length: 1,
            messages_length: 1,
            context: None,
        })
    );
}

#[test]
fn an_already_aborted_signal_calls_on_abort_with_no_steps() {
    let observation = exercise_durable_agent(DurableAgentCase::AlreadyAborted);

    assert_eq!(observation.on_abort_steps, Some(0));
}

#[test]
fn stream_context_is_forwarded_to_tool_execution() {
    let observation = exercise_durable_agent(DurableAgentCase::StreamContext);

    assert_eq!(
        observation.tool_executions,
        vec![ToolExecutionObservation {
            tool_name: "testTool".to_owned(),
            tool_call_id: "test-call-id".to_owned(),
            input: object(&[]),
            messages_id: "mockMessages".to_owned(),
            context: Some(object(&[
                ("userId", string("123")),
                ("sessionId", string("abc")),
            ])),
        }]
    );
}

#[test]
fn constructor_context_is_the_default_for_tool_execution() {
    let observation = exercise_durable_agent(DurableAgentCase::ConstructorContext);

    assert_eq!(
        observation.tool_executions[0].context,
        Some(object(&[
            ("userId", string("123")),
            ("sessionId", string("abc")),
        ]))
    );
}

#[test]
fn stream_context_overrides_constructor_context() {
    let observation = exercise_durable_agent(DurableAgentCase::StreamContextOverride);

    assert_eq!(
        observation.tool_executions[0].context,
        Some(object(&[("userId", string("override-user"))]))
    );
}

#[test]
fn returns_final_messages_and_step_results() {
    let observation = exercise_durable_agent(DurableAgentCase::ResultMessagesAndSteps);

    assert_eq!(
        observation.returned_messages_id.as_deref(),
        Some("finalMessages")
    );
    assert_eq!(
        observation.returned_steps,
        vec![StepObservation {
            text: "Hello".to_owned(),
            finish_reason: "stop".to_owned(),
            input_tokens: 10,
            output_tokens: 5,
            total_tokens: 15,
        }]
    );
}

#[test]
fn invokes_the_repair_function_with_the_failed_call_and_context() {
    let observation = exercise_durable_agent(DurableAgentCase::RepairFunction);

    assert_eq!(
        observation.repair,
        Some(RepairObservation {
            tool_call_id: "test-call-id".to_owned(),
            tool_name: "testTool".to_owned(),
            tools_present: true,
            error_present: true,
            messages_id: "mockMessages".to_owned(),
        })
    );
}

#[test]
fn patches_repaired_input_back_into_the_conversation_prompt() {
    let observation = exercise_durable_agent(DurableAgentCase::RepairPatchesPrompt);

    assert_eq!(
        observation.repaired_prompt_input,
        Some(object(&[("name", string("repaired"))]))
    );
}

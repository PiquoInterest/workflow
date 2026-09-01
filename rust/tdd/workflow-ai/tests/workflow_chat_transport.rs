use workflow_ai_tdd::workflow_chat_transport::{
    ChatEndCall, ChatSendMessageCall, FetchCall, OutputChunk, PrepareReconnectInput,
    PrepareSendMessagesInput, WorkflowChatTransportCase, exercise_workflow_chat_transport,
};

fn fetch_call(
    url: &str,
    method: Option<&str>,
    body: Option<&str>,
    headers: &[(&str, &str)],
    signal_id: Option<&str>,
) -> FetchCall {
    FetchCall {
        url: url.to_owned(),
        method: method.map(str::to_owned),
        body: body.map(str::to_owned),
        headers: headers
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect(),
        credentials: None,
        signal_id: signal_id.map(str::to_owned),
    }
}

fn chunk(chunk_type: &str, id: Option<&str>) -> OutputChunk {
    OutputChunk {
        chunk_type: chunk_type.to_owned(),
        id: id.map(str::to_owned),
    }
}

fn chunk_types(chunks: &[OutputChunk]) -> Vec<&str> {
    chunks
        .iter()
        .map(|chunk| chunk.chunk_type.as_str())
        .collect()
}

#[test]
fn constructs_with_the_default_fetch_implementation() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::DefaultFetch);

    assert!(observation.constructed);
    assert!(!observation.custom_fetch_selected);
}

#[test]
fn constructs_with_a_custom_fetch_implementation() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::CustomFetch);

    assert!(observation.constructed);
    assert!(observation.custom_fetch_selected);
}

#[test]
fn accepts_and_stores_transport_callbacks() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::CallbackStorage);

    assert!(observation.constructed);
    assert!(observation.callbacks_stored);
}

#[test]
fn defaults_max_consecutive_errors_to_three() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::DefaultMaxConsecutiveErrors,
    );

    assert_eq!(observation.max_consecutive_errors, 3);
}

#[test]
fn accepts_a_custom_max_consecutive_error_limit() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::CustomMaxConsecutiveErrors,
    );

    assert_eq!(observation.max_consecutive_errors, 5);
}

#[test]
fn applies_a_custom_send_endpoint_and_body() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::CustomSendRequest);

    assert_eq!(
        observation.prepare_send_input,
        Some(PrepareSendMessagesInput {
            id: "test-chat".to_owned(),
            message_count: 0,
            has_request_metadata: false,
            has_body: false,
            has_credentials: false,
            has_headers: false,
            api: "/api/chat".to_owned(),
            trigger: "submit-message".to_owned(),
            message_id: None,
        })
    );
    assert_eq!(
        observation.fetch_calls,
        vec![fetch_call(
            "/custom/chat",
            Some("POST"),
            Some(r#"{"custom":"body"}"#),
            &[],
            None,
        )]
    );
}

#[test]
fn surfaces_non_successful_send_responses() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::ResponseError);

    assert_eq!(
        observation.terminal_error.as_deref(),
        Some("Failed to fetch chat: 500 Internal Server Error")
    );
}

#[test]
fn applies_a_custom_reconnect_endpoint_and_headers() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::CustomReconnectRequest,
    );

    assert_eq!(
        observation.prepare_reconnect_input,
        Some(PrepareReconnectInput {
            id: "test-chat".to_owned(),
            has_request_metadata: false,
            has_body: false,
            has_credentials: false,
            has_headers: false,
            api: "/api/chat/test-chat/stream".to_owned(),
        })
    );
    assert_eq!(
        observation.fetch_calls,
        vec![fetch_call(
            "/custom/reconnect?startIndex=0",
            None,
            None,
            &[("X-Custom", "header")],
            None,
        )]
    );
}

#[test]
fn forwards_the_abort_signal_to_reconnect_fetches() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::ReconnectAbortSignal,
    );

    assert_eq!(
        observation.fetch_calls,
        vec![fetch_call(
            "/api/chat/test-chat/stream?startIndex=0",
            None,
            None,
            &[],
            Some("abort-1"),
        )]
    );
}

#[test]
fn reuses_the_abort_signal_when_send_falls_back_to_reconnect() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::SendReconnectAbortSignal,
    );

    assert_eq!(
        observation.fetch_calls,
        vec![
            fetch_call(
                "/api/chat",
                Some("POST"),
                Some(r#"{"messages":[]}"#),
                &[],
                Some("abort-1"),
            ),
            fetch_call(
                "/api/chat/test-workflow-reconnect/stream?startIndex=0",
                None,
                None,
                &[],
                Some("abort-1"),
            ),
        ]
    );
}

#[test]
fn resolves_a_negative_start_index_from_the_tail_header() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::NegativeStartIndexWithTail,
    );

    assert_eq!(observation.fetch_calls.len(), 2);
    assert_eq!(
        observation.fetch_calls[0].url,
        "/api/chat/test-chat/stream?startIndex=-20"
    );
    assert_eq!(
        observation.fetch_calls[1].url,
        "/api/chat/test-chat/stream?startIndex=480"
    );
}

#[test]
fn falls_back_to_zero_when_a_negative_resume_has_no_tail_header() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::NegativeStartIndexWithoutTail,
    );

    assert_eq!(observation.fetch_calls.len(), 2);
    assert_eq!(
        observation.fetch_calls[0].url,
        "/api/chat/test-chat/stream?startIndex=-10"
    );
    assert_eq!(
        observation.fetch_calls[1].url,
        "/api/chat/test-chat/stream?startIndex=0"
    );
    assert!(
        observation
            .warnings
            .iter()
            .any(|warning| warning.contains("Negative initialStartIndex is configured"))
    );
}

#[test]
fn falls_back_to_zero_when_the_tail_header_is_not_numeric() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::NegativeStartIndexWithInvalidTail,
    );

    assert_eq!(observation.fetch_calls.len(), 2);
    assert_eq!(
        observation.fetch_calls[1].url,
        "/api/chat/test-chat/stream?startIndex=0"
    );
    assert!(observation.warnings.iter().any(|warning| {
        warning.contains("valid \"x-workflow-stream-tail-index\"")
    }));
}

#[test]
fn drops_orphan_reasoning_chunks_and_warns_only_once() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::OrphanReasoningChunks,
    );

    assert_eq!(
        chunk_types(&observation.chunks),
        vec![
            "finish-step",
            "start-step",
            "reasoning-start",
            "reasoning-delta",
            "reasoning-end",
            "finish",
        ]
    );
    assert_eq!(observation.warnings.len(), 1);
    assert!(observation.warnings[0].contains("Dropping orphan UI chunk"));
}

#[test]
fn drops_orphan_text_chunks_and_warns_only_once() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::OrphanTextChunks);

    assert_eq!(observation.chunks, vec![chunk("finish", None)]);
    assert_eq!(observation.warnings.len(), 1);
}

#[test]
fn drops_orphan_streamed_tool_chunks_and_warns_only_once() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::OrphanToolChunks);

    assert_eq!(observation.chunks, vec![chunk("finish", None)]);
    assert_eq!(observation.warnings.len(), 1);
}

#[test]
fn passes_non_streamed_tool_calls_and_their_outputs() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::NonStreamedToolChunks,
    );

    assert_eq!(
        chunk_types(&observation.chunks),
        vec![
            "tool-input-available",
            "tool-output-available",
            "tool-input-error",
            "finish",
        ]
    );
    assert!(observation.warnings.is_empty());
}

#[test]
fn recovers_a_streamed_tool_call_at_the_full_input_chunk() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::RecoverStreamedToolCall,
    );

    assert_eq!(
        chunk_types(&observation.chunks),
        vec!["tool-input-available", "tool-output-available", "finish"]
    );
    assert_eq!(observation.warnings.len(), 1);
}

#[test]
fn passes_streamed_tool_chunks_when_the_start_is_in_the_window() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::StreamedToolCallWithStart,
    );

    assert_eq!(
        chunk_types(&observation.chunks),
        vec![
            "tool-input-start",
            "tool-input-available",
            "tool-output-available",
            "finish",
        ]
    );
}

#[test]
fn does_not_activate_the_orphan_filter_for_non_negative_start_indices() {
    let observation = exercise_workflow_chat_transport(
        WorkflowChatTransportCase::NonNegativeStartIndex,
    );

    assert_eq!(
        observation.chunks,
        vec![
            chunk("reasoning-start", Some("reasoning-0")),
            chunk("reasoning-delta", Some("reasoning-0")),
            chunk("finish", None),
        ]
    );
    assert!(observation.warnings.is_empty());
}

#[test]
fn formats_reconnect_object_errors_without_object_object() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::ReconnectObjectError);
    let error = observation
        .terminal_error
        .expect("translated fixture must produce a terminal reconnect error");

    assert!(error.contains("Failed to reconnect after 1 consecutive errors"));
    assert!(!error.contains("[object Object]"));
}

#[test]
fn invokes_the_send_callback_with_the_response_and_options() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::OnChatSendMessage);

    assert_eq!(
        observation.on_chat_send_message_calls,
        vec![ChatSendMessageCall {
            chat_id: "test-chat".to_owned(),
            trigger: "submit-message".to_owned(),
            message_count: 0,
            workflow_run_id: Some("test-workflow-456".to_owned()),
            request_id: Some("123".to_owned()),
        }]
    );
}

#[test]
fn invokes_the_end_callback_when_the_stream_finishes() {
    let observation =
        exercise_workflow_chat_transport(WorkflowChatTransportCase::OnChatEnd);

    assert_eq!(observation.on_chat_end_calls.len(), 1);
    assert_eq!(
        observation.on_chat_end_calls[0],
        ChatEndCall {
            chat_id: "test-chat".to_owned(),
            chunk_index: observation.on_chat_end_calls[0].chunk_index,
        }
    );
}

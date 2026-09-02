#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowChatTransportCase {
    DefaultFetch,
    CustomFetch,
    CallbackStorage,
    DefaultMaxConsecutiveErrors,
    CustomMaxConsecutiveErrors,
    CustomSendRequest,
    ResponseError,
    CustomReconnectRequest,
    ReconnectAbortSignal,
    SendReconnectAbortSignal,
    NegativeStartIndexWithTail,
    NegativeStartIndexWithoutTail,
    NegativeStartIndexWithInvalidTail,
    OrphanReasoningChunks,
    OrphanTextChunks,
    OrphanToolChunks,
    NonStreamedToolChunks,
    RecoverStreamedToolCall,
    StreamedToolCallWithStart,
    NonNegativeStartIndex,
    ReconnectObjectError,
    OnChatSendMessage,
    OnChatEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchCall {
    pub url: String,
    pub method: Option<String>,
    pub body: Option<String>,
    pub headers: Vec<(String, String)>,
    pub credentials: Option<String>,
    pub signal_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareSendMessagesInput {
    pub id: String,
    pub message_count: usize,
    pub has_request_metadata: bool,
    pub has_body: bool,
    pub has_credentials: bool,
    pub has_headers: bool,
    pub api: String,
    pub trigger: String,
    pub message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareReconnectInput {
    pub id: String,
    pub has_request_metadata: bool,
    pub has_body: bool,
    pub has_credentials: bool,
    pub has_headers: bool,
    pub api: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputChunk {
    pub chunk_type: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSendMessageCall {
    pub chat_id: String,
    pub trigger: String,
    pub message_count: usize,
    pub workflow_run_id: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatEndCall {
    pub chat_id: String,
    pub chunk_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowChatTransportObservation {
    pub constructed: bool,
    pub custom_fetch_selected: bool,
    pub callbacks_stored: bool,
    pub max_consecutive_errors: usize,
    pub prepare_send_input: Option<PrepareSendMessagesInput>,
    pub prepare_reconnect_input: Option<PrepareReconnectInput>,
    pub fetch_calls: Vec<FetchCall>,
    pub chunks: Vec<OutputChunk>,
    pub warnings: Vec<String>,
    pub terminal_error: Option<String>,
    pub on_chat_send_message_calls: Vec<ChatSendMessageCall>,
    pub on_chat_end_calls: Vec<ChatEndCall>,
}

pub fn exercise_workflow_chat_transport(
    case: WorkflowChatTransportCase,
) -> WorkflowChatTransportObservation {
    let _ = case;
    panic!("TDD RED: packages/ai/src/workflow-chat-transport.test.ts implementation pending")
}

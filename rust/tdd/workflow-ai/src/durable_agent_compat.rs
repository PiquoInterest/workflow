use crate::ModelValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableAgentCompatCase {
    PrepareCall,
    PrepareStepConstructor,
    PrepareStepOverride,
    PrepareStepMultiStep,
    AbortSignal,
    Timeout,
    StringInstructions,
    SystemMessageInstructions,
    SystemMessageArrayInstructions,
    OnStartConstructor,
    OnStartMethod,
    OnStartBoth,
    OnStartEvent,
    OnStepStartConstructor,
    OnStepStartMethod,
    OnStepStartBoth,
    OnStepStartEvent,
    OnStepFinishConstructor,
    OnStepFinishMethod,
    OnStepFinishBoth,
    OnStepFinishEvent,
    ToolCallStartConstructor,
    ToolCallStartMethod,
    ToolCallStartBoth,
    ToolCallStartEvent,
    ToolCallFinishConstructor,
    ToolCallFinishMethod,
    ToolCallFinishBoth,
    ToolCallFinishEvent,
    OnFinishConstructor,
    OnFinishMethod,
    OnFinishBoth,
    OnFinishEvent,
    StreamResultUsage,
    AggregateDetailedUsage,
    PerCallIntegrations,
    GlobalIntegrations,
    IntegrationsAlongsideCallbacks,
    IntegrationListenerFailure,
    StaticToolApproval,
    DynamicToolApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelIdentity {
    pub provider: String,
    pub model_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatPromptMessage {
    System {
        content: String,
        provider_options: Option<ModelValue>,
    },
    User {
        text: String,
    },
    Assistant {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnStartEvent {
    pub model: ModelIdentity,
    pub system: Option<String>,
    pub prompt: Option<String>,
    pub messages_present: bool,
    pub temperature: Option<String>,
    pub max_output_tokens: Option<u64>,
    pub context: Option<ModelValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnStepStartEvent {
    pub step_number: usize,
    pub model: ModelIdentity,
    pub system: Option<String>,
    pub messages_length: usize,
    pub previous_steps: usize,
    pub context: Option<ModelValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepFinishEvent {
    pub finish_reason: String,
    pub step_number: usize,
    pub text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub provider_metadata: Option<ModelValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallEvent {
    pub tool_name: String,
    pub tool_call_id: String,
    pub input: ModelValue,
    pub messages_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallFinishEvent {
    pub tool_call: ToolCallEvent,
    pub duration_ms: u64,
    pub success: bool,
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageObservation {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub cached_input_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub no_cache_tokens: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub text_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishEvent {
    pub text: String,
    pub finish_reason: String,
    pub steps_length: usize,
    pub total_usage: UsageObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingToolCall {
    pub tool_name: String,
    pub input: ModelValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DurableAgentCompatObservation {
    pub provider_options: Option<ModelValue>,
    pub prepare_step_numbers: Vec<usize>,
    pub prepare_step_sources: Vec<String>,
    pub abort_signal_forwarded: bool,
    pub timeout_signal_created: bool,
    pub prompt: Vec<CompatPromptMessage>,
    pub callback_order: Vec<String>,
    pub on_start_event: Option<OnStartEvent>,
    pub on_step_start_event: Option<OnStepStartEvent>,
    pub step_finish_event: Option<StepFinishEvent>,
    pub tool_call_start_event: Option<ToolCallEvent>,
    pub tool_call_finish_event: Option<ToolCallFinishEvent>,
    pub finish_event: Option<FinishEvent>,
    pub stream_result_usage: Option<UsageObservation>,
    pub detailed_usage: Option<UsageObservation>,
    pub integration_events: Vec<String>,
    pub stream_completed: bool,
    pub tool_calls: Vec<PendingToolCall>,
    pub tool_result_count: usize,
    pub approval_input: Option<ModelValue>,
}

pub fn exercise_durable_agent_compat(
    case: DurableAgentCompatCase,
) -> DurableAgentCompatObservation {
    let _ = case;
    panic!(
        "TDD RED: packages/ai/src/agent/durable-agent-compat.test.ts implementation pending"
    )
}

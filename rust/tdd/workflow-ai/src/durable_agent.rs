use crate::ModelValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableAgentCase {
    FatalError,
    GenericError,
    NormalObjectResult,
    PreformattedContent,
    PreformattedText,
    ProviderExecutedSkipLocal,
    MixedProviderLocal,
    ProviderExecutedError,
    MissingProviderResult,
    ClientSideTool,
    MixedClientServer,
    ClientToolOnFinish,
    NormalCompletionEmptyToolCalls,
    PrepareStepForwarded,
    PrepareStepConstructorDefault,
    PrepareStepStreamOverride,
    PrepareStepModifyMessages,
    PrepareStepDynamicModel,
    PrepareStepReceivesInfo,
    ToolExecuteMessages,
    ParallelToolMessages,
    SubsequentToolMessages,
    ConstructorGenerationSettings,
    StreamGenerationOverride,
    MaxSteps,
    ConstructorToolChoice,
    StreamToolChoiceOverride,
    ActiveTools,
    OnErrorForwarded,
    ToolExecutionError,
    InvalidToolInput,
    CorrectedRetry,
    OnFinishEvent,
    AlreadyAborted,
    StreamContext,
    ConstructorContext,
    StreamContextOverride,
    ResultMessagesAndSteps,
    RepairFunction,
    RepairPatchesPrompt,
    IncludeRawChunks,
    ConstructorTelemetry,
    StreamTelemetryOverride,
    CollectUiFalse,
    CollectUiUnset,
    CollectUiTrue,
    CollectUiNoFinishClose,
    CollectUiNoFinishPreventClose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentPart {
    Text { text: String },
    FileData { data: String, media_type: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolResultOutput {
    Text(String),
    ErrorText(String),
    Json(ModelValue),
    Content(Vec<ContentPart>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResultObservation {
    pub tool_call_id: String,
    pub tool_name: String,
    pub output: ToolResultOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicToolResultObservation {
    pub tool_call_id: String,
    pub tool_name: String,
    pub output: ModelValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallObservation {
    pub tool_call_id: String,
    pub tool_name: String,
    pub input: ModelValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionObservation {
    pub tool_name: String,
    pub tool_call_id: String,
    pub input: ModelValue,
    pub messages_id: String,
    pub context: Option<ModelValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiChunkObservation {
    pub chunk_type: String,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareStepObservation {
    pub source: String,
    pub can_modify_messages: bool,
    pub can_change_model: bool,
    pub model_id: String,
    pub step_number: usize,
    pub steps_length: usize,
    pub messages_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationSettingsObservation {
    pub temperature_millis: Option<u32>,
    pub max_output_tokens: Option<u64>,
    pub top_p_millis: Option<u32>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishObservation {
    pub steps_length: usize,
    pub messages_length: usize,
    pub context: Option<ModelValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairObservation {
    pub tool_call_id: String,
    pub tool_name: String,
    pub tools_present: bool,
    pub error_present: bool,
    pub messages_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetrySettingsObservation {
    pub is_enabled: Option<bool>,
    pub function_id: Option<String>,
    pub metadata: Option<ModelValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepObservation {
    pub text: String,
    pub finish_reason: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DurableAgentObservation {
    pub stream_completed: bool,
    pub iterator_next_calls: usize,
    pub iterator_tool_results: Vec<ToolResultObservation>,
    pub public_tool_calls: Vec<ToolCallObservation>,
    pub public_tool_results: Vec<PublicToolResultObservation>,
    pub tool_executions: Vec<ToolExecutionObservation>,
    pub warnings: Vec<String>,
    pub returned_messages_id: Option<String>,
    pub returned_steps: Vec<StepObservation>,
    pub written_chunks: Vec<UiChunkObservation>,
    pub on_finish: Option<FinishObservation>,
    pub on_abort_steps: Option<usize>,
    pub prepare_step_forwarded: bool,
    pub prepare_step: Option<PrepareStepObservation>,
    pub messages_per_round: Vec<String>,
    pub generation_settings: Option<GenerationSettingsObservation>,
    pub max_steps: Option<usize>,
    pub tool_choice: Option<String>,
    pub active_tools: Vec<String>,
    pub on_error_forwarded: bool,
    pub repaired_execute_input: Option<ModelValue>,
    pub repair: Option<RepairObservation>,
    pub repaired_prompt_input: Option<ModelValue>,
    pub include_raw_chunks: bool,
    pub telemetry: Option<TelemetrySettingsObservation>,
    pub collect_ui_chunks: Option<bool>,
    pub ui_messages: Option<Vec<String>>,
    pub writer_closed: bool,
    pub finish_chunk_written: bool,
}

pub fn exercise_durable_agent(case: DurableAgentCase) -> DurableAgentObservation {
    let _ = case;
    panic!("TDD RED: packages/ai/src/agent/durable-agent.test.ts implementation pending")
}

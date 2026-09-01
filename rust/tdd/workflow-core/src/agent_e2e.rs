fn pending<T>() -> T {
    panic!("TDD RED: packages/core/e2e/e2e-agent.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentScenario {
    BasicText { prompt: String },
    SingleToolCall { left: i64, right: i64 },
    MultipleSequentialTools,
    ToolErrorRecovery,
    OnStepFinish,
    OnFinish,
    ProviderTool,
    MixedProviderAndFunctionTools { left: i64, right: i64 },
    StringInstructions,
    Timeout,
    ExperimentalOnStartGap,
    ExperimentalOnStepStartGap,
    ExperimentalOnToolCallStartGap,
    ExperimentalOnToolCallFinishGap,
    PrepareCallGap,
    ConstructorPrepareStep,
    StreamPrepareStepOverride,
    MultimodalToolResult,
    ToolApprovalLegacyGap,
    ToolApprovalSecureTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedStepResult {
    pub text: String,
    pub finish_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedFinishEvent {
    pub text: String,
    pub finish_reason: String,
    pub steps_length: usize,
    pub has_messages: bool,
    pub has_total_usage: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolApprovalObservation {
    pub pending: bool,
    pub tool_calls_count: usize,
    pub tool_results_count: usize,
    pub first_tool_call_name: Option<String>,
    pub tool_executed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentObservation {
    pub step_count: usize,
    pub last_step_text: Option<String>,
    pub call_sources: Vec<String>,
    pub captured_step_result: Option<CapturedStepResult>,
    pub captured_finish_event: Option<CapturedFinishEvent>,
    pub calls: Vec<String>,
    pub prepare_step_call_count: usize,
    pub prepare_step_numbers: Vec<usize>,
    pub prepare_step_sources: Vec<String>,
    pub approval: Option<ToolApprovalObservation>,
}

/// Executes one DurableAgent scenario through the future Rust runtime.
pub fn run_agent_scenario(scenario: AgentScenario) -> AgentObservation {
    let _ = scenario;
    pending()
}

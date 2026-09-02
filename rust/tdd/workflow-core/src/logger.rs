fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/logger.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoggerScenario {
    ErrorWithUnknownFields,
    Warning,
    QuietInfoAndDebug,
    BuildDebugNamespace,
    ChildMetadataMerge,
    CallSiteOverride,
    ChainedChild,
    ForRunWithWorkflowName,
    ForRunWithoutWorkflowName,
    ForRunWithExtraMetadata,
    NoMetadata,
    StepFailureSnapshot,
    MaxRetriesSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsoleCall {
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoggerObservation {
    pub error_calls: Vec<ConsoleCall>,
    pub warning_calls: Vec<ConsoleCall>,
    pub debug_calls: Vec<ConsoleCall>,
}

/// Executes one logger fixture and captures all console calls.
pub fn observe_logger_scenario(scenario: LoggerScenario) -> LoggerObservation {
    let _ = scenario;
    pending()
}

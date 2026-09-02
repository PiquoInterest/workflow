fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/classify-error.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunErrorCode {
    CorruptedEventLog,
    MaxEventsExceeded,
    ReplayDivergence,
    RuntimeError,
    DeploymentMismatch,
    UserError,
    WorldContractError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorValue {
    CorruptedEventLog,
    MaxEventsExceeded,
    ReplayDivergence,
    WorkflowRuntime,
    WorkflowNotRegistered,
    DeploymentMismatch,
    PlainError,
    TypeError,
    WorkflowWorld {
        status: Option<u16>,
        code: Option<String>,
    },
    StringThrow,
    NullThrow,
    UndefinedThrow,
    HookConflict,
    RuntimeDecryption,
    NamedError(String),
    Throttle,
    TooEarly,
}

impl ErrorValue {
    pub fn world(status: Option<u16>, code: Option<&str>) -> Self {
        Self::WorkflowWorld {
            status,
            code: code.map(str::to_owned),
        }
    }
}

/// Maps a runtime failure to the stable terminal run error category.
pub fn classify_run_error(error: &ErrorValue) -> RunErrorCode {
    let _ = error;
    pending()
}

/// Whether queue redelivery can change the outcome of this World failure.
pub fn is_retryable_world_error(error: &ErrorValue) -> bool {
    let _ = error;
    pending()
}

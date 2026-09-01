use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub const MAX_STEP_ATTEMPT: u64 = 9_007_199_254_740_991;
pub const STEP_ATTEMPT_ADVANCE_ERROR: &str =
    "prior step attempt count must be a non-negative safe integer below Number.MAX_SAFE_INTEGER";
const TDD_MARKER: &str =
    "TDD RED: packages/core/src/runtime/count-step-started-events.test.ts implementation pending";

fn pending<T>() -> T {
    panic!("{TDD_MARKER}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepAttemptError {
    pub message: String,
}

impl Display for StepAttemptError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for StepAttemptError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepEventType {
    Started,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepEvent {
    pub event_type: StepEventType,
    pub correlation_id: String,
    pub owner_message_id: Option<String>,
}

impl StepEvent {
    pub fn started(step_id: &str, owner_message_id: Option<&str>) -> Self {
        Self {
            event_type: StepEventType::Started,
            correlation_id: step_id.to_owned(),
            owner_message_id: owner_message_id.map(str::to_owned),
        }
    }

    pub fn completed(step_id: &str) -> Self {
        Self {
            event_type: StepEventType::Completed,
            correlation_id: step_id.to_owned(),
            owner_message_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStartScope {
    Unscoped,
    OwnedBy(String),
    TotalAttempts,
}

/// Advances an exact JavaScript-safe prior count to the executor's 1-based attempt.
pub fn next_step_attempt(prior_attempts: f64) -> Result<u64, StepAttemptError> {
    let _ = prior_attempts;
    pending()
}

/// Counts already-recorded starts under the selected retry-ceiling ownership scope.
pub fn count_step_started_events(
    events: Option<&[StepEvent]>,
    step_id: &str,
    scope: StepStartScope,
) -> u64 {
    let _ = (events, step_id, scope);
    pending()
}

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// JavaScript's largest exactly representable integer.
///
/// A prior count must remain below this value because the executor immediately
/// advances it to a 1-based attempt number.
pub const MAX_STEP_ATTEMPT: u64 = 9_007_199_254_740_991;

pub const STEP_ATTEMPT_ADVANCE_ERROR: &str =
    "prior step attempt count must be a non-negative safe integer below Number.MAX_SAFE_INTEGER";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepAttemptError {
    pub message: String,
}

impl StepAttemptError {
    fn invalid_count() -> Self {
        Self {
            message: STEP_ATTEMPT_ADVANCE_ERROR.to_owned(),
        }
    }
}

impl Display for StepAttemptError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for StepAttemptError {}

/// Exact count of starts already persisted for a step.
///
/// The private field prevents callers from constructing a value that cannot be
/// advanced without crossing JavaScript's exact-integer boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StepAttemptCount(u64);

impl StepAttemptCount {
    pub fn try_from_recorded(value: u64) -> Result<Self, StepAttemptError> {
        if value >= MAX_STEP_ATTEMPT {
            return Err(StepAttemptError::invalid_count());
        }
        Ok(Self(value))
    }

    /// Convert a JavaScript number at an FFI or differential-test boundary.
    ///
    /// Internal Rust retry accounting must use `try_from_recorded` and retain
    /// integer types throughout.
    pub fn try_from_js_number(value: f64) -> Result<Self, StepAttemptError> {
        if !value.is_finite()
            || value < 0.0
            || value.fract() != 0.0
            || value >= MAX_STEP_ATTEMPT as f64
        {
            return Err(StepAttemptError::invalid_count());
        }

        let integer = value as u64;
        if integer as f64 != value {
            return Err(StepAttemptError::invalid_count());
        }

        Self::try_from_recorded(integer)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn next(self) -> u64 {
        self.0 + 1
    }
}

/// JavaScript-facing compatibility transition used by conformance tests and
/// future language bindings.
pub fn next_step_attempt_from_js_number(prior_attempts: f64) -> Result<u64, StepAttemptError> {
    StepAttemptCount::try_from_js_number(prior_attempts).map(StepAttemptCount::next)
}

/// Internal executor transition with no floating-point conversion.
pub const fn next_step_attempt(prior_attempts: StepAttemptCount) -> u64 {
    prior_attempts.next()
}

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

fn checked_increment(value: &mut u64) -> Result<(), StepAttemptError> {
    *value = value
        .checked_add(1)
        .ok_or_else(StepAttemptError::invalid_count)?;
    Ok(())
}

fn checked_add(left: u64, right: u64) -> Result<u64, StepAttemptError> {
    left.checked_add(right)
        .ok_or_else(StepAttemptError::invalid_count)
}

/// Count starts already persisted for a step under one retry-ceiling scope.
///
/// `TotalAttempts` counts every bare/background start plus the largest count
/// stamped by one owner message. A valid lifecycle has one owning message
/// phase; racing invocations may contribute one-off owner IDs, but those must
/// not accumulate into false retries.
pub fn count_step_started_events(
    events: Option<&[StepEvent]>,
    step_id: &str,
    scope: StepStartScope,
) -> Result<StepAttemptCount, StepAttemptError> {
    let Some(events) = events else {
        return StepAttemptCount::try_from_recorded(0);
    };

    let mut bare = 0_u64;
    let mut by_owner = BTreeMap::<&str, u64>::new();

    for event in events {
        if event.event_type != StepEventType::Started || event.correlation_id != step_id {
            continue;
        }

        if let Some(owner) = event.owner_message_id.as_deref() {
            checked_increment(by_owner.entry(owner).or_default())?;
        } else {
            checked_increment(&mut bare)?;
        }
    }

    let count = match scope {
        StepStartScope::Unscoped => by_owner
            .values()
            .try_fold(bare, |total, owner_count| checked_add(total, *owner_count))?,
        StepStartScope::OwnedBy(message_id) => {
            by_owner.get(message_id.as_str()).copied().unwrap_or(0)
        }
        StepStartScope::TotalAttempts => {
            let max_owner = by_owner.values().copied().max().unwrap_or(0);
            checked_add(bare, max_owner)?
        }
    };

    StepAttemptCount::try_from_recorded(count)
}

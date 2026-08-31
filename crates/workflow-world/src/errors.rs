use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Value, json};

use crate::time::{DurationInput, parse_duration_to_unix_ms};
use crate::{ValidationError, ValidationResult};

/// Base URL used by the TypeScript error package for diagnostic documentation.
pub const ERROR_BASE_URL: &str = "https://workflow-sdk.dev/err";

/// Stable error-documentation slugs.
pub mod slugs {
    pub const NODE_JS_MODULE_IN_WORKFLOW: &str = "node-js-module-in-workflow";
    pub const START_INVALID_WORKFLOW_FUNCTION: &str = "start-invalid-workflow-function";
    pub const SERIALIZATION_FAILED: &str = "serialization-failed";
    pub const WEBHOOK_INVALID_RESPOND_WITH_VALUE: &str = "webhook-invalid-respond-with-value";
    pub const WEBHOOK_RESPONSE_NOT_SENT: &str = "webhook-response-not-sent";
    pub const FETCH_IN_WORKFLOW_FUNCTION: &str = "fetch-in-workflow";
    pub const TIMEOUT_FUNCTIONS_IN_WORKFLOW: &str = "timeout-in-workflow";
    pub const HOOK_CONFLICT: &str = "hook-conflict";
    pub const CORRUPTED_EVENT_LOG: &str = "corrupted-event-log";
    pub const REPLAY_DIVERGENCE: &str = "replay-divergence";
    pub const STEP_NOT_REGISTERED: &str = "step-not-registered";
    pub const WORKFLOW_NOT_REGISTERED: &str = "workflow-not-registered";
    pub const RUNTIME_DECRYPTION_FAILED: &str = "runtime-decryption-failed";
    pub const DEPLOYMENT_MISMATCH: &str = "deployment-mismatch";
}

/// Stable run-failure classification codes.
pub mod run_error_codes {
    pub const USER_ERROR: &str = "USER_ERROR";
    pub const RUNTIME_ERROR: &str = "RUNTIME_ERROR";
    pub const CORRUPTED_EVENT_LOG: &str = "CORRUPTED_EVENT_LOG";
    pub const REPLAY_DIVERGENCE: &str = "REPLAY_DIVERGENCE";
    pub const MAX_DELIVERIES_EXCEEDED: &str = "MAX_DELIVERIES_EXCEEDED";
    pub const MAX_EVENTS_EXCEEDED: &str = "MAX_EVENTS_EXCEEDED";
    pub const REPLAY_TIMEOUT: &str = "REPLAY_TIMEOUT";
    pub const WORLD_CONTRACT_ERROR: &str = "WORLD_CONTRACT_ERROR";
    pub const DEPLOYMENT_MISMATCH: &str = "DEPLOYMENT_MISMATCH";
}

/// Language-neutral representation of one public Workflow error.
///
/// JavaScript prototype identity and stack traces are runtime-specific. The
/// migration boundary therefore compares the stable public name, message,
/// retry classification, and enumerable structured fields.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowErrorDescriptor {
    pub name: String,
    pub message: String,
    pub fatal: bool,
    pub fields: BTreeMap<String, Value>,
}

impl WorkflowErrorDescriptor {
    fn new(name: &str, message: impl Into<String>) -> Self {
        Self {
            name: name.to_owned(),
            message: message.into(),
            fatal: false,
            fields: BTreeMap::new(),
        }
    }

    fn with_field(mut self, key: &str, value: Value) -> Self {
        self.fields.insert(key.to_owned(), value);
        self
    }

    fn with_optional_field(mut self, key: &str, value: Option<Value>) -> Self {
        if let Some(value) = value {
            self.fields.insert(key.to_owned(), value);
        }
        self
    }

    fn fatal(mut self) -> Self {
        self.fatal = true;
        self
    }
}

/// Composes the same plain-text diagnostic tree as the TypeScript package.
pub fn append_framed_details(title: &str, hint: Option<&str>, slug: Option<&str>) -> String {
    let mut details = Vec::new();
    if let Some(hint) = hint.filter(|value| !value.is_empty()) {
        details.push(("hint", hint.to_owned()));
    }
    if let Some(slug) = slug.filter(|value| !value.is_empty()) {
        details.push(("docs", format!("{ERROR_BASE_URL}/{slug}")));
    }
    if details.is_empty() {
        return title.to_owned();
    }

    let mut lines = vec![title.to_owned()];
    let last = details.len() - 1;
    for (index, (label, value)) in details.into_iter().enumerate() {
        let head = if index == last { "╰▶ " } else { "├▶ " };
        let continuation = if index == last { "   " } else { "│  " };
        let text = format!("{label}: {value}");
        for (line_index, line) in text.split('\n').enumerate() {
            let prefix = if line_index == 0 { head } else { continuation };
            lines.push(format!("{prefix}{line}"));
        }
    }
    lines.join("\n")
}

pub fn workflow_error(message: &str, slug: Option<&str>) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "WorkflowError",
        append_framed_details(message, None, slug),
    )
}

pub fn workflow_world_error(
    message: &str,
    status: Option<Value>,
    code: Option<Value>,
    url: Option<Value>,
    retry_after: Option<Value>,
) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("WorkflowWorldError", message)
        .with_optional_field("status", status)
        .with_optional_field("code", code)
        .with_optional_field("url", url)
        .with_optional_field("retryAfter", retry_after)
}

pub fn workflow_run_failed_error(
    run_id: &str,
    error: &Value,
    error_code: Option<&str>,
) -> WorkflowErrorDescriptor {
    let failure_message = thrown_value_message(error);
    WorkflowErrorDescriptor::new(
        "WorkflowRunFailedError",
        format!("Workflow run \"{run_id}\" failed: {failure_message}"),
    )
    .with_field("runId", json!(run_id))
    .with_optional_field("errorCode", error_code.map(|value| json!(value)))
}

pub fn workflow_run_not_completed_error(run_id: &str, status: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "WorkflowRunNotCompletedError",
        format!("Workflow run \"{run_id}\" has not completed"),
    )
    .with_field("runId", json!(run_id))
    .with_field("status", json!(status))
}

pub fn workflow_runtime_error(message: &str, slug: Option<&str>) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "WorkflowRuntimeError",
        append_framed_details(message, None, slug),
    )
}

pub fn corrupted_event_log_error(message: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "CorruptedEventLogError",
        append_framed_details(message, None, Some(slugs::CORRUPTED_EVENT_LOG)),
    )
}

pub fn replay_divergence_error(message: &str, event_id: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "ReplayDivergenceError",
        append_framed_details(message, None, Some(slugs::REPLAY_DIVERGENCE)),
    )
    .with_field("eventId", json!(event_id))
}

pub fn max_events_exceeded_error(event_count: Value, limit: Value) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "MaxEventsExceededError",
        format!(
            "Workflow exceeded the maximum of {} events per run",
            js_number_display(&limit)
        ),
    )
    .with_field("eventCount", event_count)
    .with_field("limit", limit)
}

pub fn runtime_decryption_error(message: &str, context: Option<Value>) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "RuntimeDecryptionError",
        append_framed_details(message, None, Some(slugs::RUNTIME_DECRYPTION_FAILED)),
    )
    .with_optional_field("context", context)
}

pub fn workflow_build_error(message: &str, hint: Option<&str>) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "WorkflowBuildError",
        append_framed_details(message, hint, None),
    )
    .with_optional_field("hint", hint.map(|value| json!(value)))
}

pub fn serialization_error(message: &str, hint: Option<&str>) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "SerializationError",
        append_framed_details(message, hint, None),
    )
    .with_optional_field("hint", hint.map(|value| json!(value)))
    .fatal()
}

pub fn step_not_registered_error(step_name: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "StepNotRegisteredError",
        append_framed_details(
            &format!(
                "Step \"{step_name}\" is not registered in the current deployment. This usually indicates a build or bundling issue that caused the step to not be included in the deployment."
            ),
            None,
            Some(slugs::STEP_NOT_REGISTERED),
        ),
    )
    .with_field("stepName", json!(step_name))
}

pub fn workflow_not_registered_error(workflow_name: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "WorkflowNotRegisteredError",
        append_framed_details(
            &format!(
                "Workflow \"{workflow_name}\" is not registered in the current deployment. This usually means a run was started against a deployment that does not have this workflow, or there was a build/bundling issue."
            ),
            None,
            Some(slugs::WORKFLOW_NOT_REGISTERED),
        ),
    )
    .with_field("workflowName", json!(workflow_name))
}

pub fn workflow_deployment_mismatch_error(
    run_id: &str,
    expected_deployment_id: &str,
    actual_deployment_id: &str,
    recovery_attempts: u64,
) -> WorkflowErrorDescriptor {
    let recovery = if recovery_attempts > 0 {
        let unit = if recovery_attempts == 1 { "time" } else { "times" };
        format!(
            " The runtime re-routed the message to \"{expected_deployment_id}\" {recovery_attempts} {unit} and it kept arriving elsewhere, so the run was stopped to protect against code-skew errors."
        )
    } else {
        " The run was stopped to protect against code-skew errors.".to_owned()
    };
    let message = format!(
        "Workflow run \"{run_id}\" is pinned to deployment \"{expected_deployment_id}\", but was received by deployment \"{actual_deployment_id}\".{recovery} Verify that the run's deployment is still available and that queue callbacks are routed to it."
    );
    WorkflowErrorDescriptor::new(
        "WorkflowDeploymentMismatchError",
        append_framed_details(&message, None, Some(slugs::DEPLOYMENT_MISMATCH)),
    )
    .with_field("runId", json!(run_id))
    .with_field("expectedDeploymentId", json!(expected_deployment_id))
    .with_field("actualDeploymentId", json!(actual_deployment_id))
    .with_field("recoveryAttempts", json!(recovery_attempts))
}

pub fn workflow_run_not_found_error(run_id: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "WorkflowRunNotFoundError",
        format!("Workflow run \"{run_id}\" not found"),
    )
    .with_field("runId", json!(run_id))
}

pub fn hook_conflict_error(
    token: &str,
    conflicting_run_id: Option<&str>,
) -> WorkflowErrorDescriptor {
    let owner = conflicting_run_id
        .map(|run_id| format!(" (run \"{run_id}\")"))
        .unwrap_or_default();
    WorkflowErrorDescriptor::new(
        "HookConflictError",
        append_framed_details(
            &format!("Hook token \"{token}\" is already in use by another workflow{owner}"),
            None,
            Some(slugs::HOOK_CONFLICT),
        ),
    )
    .with_field("token", json!(token))
    .with_optional_field(
        "conflictingRunId",
        conflicting_run_id.map(|value| json!(value)),
    )
}

pub fn hook_not_found_error(token: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("HookNotFoundError", "Hook not found")
        .with_field("token", json!(token))
}

pub fn entity_conflict_error(message: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("EntityConflictError", message)
}

pub fn run_expired_error(message: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("RunExpiredError", message)
}

pub fn stream_expired_error(
    message: &str,
    run_id: Option<&str>,
    stream_id: Option<&str>,
    expired_at_ms: Option<Value>,
) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("StreamExpiredError", message)
        .with_field("status", json!(410))
        .with_field("code", json!("stream-expired"))
        .with_optional_field("runId", run_id.map(|value| json!(value)))
        .with_optional_field("streamId", stream_id.map(|value| json!(value)))
        .with_optional_field("expiredAt", expired_at_ms)
}

pub fn too_early_error(message: &str, retry_after: Option<Value>) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("TooEarlyError", message)
        .with_optional_field("retryAfter", retry_after)
}

pub fn throttle_error(message: &str, retry_after: Option<Value>) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("ThrottleError", message)
        .with_optional_field("retryAfter", retry_after)
}

pub fn precondition_failed_error(
    message: &str,
    retry_after: Option<Value>,
    details: Option<Value>,
) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("PreconditionFailedError", message)
        .with_field("status", json!(412))
        .with_optional_field("retryAfter", retry_after)
        .with_optional_field("details", details)
}

pub fn workflow_run_cancelled_error(run_id: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "WorkflowRunCancelledError",
        format!("Workflow run \"{run_id}\" cancelled"),
    )
    .with_field("runId", json!(run_id))
}

pub fn run_not_supported_error(
    run_spec_version: u64,
    world_spec_version: u64,
) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new(
        "RunNotSupportedError",
        format!(
            "Run requires spec version {run_spec_version}, but world supports version {world_spec_version}. Please upgrade 'workflow' package."
        ),
    )
    .with_field("runSpecVersion", json!(run_spec_version))
    .with_field("worldSpecVersion", json!(world_spec_version))
}

pub fn fatal_error(message: &str) -> WorkflowErrorDescriptor {
    WorkflowErrorDescriptor::new("FatalError", message).fatal()
}

pub fn retryable_error(
    message: &str,
    retry_after: Option<DurationInput>,
    now_ms: f64,
) -> ValidationResult<WorkflowErrorDescriptor> {
    let retry_at = parse_duration_to_unix_ms(
        retry_after.unwrap_or(DurationInput::Milliseconds(1_000.0)),
        now_ms,
    )?;
    Ok(WorkflowErrorDescriptor::new("RetryableError", message)
        .with_field("retryAfter", json!(retry_at)))
}

pub fn is_fatal_error(error: &WorkflowErrorDescriptor) -> bool {
    error.fatal
}

/// Builds one error descriptor from a conformance fixture.
pub fn make_error(input: &Value) -> ValidationResult<WorkflowErrorDescriptor> {
    let kind = required_string(input, "kind")?;
    let message = optional_string(input, "message")?.unwrap_or_else(|| "boom".to_owned());

    match kind.as_str() {
        "WorkflowError" => Ok(workflow_error(
            &message,
            optional_string(input, "slug")?.as_deref(),
        )),
        "WorkflowWorldError" => Ok(workflow_world_error(
            &message,
            optional_value(input, "status"),
            optional_value(input, "code"),
            optional_value(input, "url"),
            optional_value(input, "retryAfter"),
        )),
        "WorkflowRunFailedError" => {
            let run_id = required_string(input, "runId")?;
            let error = input.get("error").cloned().unwrap_or(Value::Null);
            let error_code = optional_string(input, "errorCode")?;
            Ok(workflow_run_failed_error(
                &run_id,
                &error,
                error_code.as_deref(),
            ))
        }
        "WorkflowRunNotCompletedError" => Ok(workflow_run_not_completed_error(
            &required_string(input, "runId")?,
            &required_string(input, "status")?,
        )),
        "WorkflowRuntimeError" => Ok(workflow_runtime_error(
            &message,
            optional_string(input, "slug")?.as_deref(),
        )),
        "CorruptedEventLogError" => Ok(corrupted_event_log_error(&message)),
        "ReplayDivergenceError" => Ok(replay_divergence_error(
            &message,
            &required_string(input, "eventId")?,
        )),
        "MaxEventsExceededError" => Ok(max_events_exceeded_error(
            required_value(input, "eventCount")?,
            required_value(input, "limit")?,
        )),
        "RuntimeDecryptionError" => Ok(runtime_decryption_error(
            &message,
            optional_value(input, "context"),
        )),
        "WorkflowBuildError" => Ok(workflow_build_error(
            &message,
            optional_string(input, "hint")?.as_deref(),
        )),
        "SerializationError" => Ok(serialization_error(
            &message,
            optional_string(input, "hint")?.as_deref(),
        )),
        "StepNotRegisteredError" => Ok(step_not_registered_error(&required_string(
            input, "stepName",
        )?)),
        "WorkflowNotRegisteredError" => Ok(workflow_not_registered_error(&required_string(
            input,
            "workflowName",
        )?)),
        "WorkflowDeploymentMismatchError" => Ok(workflow_deployment_mismatch_error(
            &required_string(input, "runId")?,
            &required_string(input, "expectedDeploymentId")?,
            &required_string(input, "actualDeploymentId")?,
            optional_u64(input, "recoveryAttempts")?.unwrap_or(0),
        )),
        "WorkflowRunNotFoundError" => Ok(workflow_run_not_found_error(&required_string(
            input, "runId",
        )?)),
        "HookConflictError" => {
            let conflicting_run_id = optional_string(input, "conflictingRunId")?;
            Ok(hook_conflict_error(
                &required_string(input, "token")?,
                conflicting_run_id.as_deref(),
            ))
        }
        "HookNotFoundError" => Ok(hook_not_found_error(&required_string(input, "token")?)),
        "EntityConflictError" => Ok(entity_conflict_error(&message)),
        "RunExpiredError" => Ok(run_expired_error(&message)),
        "StreamExpiredError" => {
            let run_id = optional_string(input, "runId")?;
            let stream_id = optional_string(input, "streamId")?;
            Ok(stream_expired_error(
                &message,
                run_id.as_deref(),
                stream_id.as_deref(),
                optional_value(input, "expiredAtMs"),
            ))
        }
        "TooEarlyError" => Ok(too_early_error(
            &message,
            optional_value(input, "retryAfter"),
        )),
        "ThrottleError" => Ok(throttle_error(
            &message,
            optional_value(input, "retryAfter"),
        )),
        "PreconditionFailedError" => Ok(precondition_failed_error(
            &message,
            optional_value(input, "retryAfter"),
            optional_value(input, "details"),
        )),
        "WorkflowRunCancelledError" => Ok(workflow_run_cancelled_error(&required_string(
            input, "runId",
        )?)),
        "RunNotSupportedError" => Ok(run_not_supported_error(
            required_u64(input, "runSpecVersion")?,
            required_u64(input, "worldSpecVersion")?,
        )),
        "FatalError" => Ok(fatal_error(&message)),
        "RetryableError" => {
            let retry_after = duration_input_from_fixture(input)?;
            let now_ms = required_f64(input, "nowMs")?;
            retryable_error(&message, retry_after, now_ms)
        }
        other => Err(ValidationError::new(
            "unknown_error_kind",
            format!("Unknown Workflow error kind: {other}"),
        )),
    }
}

fn duration_input_from_fixture(input: &Value) -> ValidationResult<Option<DurationInput>> {
    let Some(value) = input.get("retryAfter") else {
        return Ok(None);
    };
    let kind = optional_string(input, "retryKind")?.unwrap_or_else(|| "number".to_owned());
    match kind.as_str() {
        "string" => value
            .as_str()
            .map(|value| Some(DurationInput::String(value.to_owned())))
            .ok_or_else(|| {
                ValidationError::new(
                    "invalid_duration",
                    "retryAfter must be a string for retryKind=string",
                )
            }),
        "number" => value
            .as_f64()
            .map(|value| Some(DurationInput::Milliseconds(value)))
            .ok_or_else(|| {
                ValidationError::new(
                    "invalid_duration",
                    "retryAfter must be a number for retryKind=number",
                )
            }),
        "date" => value
            .as_f64()
            .map(|value| Some(DurationInput::DateMilliseconds(value)))
            .ok_or_else(|| {
                ValidationError::new(
                    "invalid_duration",
                    "retryAfter must be a timestamp for retryKind=date",
                )
            }),
        other => Err(ValidationError::new(
            "invalid_duration_kind",
            format!("Unknown retry duration kind: {other}"),
        )),
    }
}

fn thrown_value_message(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Object(object) if object.contains_key("message") => {
            js_value_to_string(&object["message"])
        }
        _ => "Unknown error".to_owned(),
    }
}

fn js_value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .map(js_value_to_string)
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_owned(),
    }
}

fn js_number_display(value: &Value) -> String {
    match value {
        Value::Number(number) => number.to_string(),
        other => js_value_to_string(other),
    }
}

fn required_value(input: &Value, key: &str) -> ValidationResult<Value> {
    input.get(key).cloned().ok_or_else(|| missing_field(key))
}

fn optional_value(input: &Value, key: &str) -> Option<Value> {
    input.get(key).filter(|value| !value.is_null()).cloned()
}

fn required_string(input: &Value, key: &str) -> ValidationResult<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ValidationError::new("invalid_string", format!("{key} must be a string")))
}

fn optional_string(input: &Value, key: &str) -> ValidationResult<Option<String>> {
    let Some(value) = input.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value.as_str().map(str::to_owned).map(Some).ok_or_else(|| {
        ValidationError::new(
            "invalid_string",
            format!("{key} must be a string or null"),
        )
    })
}

fn required_u64(input: &Value, key: &str) -> ValidationResult<u64> {
    input.get(key).and_then(Value::as_u64).ok_or_else(|| {
        ValidationError::new(
            "invalid_integer",
            format!("{key} must be a non-negative integer"),
        )
    })
}

fn optional_u64(input: &Value, key: &str) -> ValidationResult<Option<u64>> {
    let Some(value) = input.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value.as_u64().map(Some).ok_or_else(|| {
        ValidationError::new(
            "invalid_integer",
            format!("{key} must be a non-negative integer or null"),
        )
    })
}

fn required_f64(input: &Value, key: &str) -> ValidationResult<f64> {
    input
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| ValidationError::new("invalid_number", format!("{key} must be a number")))
}

fn missing_field(key: &str) -> ValidationError {
    ValidationError::new("missing_field", format!("Missing required field: {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_run_error_code_is_stable() {
        assert_eq!(run_error_codes::USER_ERROR, "USER_ERROR");
        assert_eq!(run_error_codes::RUNTIME_ERROR, "RUNTIME_ERROR");
        assert_eq!(run_error_codes::CORRUPTED_EVENT_LOG, "CORRUPTED_EVENT_LOG");
        assert_eq!(run_error_codes::REPLAY_DIVERGENCE, "REPLAY_DIVERGENCE");
        assert_eq!(
            run_error_codes::MAX_DELIVERIES_EXCEEDED,
            "MAX_DELIVERIES_EXCEEDED"
        );
        assert_eq!(
            run_error_codes::MAX_EVENTS_EXCEEDED,
            "MAX_EVENTS_EXCEEDED"
        );
        assert_eq!(run_error_codes::REPLAY_TIMEOUT, "REPLAY_TIMEOUT");
        assert_eq!(
            run_error_codes::WORLD_CONTRACT_ERROR,
            "WORLD_CONTRACT_ERROR"
        );
        assert_eq!(run_error_codes::DEPLOYMENT_MISMATCH, "DEPLOYMENT_MISMATCH");
    }

    #[test]
    fn unknown_error_kinds_fail_closed() {
        let error = make_error(&json!({"kind": "InventedError"})).unwrap_err();
        assert_eq!(error.code(), "unknown_error_kind");
    }
}

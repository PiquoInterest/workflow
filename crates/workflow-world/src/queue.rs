use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ValidationError, ValidationResult};

/// Only workflow queues remain part of the public contract.
pub const WORKFLOW_QUEUE_KIND: &str = "workflow";
/// Default queue topic prefix.
pub const DEFAULT_QUEUE_TOPIC_PREFIX: &str = "__wkf_workflow_";
/// Environment variable carrying an optional queue namespace.
pub const QUEUE_NAMESPACE_ENV_VAR: &str = "WORKFLOW_QUEUE_NAMESPACE";

/// Parsed queue-name components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedQueueName {
    pub prefix: String,
    pub id: String,
}

/// Resolves an explicit queue namespace ahead of the environment value.
pub fn resolve_queue_namespace(
    explicit: Option<&str>,
    environment: &BTreeMap<String, String>,
) -> Option<String> {
    explicit
        .map(str::to_owned)
        .or_else(|| environment.get(QUEUE_NAMESPACE_ENV_VAR).cloned())
}

/// Validates a queue namespace.
pub fn validate_queue_namespace(namespace: &str) -> ValidationResult<()> {
    let mut bytes = namespace.bytes();
    let Some(first) = bytes.next() else {
        return Err(ValidationError::new(
            "invalid_queue_namespace",
            "Queue namespace must not be empty",
        ));
    };
    if !first.is_ascii_lowercase()
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return Err(ValidationError::new(
            "invalid_queue_namespace",
            "Queue namespace must be lowercase alphanumeric and start with a letter",
        ));
    }
    Ok(())
}

/// Builds the workflow queue prefix.
pub fn get_queue_topic_prefix(kind: &str, namespace: Option<&str>) -> ValidationResult<String> {
    if kind != WORKFLOW_QUEUE_KIND {
        return Err(ValidationError::new(
            "unsupported_queue_kind",
            format!("Unsupported queue kind: {kind}"),
        ));
    }

    match namespace {
        None => Ok(DEFAULT_QUEUE_TOPIC_PREFIX.to_owned()),
        Some(namespace) => {
            validate_queue_namespace(namespace)?;
            Ok(format!("__{namespace}_wkf_workflow_"))
        }
    }
}

/// Whether a string is a canonical workflow queue prefix.
pub fn is_valid_queue_prefix(value: &str) -> bool {
    if value == DEFAULT_QUEUE_TOPIC_PREFIX {
        return true;
    }
    let Some(inner) = value
        .strip_prefix("__")
        .and_then(|value| value.strip_suffix("_wkf_workflow_"))
    else {
        return false;
    };
    validate_queue_namespace(inner).is_ok()
}

/// Whether a string is a complete workflow queue name.
pub fn is_valid_queue_name(value: &str) -> bool {
    split_queue_name(value).is_some()
}

/// Parses a canonical queue name into its prefix and opaque id suffix.
pub fn parse_queue_name(value: &str) -> ValidationResult<ParsedQueueName> {
    let Some((prefix, id)) = split_queue_name(value) else {
        return Err(ValidationError::new(
            "invalid_queue_name",
            format!("Invalid queue name: {value}"),
        ));
    };
    Ok(ParsedQueueName {
        prefix: prefix.to_owned(),
        id: id.to_owned(),
    })
}

fn split_queue_name(value: &str) -> Option<(&str, &str)> {
    if let Some(id) = value.strip_prefix(DEFAULT_QUEUE_TOPIC_PREFIX) {
        return valid_queue_id(id).then_some((DEFAULT_QUEUE_TOPIC_PREFIX, id));
    }

    let rest = value.strip_prefix("__")?;
    let marker = "_wkf_workflow_";
    let marker_index = rest.find(marker)?;
    let namespace = &rest[..marker_index];
    if validate_queue_namespace(namespace).is_err() {
        return None;
    }
    let id = &rest[marker_index + marker.len()..];
    if !valid_queue_id(id) {
        return None;
    }
    let prefix_length = 2 + marker_index + marker.len();
    Some((&value[..prefix_length], id))
}

fn valid_queue_id(id: &str) -> bool {
    !id.is_empty()
        && !id
            .chars()
            .any(|character| matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
}

/// OpenTelemetry propagation carrier.
pub type TraceCarrier = BTreeMap<String, String>;

/// Run creation data carried on the first resilient-start queue delivery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInput {
    pub input: Value,
    pub deployment_id: String,
    pub workflow_name: String,
    pub spec_version: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_context: Option<BTreeMap<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_reserved_attributes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

impl RunInput {
    fn validate(&self) -> ValidationResult<()> {
        if !self.spec_version.is_finite() {
            return Err(ValidationError::new(
                "invalid_run_input_spec_version",
                "runInput.specVersion must be a finite number",
            ));
        }
        if self.allow_reserved_attributes == Some(false) {
            return Err(ValidationError::new(
                "invalid_allow_reserved_attributes",
                "runInput.allowReservedAttributes may only be present with value true",
            ));
        }
        Ok(())
    }
}

/// Binary input carried by resilient step dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepDispatchInput {
    pub input: Vec<u8>,
}

/// Idempotent lazy-hook-resume input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResumeInput {
    pub resume_id: String,
    pub hook_id: String,
    pub token: String,
    pub payload: Value,
    pub payload_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
}

/// Replay-divergence continuation state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayDivergence {
    pub event_id: String,
    pub count: u64,
}

/// Identity of a delayed wait continuation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaitContinuation {
    pub correlation_id: String,
    pub attempt: u64,
}

/// Health probe sent through the workflow queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckPayload {
    #[serde(rename = "__healthCheck")]
    pub health_check: bool,
    pub correlation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

/// Workflow invocation queue payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowInvokePayload {
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_carrier: Option<TraceCarrier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_at: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_divergence: Option<ReplayDivergence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precondition_reinvocations: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_error_retry_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_mismatch_retry_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_continuation: Option<WaitContinuation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_input: Option<RunInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hook_input: Option<HookResumeInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_input: Option<StepDispatchInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hook_resume_timing: Option<Value>,
}

impl WorkflowInvokePayload {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(divergence) = &self.replay_divergence {
            if divergence.count == 0 {
                return Err(ValidationError::new(
                    "invalid_replay_divergence_count",
                    "replayDivergence.count must be positive",
                ));
            }
        }
        if self.precondition_reinvocations == Some(0) {
            return Err(ValidationError::new(
                "invalid_precondition_reinvocations",
                "preconditionReinvocations must be positive",
            ));
        }
        if let Some(run_input) = &self.run_input {
            run_input.validate()?;
        }
        Ok(())
    }
}

/// Disjoint queue payload variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueuePayload {
    HealthCheck(HealthCheckPayload),
    WorkflowInvoke(Box<WorkflowInvokePayload>),
}

/// Parses a queue payload with a hard health-check discriminator.
///
/// Zod's ordered union can otherwise fall through from a malformed object that
/// contains `__healthCheck` into the permissive invoke object and strip the
/// discriminator. Rust treats the *presence* of that key as a commitment to
/// the probe protocol and rejects malformed probes instead of replaying a run.
pub fn parse_queue_payload(value: Value) -> ValidationResult<QueuePayload> {
    let object = value.as_object().ok_or_else(|| {
        ValidationError::new("invalid_queue_payload", "Queue payload must be an object")
    })?;

    if object.contains_key("__healthCheck") {
        let payload: HealthCheckPayload = serde_json::from_value(value).map_err(|error| {
            ValidationError::new(
                "invalid_health_check_payload",
                format!("Invalid health-check payload: {error}"),
            )
        })?;
        if !payload.health_check {
            return Err(ValidationError::new(
                "invalid_health_check_discriminator",
                "__healthCheck must be true",
            ));
        }
        return Ok(QueuePayload::HealthCheck(payload));
    }

    let mut sanitized = value;
    // Zod's `.catch(undefined)` makes malformed observational/continuation
    // fields non-fatal. Reproduce that compatibility behavior before serde.
    if let Some(object) = sanitized.as_object_mut() {
        if object
            .get("waitContinuation")
            .is_some_and(|value| parse_wait_continuation(value).is_none())
        {
            object.remove("waitContinuation");
        }
        if object
            .get("hookResumeTiming")
            .is_some_and(|value| !valid_hook_resume_timing(value))
        {
            object.remove("hookResumeTiming");
        }
    }

    let payload: WorkflowInvokePayload = serde_json::from_value(sanitized).map_err(|error| {
        ValidationError::new(
            "invalid_workflow_invoke_payload",
            format!("Invalid workflow invocation payload: {error}"),
        )
    })?;
    payload.validate()?;
    Ok(QueuePayload::WorkflowInvoke(Box::new(payload)))
}

fn parse_wait_continuation(value: &Value) -> Option<WaitContinuation> {
    let object = value.as_object()?;
    let correlation_id = object.get("correlationId")?.as_str()?.to_owned();
    let attempt = json_nonnegative_integer(object.get("attempt")?)?;
    Some(WaitContinuation {
        correlation_id,
        attempt,
    })
}

fn valid_hook_resume_timing(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object
        .get("resumeRequestedAtMs")
        .and_then(Value::as_f64)
        .is_some_and(f64::is_finite)
        && object
            .get("queuePublishRequestedAtMs")
            .and_then(Value::as_f64)
            .is_some_and(f64::is_finite)
}

fn json_nonnegative_integer(value: &Value) -> Option<u64> {
    if let Some(value) = value.as_u64() {
        return Some(value);
    }
    let value = value.as_f64()?;
    (value.is_finite() && value >= 0.0 && value.fract() == 0.0 && value <= u64::MAX as f64)
        .then_some(value as u64)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn queue_prefixes_and_names_match_the_typescript_contract() {
        assert_eq!(
            get_queue_topic_prefix("workflow", None).unwrap(),
            "__wkf_workflow_"
        );
        assert_eq!(
            get_queue_topic_prefix("workflow", Some("custom")).unwrap(),
            "__custom_wkf_workflow_"
        );
        assert!(get_queue_topic_prefix("step", None).is_err());
        assert!(get_queue_topic_prefix("workflow", Some("123abc")).is_err());
        assert!(get_queue_topic_prefix("workflow", Some("Custom")).is_err());

        assert!(is_valid_queue_prefix("__wkf_workflow_"));
        assert!(is_valid_queue_prefix("__custom_wkf_workflow_"));
        assert!(!is_valid_queue_prefix("__wkf_step_"));
        assert!(is_valid_queue_name("__wkf_workflow_myFlow"));
        assert!(!is_valid_queue_name("__wkf_workflow_"));
        assert_eq!(
            parse_queue_name("__custom_wkf_workflow_myFlow").unwrap(),
            ParsedQueueName {
                prefix: "__custom_wkf_workflow_".to_owned(),
                id: "myFlow".to_owned(),
            }
        );
    }

    #[test]
    fn a_valid_probe_with_run_id_keeps_its_discriminator() {
        let parsed = parse_queue_payload(serde_json::json!({
            "__healthCheck": true,
            "correlationId": "corr_123",
            "runId": "wrun_01ABC"
        }))
        .unwrap();
        assert!(matches!(parsed, QueuePayload::HealthCheck(_)));
        assert_eq!(
            serde_json::to_value(parsed).unwrap(),
            serde_json::json!({
                "__healthCheck": true,
                "correlationId": "corr_123",
                "runId": "wrun_01ABC"
            })
        );
    }

    #[test]
    fn malformed_health_check_never_falls_through_to_invoke() {
        let error = parse_queue_payload(serde_json::json!({
            "__healthCheck": false,
            "correlationId": "corr_123",
            "runId": "wrun_01ABC"
        }))
        .unwrap_err();
        assert_eq!(error.code(), "invalid_health_check_discriminator");
    }

    #[test]
    fn invoke_payload_strips_unknown_fields_and_keeps_environment() {
        let parsed = parse_queue_payload(serde_json::json!({
            "runId": "wrun_01ABC",
            "futureTopLevel": "ignored",
            "runInput": {
                "input": { "foo": "bar" },
                "deploymentId": "dpl_123",
                "workflowName": "myWorkflow",
                "specVersion": 7,
                "environment": "preview",
                "someFutureField": "ignored"
            }
        }))
        .unwrap();
        let QueuePayload::WorkflowInvoke(payload) = parsed else {
            panic!("expected invocation payload");
        };
        let payload = *payload;
        assert_eq!(
            payload.run_input.and_then(|input| input.environment),
            Some("preview".to_owned())
        );
    }

    #[test]
    fn malformed_noncritical_wait_metadata_degrades_to_absent() {
        let parsed = parse_queue_payload(serde_json::json!({
            "runId": "wrun_01ABC",
            "waitContinuation": { "correlationId": 12, "attempt": -1 }
        }))
        .unwrap();
        let QueuePayload::WorkflowInvoke(payload) = parsed else {
            panic!("expected invocation payload");
        };
        let payload = *payload;
        assert_eq!(payload.wait_continuation, None);
    }

    #[test]
    fn non_binary_step_dispatch_is_rejected() {
        assert!(
            parse_queue_payload(serde_json::json!({
                "runId": "wrun_01ABC",
                "stepId": "step_1",
                "stepInput": { "input": "mangled-to-string" }
            }))
            .is_err()
        );
    }
}

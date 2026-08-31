use serde::Serialize;
use serde_json::{Map, Value};

use crate::{ValidationError, ValidationResult};

/// State-independent fields shared by every materialized step.
///
/// Payloads and timestamps remain JSON values in this bounded port stage. The
/// state parser is responsible for the cross-field lifecycle invariant; full
/// serialized-data and date coercion parity is tracked separately.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepCommon {
    pub run_id: String,
    pub step_id: String,
    pub step_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
    pub attempt: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<Value>,
    pub created_at: Value,
    pub updated_at: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_version: Option<u32>,
}

/// A step lifecycle state whose Rust type excludes contradictory field sets.
///
/// TypeScript's legacy `StepSchema` models every lifecycle-dependent field as
/// independently optional. This enum makes the permitted combinations
/// explicit and validates untrusted records before they enter a Rust World.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StepState {
    Pending {
        #[serde(flatten)]
        common: StepCommon,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Value>,
        #[serde(rename = "retryAfter", skip_serializing_if = "Option::is_none")]
        retry_after: Option<Value>,
    },
    Running {
        #[serde(flatten)]
        common: StepCommon,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Value>,
    },
    Completed {
        #[serde(flatten)]
        common: StepCommon,
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Value>,
        #[serde(rename = "completedAt")]
        completed_at: Value,
    },
    Failed {
        #[serde(flatten)]
        common: StepCommon,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Value>,
        #[serde(rename = "completedAt")]
        completed_at: Value,
    },
    Cancelled {
        #[serde(flatten)]
        common: StepCommon,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Value>,
        #[serde(rename = "completedAt")]
        completed_at: Value,
    },
}

/// Parses and validates the lifecycle-dependent portion of a TypeScript step.
///
/// Unknown fields are stripped, matching Zod object parsing. Field presence is
/// checked independently of value, so a forbidden `null` cannot masquerade as
/// an omitted field. Error messages name only the status and field and never
/// reflect serialized payload contents.
pub fn parse_step_state(value: &Value) -> ValidationResult<StepState> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid_step_state("Step must be an object"))?;
    let status = required_string(object, "status")?;
    let common = parse_common(object)?;
    let legacy = common.spec_version.is_none_or(|version| version <= 1);
    let error = optional_value(object, "error");

    match status.as_str() {
        "pending" => {
            reject_fields(object, "pending", &["output", "completedAt"])?;
            Ok(StepState::Pending {
                common,
                error,
                retry_after: optional_value(object, "retryAfter"),
            })
        }
        "running" => {
            reject_fields(object, "running", &["output", "completedAt", "retryAfter"])?;
            Ok(StepState::Running { common, error })
        }
        "completed" => {
            reject_fields(object, "completed", &["retryAfter"])?;
            let output = optional_value(object, "output");
            if !legacy && output.is_none() {
                return Err(missing_state_field("completed", "output"));
            }
            Ok(StepState::Completed {
                common,
                output,
                error,
                completed_at: required_state_field(object, "completed", "completedAt")?,
            })
        }
        "failed" => {
            reject_fields(object, "failed", &["output", "retryAfter"])?;
            if !legacy && error.is_none() {
                return Err(missing_state_field("failed", "error"));
            }
            Ok(StepState::Failed {
                common,
                error,
                completed_at: required_state_field(object, "failed", "completedAt")?,
            })
        }
        "cancelled" => {
            reject_fields(object, "cancelled", &["output", "retryAfter"])?;
            Ok(StepState::Cancelled {
                common,
                error,
                completed_at: required_state_field(object, "cancelled", "completedAt")?,
            })
        }
        _ => Err(invalid_step_state(format!(
            "Unknown step status \"{status}\""
        ))),
    }
}

fn parse_common(object: &Map<String, Value>) -> ValidationResult<StepCommon> {
    Ok(StepCommon {
        run_id: required_string(object, "runId")?,
        step_id: required_string(object, "stepId")?,
        step_name: required_string(object, "stepName")?,
        input: optional_value(object, "input"),
        attempt: required_number(object, "attempt")?,
        started_at: optional_value(object, "startedAt"),
        created_at: required_value(object, "createdAt")?,
        updated_at: required_value(object, "updatedAt")?,
        spec_version: optional_protocol_version(object, "specVersion")?,
    })
}

fn required_string(object: &Map<String, Value>, key: &str) -> ValidationResult<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| invalid_step_state(format!("{key} must be a string")))
}

fn required_number(object: &Map<String, Value>, key: &str) -> ValidationResult<Value> {
    let value = object
        .get(key)
        .filter(|value| value.as_f64().is_some_and(|number| number.is_finite()))
        .cloned()
        .ok_or_else(|| invalid_step_state(format!("{key} must be a finite number")))?;
    Ok(value)
}

fn required_value(object: &Map<String, Value>, key: &str) -> ValidationResult<Value> {
    object
        .get(key)
        .cloned()
        .ok_or_else(|| invalid_step_state(format!("Missing required field \"{key}\"")))
}

fn optional_value(object: &Map<String, Value>, key: &str) -> Option<Value> {
    object.get(key).cloned()
}

fn optional_protocol_version(
    object: &Map<String, Value>,
    key: &str,
) -> ValidationResult<Option<u32>> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };

    if let Some(value) = value.as_u64() {
        return u32::try_from(value)
            .ok()
            .filter(|value| *value > 0)
            .map(Some)
            .ok_or_else(|| invalid_step_state(format!("{key} must be a positive 32-bit integer")));
    }
    if let Some(value) = value.as_i64() {
        return u32::try_from(value)
            .ok()
            .filter(|value| *value > 0)
            .map(Some)
            .ok_or_else(|| invalid_step_state(format!("{key} must be a positive 32-bit integer")));
    }

    let value = value
        .as_f64()
        .filter(|value| {
            value.is_finite() && value.fract() == 0.0 && (1.0..=u32::MAX as f64).contains(value)
        })
        .map(|value| value as u32)
        .ok_or_else(|| invalid_step_state(format!("{key} must be a positive 32-bit integer")))?;
    Ok(Some(value))
}

fn reject_fields(
    object: &Map<String, Value>,
    status: &str,
    fields: &[&str],
) -> ValidationResult<()> {
    for field in fields {
        if object.contains_key(*field) {
            return Err(invalid_step_state(format!(
                "Step status \"{status}\" cannot contain field \"{field}\""
            )));
        }
    }
    Ok(())
}

fn required_state_field(
    object: &Map<String, Value>,
    status: &str,
    field: &str,
) -> ValidationResult<Value> {
    object
        .get(field)
        .cloned()
        .ok_or_else(|| missing_state_field(status, field))
}

fn missing_state_field(status: &str, field: &str) -> ValidationError {
    invalid_step_state(format!(
        "Step status \"{status}\" requires field \"{field}\""
    ))
}

fn invalid_step_state(message: impl Into<String>) -> ValidationError {
    ValidationError::new("invalid_step_state", message)
}

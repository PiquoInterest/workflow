use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::{ValidationError, ValidationResult};

/// Current lazy-hook-resume consumer protocol version.
pub const HOOK_RESUME_INPUT_VERSION: u32 = 1;
/// Current backend lazy-hook-resume deduplication protocol version.
pub const HOOK_RESUME_DEDUP_VERSION: u32 = 1;

/// Immutable slice of the owning run required to resume a hook.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResumeContext {
    pub deployment_id: String,
    pub workflow_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_spec_version: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_core_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_carrier: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_resume_input_version: Option<f64>,
}

/// Live backend capability attestation returned by a by-token lookup.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResumeCapabilities {
    pub hook_resume_dedup_version: f64,
}

/// Hook protocol fields permitted in persistent storage.
///
/// This type intentionally has no `resume_capabilities` field. The capability
/// is a fresh server attestation and persisting it would defeat rollback and
/// kill-switch behavior.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedHookProtocolFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_context: Option<HookResumeContext>,
}

/// Protocol fields returned by a live hook lookup.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookLookupProtocolFields {
    #[serde(flatten)]
    pub persisted: PersistedHookProtocolFields,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_capabilities: Option<HookResumeCapabilities>,
}

impl HookLookupProtocolFields {
    /// Drops response-only capability attestations before persistence.
    pub fn into_persisted(self) -> PersistedHookProtocolFields {
        self.persisted
    }
}

/// Parses the TypeScript `HookResumeContextSchema` contract.
///
/// Unknown fields are stripped, matching Zod's default object behavior. Null is
/// not treated as missing because `.optional()` accepts `undefined`, not null.
pub fn parse_hook_resume_context(value: Value) -> ValidationResult<HookResumeContext> {
    let object = expect_object(&value, "hook resume context")?;
    Ok(HookResumeContext {
        deployment_id: required_string(object, "deploymentId")?,
        workflow_name: required_string(object, "workflowName")?,
        run_spec_version: optional_number(object, "runSpecVersion")?,
        workflow_core_version: optional_string(object, "workflowCoreVersion")?,
        trace_carrier: optional_string_map(object, "traceCarrier")?,
        encryption_public_key: optional_string(object, "encryptionPublicKey")?,
        hook_resume_input_version: optional_number(object, "hookResumeInputVersion")?,
    })
}

/// Parses the TypeScript `HookResumeCapabilitiesSchema` contract.
pub fn parse_hook_resume_capabilities(value: Value) -> ValidationResult<HookResumeCapabilities> {
    let object = expect_object(&value, "hook resume capabilities")?;
    Ok(HookResumeCapabilities {
        hook_resume_dedup_version: required_number(object, "hookResumeDedupVersion")?,
    })
}

fn expect_object<'a>(value: &'a Value, label: &str) -> ValidationResult<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| {
        ValidationError::new(
            "invalid_hook_contract",
            format!("Expected {label} to be an object"),
        )
    })
}

fn required_string(object: &Map<String, Value>, key: &str) -> ValidationResult<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| invalid_field(key, "a string"))
}

fn optional_string(object: &Map<String, Value>, key: &str) -> ValidationResult<Option<String>> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_str()
            .map(|value| Some(value.to_owned()))
            .ok_or_else(|| invalid_field(key, "a string")),
    }
}

fn required_number(object: &Map<String, Value>, key: &str) -> ValidationResult<f64> {
    object
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| invalid_field(key, "a finite number"))
}

fn optional_number(object: &Map<String, Value>, key: &str) -> ValidationResult<Option<f64>> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_f64()
            .filter(|value| value.is_finite())
            .map(Some)
            .ok_or_else(|| invalid_field(key, "a finite number")),
    }
}

fn optional_string_map(
    object: &Map<String, Value>,
    key: &str,
) -> ValidationResult<Option<BTreeMap<String, String>>> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let Some(values) = value.as_object() else {
        return Err(invalid_field(key, "an object with string values"));
    };

    let mut output = BTreeMap::new();
    for (entry_key, entry_value) in values {
        let Some(entry_value) = entry_value.as_str() else {
            return Err(invalid_field(key, "an object with string values"));
        };
        output.insert(entry_key.clone(), entry_value.to_owned());
    }
    Ok(Some(output))
}

fn invalid_field(key: &str, expected: &str) -> ValidationError {
    ValidationError::new(
        "invalid_hook_contract",
        format!("{key} must be {expected}"),
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn optional_fields_reject_null_instead_of_treating_it_as_missing() {
        let input = json!({
            "deploymentId": "deployment_1",
            "workflowName": "processOrder",
            "runSpecVersion": null,
        });
        assert!(parse_hook_resume_context(input).is_err());
    }

    #[test]
    fn unknown_fields_are_not_serialized_back_out() {
        let parsed = parse_hook_resume_context(json!({
            "deploymentId": "deployment_1",
            "workflowName": "processOrder",
            "unknown": true,
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(parsed).unwrap(),
            json!({
                "deploymentId": "deployment_1",
                "workflowName": "processOrder",
            })
        );
    }
}

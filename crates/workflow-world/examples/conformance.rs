use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use workflow_world::attributes::{
    apply_attribute_changes, validate_attribute_changes, AttributeChange,
    AttributeValidationOptions,
};
use workflow_world::env::{env_flag_from, env_number_from, EnvNumberOptions};
use workflow_world::event_metadata::{
    classify_entity_event, entity_event_class, event_data_ref_fields, is_sealed_noop_event,
};
use workflow_world::events::{
    is_child_entity_creation_event, is_child_entity_creation_event_type,
    is_hook_event_requiring_existence, is_hook_lifecycle_event_type, is_run_event_type,
    is_step_event_type, is_terminal_run_event_type, is_terminal_step_event_type,
    is_wait_event_type, strip_event_data_refs,
};
use workflow_world::queue::{
    get_queue_topic_prefix, is_valid_queue_name, is_valid_queue_prefix, parse_queue_name,
    parse_queue_payload,
};
use workflow_world::runs::BulkCancelWorkflowRunsRequest;
use workflow_world::serialization::{validate_serialized_data_for_spec, SerializedData};
use workflow_world::shared::ResolveData;
use workflow_world::slot_identity::{
    event_id_to_slot, is_slot_body, is_slot_event_id, number_to_event_id,
};
use workflow_world::spec_version::{
    is_legacy_spec_version, minted_spec_version, requires_newer_world,
};
use workflow_world::{ValidationError, ValidationResult};

const MAX_INPUT_BYTES: u64 = 1_048_576;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConformanceRequest {
    op: String,
    #[serde(default)]
    input: Value,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ConformanceResponse {
    Success { ok: bool, value: Value },
    Failure { ok: bool, error: ErrorBody },
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

fn main() {
    let response = match execute_from_stdin() {
        Ok(value) => ConformanceResponse::Success { ok: true, value },
        Err(error) => ConformanceResponse::Failure {
            ok: false,
            error: ErrorBody {
                code: error.code().to_owned(),
                message: error.message().to_owned(),
            },
        },
    };

    match serde_json::to_string(&response) {
        Ok(response) => println!("{response}"),
        Err(error) => {
            println!(
                "{{\"ok\":false,\"error\":{{\"code\":\"response_serialization_failed\",\"message\":{}}}}}",
                serde_json::to_string(&error.to_string())
                    .unwrap_or_else(|_| "\"unknown serialization error\"".to_owned())
            );
        }
    }
}

fn execute_from_stdin() -> ValidationResult<Value> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ValidationError::new("input_read_failed", error.to_string()))?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(ValidationError::new(
            "input_too_large",
            format!("Conformance request exceeds {MAX_INPUT_BYTES} bytes"),
        ));
    }
    let request: ConformanceRequest = serde_json::from_slice(&bytes)
        .map_err(|error| ValidationError::new("invalid_request", error.to_string()))?;
    execute(request)
}

fn execute(request: ConformanceRequest) -> ValidationResult<Value> {
    match request.op.as_str() {
        "validateAttributeChanges" => validate_attributes(request.input),
        "applyAttributeChanges" => apply_attributes(request.input),
        "entityEventClass" => {
            let event_type: String = required(&request.input, "eventType")?;
            Ok(entity_event_class(&event_type)
                .map(|class| Value::String(class.as_str().to_owned()))
                .unwrap_or(Value::Null))
        }
        "classifyEntityEvent" => {
            let event_type: String = required(&request.input, "eventType")?;
            let correlation_id: Option<String> = optional(&request.input, "correlationId")?;
            to_value(classify_entity_event(
                &event_type,
                correlation_id.as_deref(),
            ))
        }
        "getEventDataRefFields" => {
            let event_type: String = required(&request.input, "eventType")?;
            to_value(event_data_ref_fields(&event_type))
        }
        "isSealedNoopEvent" => {
            let event_type: String = required(&request.input, "eventType")?;
            Ok(Value::Bool(is_sealed_noop_event(&event_type)))
        }
        "envNumber" => conformance_env_number(request.input),
        "envFlag" => conformance_env_flag(request.input),
        "mintedSpecVersion" => {
            let environment = environment_from_input(&request.input)?;
            Ok(json!(minted_spec_version(&environment)))
        }
        "isLegacySpecVersion" => {
            let version = optional_u32(&request.input, "version")?;
            Ok(Value::Bool(is_legacy_spec_version(version)))
        }
        "requiresNewerWorld" => {
            let version = optional_u32(&request.input, "version")?;
            Ok(Value::Bool(requires_newer_world(version)))
        }
        "slotToEventId" => {
            let slot: f64 = required(&request.input, "slot")?;
            Ok(Value::String(number_to_event_id(slot)?))
        }
        "eventIdToSlot" => {
            let event_id: String = required(&request.input, "eventId")?;
            Ok(event_id_to_slot(&event_id).map_or(Value::Null, |slot| json!(slot)))
        }
        "isSlotBody" => {
            let body: String = required(&request.input, "body")?;
            Ok(Value::Bool(is_slot_body(&body)))
        }
        "isSlotEventId" => {
            let event_id: String = required(&request.input, "eventId")?;
            Ok(Value::Bool(is_slot_event_id(&event_id)))
        }
        "getQueueTopicPrefix" => {
            let kind: String = required(&request.input, "kind")?;
            let namespace: Option<String> = optional(&request.input, "namespace")?;
            Ok(Value::String(get_queue_topic_prefix(
                &kind,
                namespace.as_deref(),
            )?))
        }
        "isValidQueuePrefix" => {
            let value: String = required(&request.input, "value")?;
            Ok(Value::Bool(is_valid_queue_prefix(&value)))
        }
        "isValidQueueName" => {
            let value: String = required(&request.input, "value")?;
            Ok(Value::Bool(is_valid_queue_name(&value)))
        }
        "parseQueueName" => {
            let value: String = required(&request.input, "value")?;
            to_value(parse_queue_name(&value)?)
        }
        "parseQueuePayload" => {
            let payload = request
                .input
                .get("payload")
                .cloned()
                .ok_or_else(|| missing_field("payload"))?;
            to_value(parse_queue_payload(payload)?)
        }
        "isRunEventType" => event_predicate(request.input, is_run_event_type),
        "isTerminalRunEventType" => {
            event_predicate(request.input, is_terminal_run_event_type)
        }
        "isStepEventType" => event_predicate(request.input, is_step_event_type),
        "isTerminalStepEventType" => {
            event_predicate(request.input, is_terminal_step_event_type)
        }
        "isHookLifecycleEventType" => {
            event_predicate(request.input, is_hook_lifecycle_event_type)
        }
        "isHookEventRequiringExistence" => {
            event_predicate(request.input, is_hook_event_requiring_existence)
        }
        "isWaitEventType" => event_predicate(request.input, is_wait_event_type),
        "isChildEntityCreationEventType" => {
            event_predicate(request.input, is_child_entity_creation_event_type)
        }
        "isChildEntityCreationEvent" => {
            let event = request
                .input
                .get("event")
                .ok_or_else(|| missing_field("event"))?;
            Ok(Value::Bool(is_child_entity_creation_event(event)))
        }
        "stripEventDataRefs" => {
            let event = request
                .input
                .get("event")
                .cloned()
                .ok_or_else(|| missing_field("event"))?;
            let resolve_data: ResolveData = required(&request.input, "resolveData")?;
            Ok(strip_event_data_refs(event, resolve_data))
        }
        "validateBulkCancelRequest" => {
            let value = request
                .input
                .get("request")
                .cloned()
                .ok_or_else(|| missing_field("request"))?;
            let request: BulkCancelWorkflowRunsRequest = from_value(value)?;
            request.validate()?;
            Ok(Value::Null)
        }
        "validateSerializedDataForSpec" => {
            let version = optional_u32(&request.input, "specVersion")?;
            let data: SerializedData = required(&request.input, "data")?;
            validate_serialized_data_for_spec(version, &data)?;
            Ok(Value::Null)
        }
        other => Err(ValidationError::new(
            "unknown_operation",
            format!("Unknown conformance operation: {other}"),
        )),
    }
}

fn validate_attributes(input: Value) -> ValidationResult<Value> {
    let changes: Vec<AttributeChange> = required(&input, "changes")?;
    let allow_reserved_attributes =
        optional::<bool>(&input, "allowReservedAttributes")?.unwrap_or(false);
    let existing_keys = optional::<Vec<String>>(&input, "existingKeys")?
        .map(|keys| keys.into_iter().collect::<BTreeSet<_>>());
    validate_attribute_changes(
        &changes,
        &AttributeValidationOptions {
            existing_keys,
            allow_reserved_attributes,
        },
    )?;
    Ok(Value::Null)
}

fn apply_attributes(input: Value) -> ValidationResult<Value> {
    let changes: Vec<AttributeChange> = required(&input, "changes")?;
    let existing: Option<BTreeMap<String, String>> = optional(&input, "existing")?;
    to_value(apply_attribute_changes(existing.as_ref(), &changes))
}

fn conformance_env_number(input: Value) -> ValidationResult<Value> {
    let name: String = required(&input, "name")?;
    let fallback: f64 = required(&input, "fallback")?;
    let raw: Option<String> = optional(&input, "raw")?;
    let options = EnvNumberOptions {
        min: optional::<f64>(&input, "min")?.unwrap_or(0.0),
        max: optional(&input, "max")?,
        integer: optional::<bool>(&input, "integer")?.unwrap_or(false),
    };
    let environment = raw
        .map(|raw| BTreeMap::from([(name.clone(), raw)]))
        .unwrap_or_default();
    Ok(json!(env_number_from(
        &name,
        fallback,
        options,
        &environment
    )))
}

fn conformance_env_flag(input: Value) -> ValidationResult<Value> {
    let name: String = required(&input, "name")?;
    let fallback: bool = required(&input, "fallback")?;
    let raw: Option<String> = optional(&input, "raw")?;
    let environment = raw
        .map(|raw| BTreeMap::from([(name.clone(), raw)]))
        .unwrap_or_default();
    Ok(Value::Bool(env_flag_from(&name, fallback, &environment)))
}

fn environment_from_input(input: &Value) -> ValidationResult<BTreeMap<String, String>> {
    let environment: Option<BTreeMap<String, String>> = optional(input, "environment")?;
    Ok(environment.unwrap_or_default())
}

fn event_predicate(input: Value, predicate: fn(&str) -> bool) -> ValidationResult<Value> {
    let event_type: String = required(&input, "eventType")?;
    Ok(Value::Bool(predicate(&event_type)))
}

fn optional_u32(input: &Value, key: &str) -> ValidationResult<Option<u32>> {
    let Some(value) = input.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    if let Some(value) = value.as_u64().and_then(|value| u32::try_from(value).ok()) {
        return Ok(Some(value));
    }
    Err(ValidationError::new(
        "invalid_integer",
        format!("{key} must be a non-negative 32-bit integer or null"),
    ))
}

fn required<T: DeserializeOwned>(input: &Value, key: &str) -> ValidationResult<T> {
    let value = input
        .get(key)
        .cloned()
        .ok_or_else(|| missing_field(key))?;
    from_value(value)
}

fn optional<T: DeserializeOwned>(input: &Value, key: &str) -> ValidationResult<Option<T>> {
    let Some(value) = input.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    from_value(value.clone()).map(Some)
}

fn from_value<T: DeserializeOwned>(value: Value) -> ValidationResult<T> {
    serde_json::from_value(value)
        .map_err(|error| ValidationError::new("invalid_input", error.to_string()))
}

fn to_value<T: Serialize>(value: T) -> ValidationResult<Value> {
    serde_json::to_value(value)
        .map_err(|error| ValidationError::new("output_serialization_failed", error.to_string()))
}

fn missing_field(field: &str) -> ValidationError {
    ValidationError::new("missing_field", format!("Missing required field: {field}"))
}

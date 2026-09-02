use std::io::{self, Read};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use workflow_world::errors::make_error;
use workflow_world::serde_symbols::{WORKFLOW_DESERIALIZE, WORKFLOW_SERIALIZE};
use workflow_world::time::{DurationInput, parse_duration_to_unix_ms};
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
        "serdeSymbols" => Ok(json!({
            "serialize": WORKFLOW_SERIALIZE,
            "deserialize": WORKFLOW_DESERIALIZE,
        })),
        "makeError" => serde_json::to_value(make_error(&request.input)?).map_err(|error| {
            ValidationError::new("output_serialization_failed", error.to_string())
        }),
        "parseDurationToUnixMs" => {
            let input = duration_input(&request.input)?;
            let now_ms = required_f64(&request.input, "nowMs")?;
            Ok(json!(parse_duration_to_unix_ms(input, now_ms)?))
        }
        other => Err(ValidationError::new(
            "unknown_operation",
            format!("Unknown conformance operation: {other}"),
        )),
    }
}

fn duration_input(input: &Value) -> ValidationResult<DurationInput> {
    let kind = required_string(input, "kind")?;
    let value = input.get("value").ok_or_else(|| missing_field("value"))?;

    match kind.as_str() {
        "string" => value
            .as_str()
            .map(|value| DurationInput::String(value.to_owned()))
            .ok_or_else(|| {
                ValidationError::new("invalid_duration", "value must be a string for kind=string")
            }),
        "number" => value
            .as_f64()
            .map(DurationInput::Milliseconds)
            .ok_or_else(|| {
                ValidationError::new("invalid_duration", "value must be a number for kind=number")
            }),
        "date" => value
            .as_f64()
            .map(DurationInput::DateMilliseconds)
            .ok_or_else(|| {
                ValidationError::new(
                    "invalid_duration",
                    "value must be a timestamp for kind=date",
                )
            }),
        other => Err(ValidationError::new(
            "invalid_duration_kind",
            format!("Unknown duration kind: {other}"),
        )),
    }
}

fn required_string(input: &Value, key: &str) -> ValidationResult<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ValidationError::new("invalid_string", format!("{key} must be a string")))
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

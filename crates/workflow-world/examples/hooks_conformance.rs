use std::io::{self, Read};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use workflow_world::hooks::{
    HOOK_RESUME_DEDUP_VERSION, HOOK_RESUME_INPUT_VERSION, parse_hook_resume_capabilities,
    parse_hook_resume_context,
};
use workflow_world::{ValidationError, ValidationResult};

const MAX_INPUT_BYTES: u64 = 1_048_576;

#[derive(Debug, Deserialize)]
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
        Err(error) => println!(
            "{{\"ok\":false,\"error\":{{\"code\":\"response_serialization_failed\",\"message\":{}}}}}",
            serde_json::to_string(&error.to_string())
                .unwrap_or_else(|_| "\"unknown serialization error\"".to_owned())
        ),
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
        "hookProtocolVersions" => Ok(json!({
            "hookResumeInputVersion": HOOK_RESUME_INPUT_VERSION,
            "hookResumeDedupVersion": HOOK_RESUME_DEDUP_VERSION,
        })),
        "parseHookResumeContext" => {
            let value = required_value(&request.input, "value")?;
            to_value(parse_hook_resume_context(value)?)
        }
        "parseHookResumeCapabilities" => {
            let value = required_value(&request.input, "value")?;
            to_value(parse_hook_resume_capabilities(value)?)
        }
        other => Err(ValidationError::new(
            "unknown_operation",
            format!("Unknown conformance operation: {other}"),
        )),
    }
}

fn required_value(input: &Value, key: &str) -> ValidationResult<Value> {
    input.get(key).cloned().ok_or_else(|| {
        ValidationError::new("missing_field", format!("Missing required field: {key}"))
    })
}

fn to_value<T: Serialize>(value: T) -> ValidationResult<Value> {
    serde_json::to_value(value)
        .map_err(|error| ValidationError::new("output_serialization_failed", error.to_string()))
}

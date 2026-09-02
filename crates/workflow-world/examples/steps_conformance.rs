use std::io::{self, Read};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use workflow_world::steps::{parse_step_attempt, parse_step_state};
use workflow_world::{ValidationError, ValidationResult};

const MAX_INPUT_BYTES: u64 = 1_048_576;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Operation {
    ParseStepState,
    NextStepAttempt,
}

#[derive(Debug, Deserialize)]
struct Request {
    #[serde(default)]
    operation: Option<Operation>,
    value: Value,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    Success { ok: bool, value: Value },
    Failure { ok: bool, error: ErrorBody },
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

fn main() {
    let response = match execute() {
        Ok(value) => Response::Success { ok: true, value },
        Err(error) => Response::Failure {
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
            let message = serde_json::to_string(&error.to_string())
                .unwrap_or_else(|_| "\"unknown serialization error\"".to_owned());
            println!(
                "{{\"ok\":false,\"error\":{{\"code\":\"response_serialization_failed\",\"message\":{message}}}}}"
            );
        }
    }
}

fn execute() -> ValidationResult<Value> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ValidationError::new("input_read_failed", error.to_string()))?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(ValidationError::new(
            "input_too_large",
            format!("Step conformance request exceeds {MAX_INPUT_BYTES} bytes"),
        ));
    }

    let request: Request = serde_json::from_slice(&bytes)
        .map_err(|error| ValidationError::new("invalid_request", error.to_string()))?;

    let value = match request.operation {
        None | Some(Operation::ParseStepState) => {
            serde_json::to_value(parse_step_state(&request.value)?)
        }
        Some(Operation::NextStepAttempt) => {
            serde_json::to_value(parse_step_attempt(&request.value)?.checked_next()?)
        }
    };
    value.map_err(|error| ValidationError::new("output_serialization_failed", error.to_string()))
}

use workflow_core_tdd::classify_error::{
    ErrorValue, RunErrorCode, classify_run_error, is_retryable_world_error,
};

fn assert_code(error: ErrorValue, expected: RunErrorCode) {
    assert_eq!(classify_run_error(&error), expected);
}

#[test]
fn classifies_corrupted_event_log_errors() {
    assert_code(
        ErrorValue::CorruptedEventLog,
        RunErrorCode::CorruptedEventLog,
    );
}

#[test]
fn classifies_max_events_exceeded_errors() {
    assert_code(
        ErrorValue::MaxEventsExceeded,
        RunErrorCode::MaxEventsExceeded,
    );
}

#[test]
fn classifies_replay_divergence_errors() {
    assert_code(ErrorValue::ReplayDivergence, RunErrorCode::ReplayDivergence);
}

#[test]
fn classifies_workflow_runtime_errors() {
    assert_code(ErrorValue::WorkflowRuntime, RunErrorCode::RuntimeError);
}

#[test]
fn classifies_unregistered_workflows_as_runtime_errors() {
    assert_code(
        ErrorValue::WorkflowNotRegistered,
        RunErrorCode::RuntimeError,
    );
}

#[test]
fn classifies_deployment_mismatches() {
    assert_code(
        ErrorValue::DeploymentMismatch,
        RunErrorCode::DeploymentMismatch,
    );
}

#[test]
fn classifies_plain_errors_as_user_errors() {
    assert_code(ErrorValue::PlainError, RunErrorCode::UserError);
}

#[test]
fn classifies_type_errors_as_user_errors() {
    assert_code(ErrorValue::TypeError, RunErrorCode::UserError);
}

#[test]
fn classifies_world_five_hundred_errors_as_contract_errors() {
    assert_code(
        ErrorValue::world(Some(500), None),
        RunErrorCode::WorldContractError,
    );
}

#[test]
fn classifies_world_schema_validation_failures_as_contract_errors() {
    assert_code(
        ErrorValue::world(None, Some("SCHEMA_VALIDATION")),
        RunErrorCode::WorldContractError,
    );
}

#[test]
fn classifies_world_response_parse_failures_as_contract_errors() {
    assert_code(
        ErrorValue::world(None, Some("PARSE_ERROR")),
        RunErrorCode::WorldContractError,
    );
}

#[test]
fn classifies_string_throws_as_user_errors() {
    assert_code(ErrorValue::StringThrow, RunErrorCode::UserError);
}

#[test]
fn classifies_null_throws_as_user_errors() {
    assert_code(ErrorValue::NullThrow, RunErrorCode::UserError);
}

#[test]
fn classifies_undefined_throws_as_user_errors() {
    assert_code(ErrorValue::UndefinedThrow, RunErrorCode::UserError);
}

#[test]
fn classifies_hook_conflicts_as_user_errors() {
    assert_code(ErrorValue::HookConflict, RunErrorCode::UserError);
}

#[test]
fn classifies_runtime_decryption_errors() {
    assert_code(ErrorValue::RuntimeDecryption, RunErrorCode::RuntimeError);
}

#[test]
fn classifies_raw_operation_errors_as_user_errors() {
    assert_code(
        ErrorValue::NamedError("OperationError".to_owned()),
        RunErrorCode::UserError,
    );
}

#[test]
fn classifies_explicit_world_contract_errors() {
    assert_code(
        ErrorValue::world(None, Some("WORLD_CONTRACT_ERROR")),
        RunErrorCode::WorldContractError,
    );
}

#[test]
fn classifies_transport_errors_as_world_contract_errors() {
    assert_code(
        ErrorValue::world(None, Some("TRANSPORT")),
        RunErrorCode::WorldContractError,
    );
}

#[test]
fn classifies_throttling_as_a_world_contract_error() {
    assert_code(ErrorValue::Throttle, RunErrorCode::WorldContractError);
}

#[test]
fn throttling_is_retryable() {
    assert!(is_retryable_world_error(&ErrorValue::Throttle));
}

#[test]
fn world_five_hundred_errors_are_retryable() {
    assert!(is_retryable_world_error(&ErrorValue::world(
        Some(502),
        None
    )));
    assert!(is_retryable_world_error(&ErrorValue::world(
        Some(503),
        None
    )));
}

#[test]
fn transport_and_timeout_errors_are_retryable() {
    assert!(is_retryable_world_error(&ErrorValue::world(
        None,
        Some("TRANSPORT")
    )));
    assert!(is_retryable_world_error(&ErrorValue::world(
        None,
        Some("TIMEOUT")
    )));
}

#[test]
fn permanently_missing_event_payloads_are_not_retryable() {
    assert!(!is_retryable_world_error(&ErrorValue::CorruptedEventLog));
}

#[test]
fn ordinary_world_four_hundred_errors_are_not_retryable() {
    assert!(!is_retryable_world_error(&ErrorValue::world(
        Some(400),
        None
    )));
}

#[test]
fn parse_and_schema_contract_errors_are_not_retryable() {
    assert!(!is_retryable_world_error(&ErrorValue::world(
        None,
        Some("PARSE_ERROR")
    )));
    assert!(!is_retryable_world_error(&ErrorValue::world(
        None,
        Some("SCHEMA_VALIDATION")
    )));
}

#[test]
fn too_early_step_pacing_errors_are_not_retryable_here() {
    assert!(!is_retryable_world_error(&ErrorValue::TooEarly));
}

#[test]
fn plain_and_unclassified_values_are_not_retryable() {
    for error in [
        ErrorValue::PlainError,
        ErrorValue::world(None, None),
        ErrorValue::StringThrow,
        ErrorValue::NullThrow,
        ErrorValue::UndefinedThrow,
    ] {
        assert!(!is_retryable_world_error(&error));
    }
}

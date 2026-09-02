use workflow_core_tdd::describe_error::{
    CONTEXT_ERROR_HINT, CORRUPTED_EVENT_LOG_HINT, DEPLOYMENT_MISMATCH_HINT, DescribableError,
    ErrorAttribution, ErrorDescription, MAX_DELIVERIES_HINT, MAX_EVENTS_HINT, PersistedErrorSignal,
    REPLAY_TIMEOUT_HINT, RUNTIME_ERROR_HINT, RunErrorCode, SERIALIZATION_ERROR_HINT,
    WORLD_CONTRACT_HINT, describe_error, describe_run_error,
};

fn assert_description(
    actual: ErrorDescription,
    attribution: ErrorAttribution,
    code: RunErrorCode,
    hint: Option<&'static str>,
) {
    assert_eq!(
        actual,
        ErrorDescription {
            attribution,
            error_code: code,
            hint,
        }
    );
}

fn signal(code: Option<&str>, name: Option<&str>) -> PersistedErrorSignal {
    PersistedErrorSignal {
        error_code: code.map(str::to_owned),
        error_name: name.map(str::to_owned),
    }
}

#[test]
fn plain_user_errors_are_attributed_to_the_user_without_a_hint() {
    assert_description(
        describe_error(DescribableError::PlainUser, None),
        ErrorAttribution::User,
        RunErrorCode::UserError,
        None,
    );
}

#[test]
fn non_error_throws_are_attributed_to_the_user() {
    for error in [DescribableError::NonError, DescribableError::Absent] {
        assert_eq!(
            describe_error(error, None).attribution,
            ErrorAttribution::User
        );
    }
}

#[test]
fn serialization_errors_are_user_attributed_with_a_hint() {
    let result = describe_error(DescribableError::Serialization, None);
    assert_eq!(result.attribution, ErrorAttribution::User);
    assert!(result.hint.unwrap().contains("serialized"));
}

#[test]
fn context_violation_errors_are_user_attributed_with_a_hint() {
    let result = describe_error(DescribableError::ContextViolation, None);
    assert_eq!(result.attribution, ErrorAttribution::User);
    assert!(result.hint.unwrap().contains("wrong context"));
}

#[test]
fn workflow_runtime_errors_are_attributed_to_the_sdk() {
    let result = describe_error(DescribableError::WorkflowRuntime, None);
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::RuntimeError);
    assert!(result.hint.unwrap().contains("internal workflow SDK error"));
}

#[test]
fn corrupted_event_log_errors_receive_the_distinct_sdk_code() {
    let result = describe_error(DescribableError::CorruptedEventLog, None);
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::CorruptedEventLog);
    assert!(result.hint.unwrap().contains("event log contains"));
}

#[test]
fn step_not_registered_errors_are_attributed_to_the_sdk() {
    let result = describe_error(DescribableError::StepNotRegistered, None);
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::RuntimeError);
}

#[test]
fn precomputed_replay_timeout_is_attributed_to_the_sdk() {
    let result = describe_error(DescribableError::Absent, Some(RunErrorCode::ReplayTimeout));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::ReplayTimeout);
    assert!(
        result
            .hint
            .unwrap()
            .contains("replay between step boundaries took too long")
    );
}

#[test]
fn precomputed_max_deliveries_is_attributed_to_the_sdk() {
    let result = describe_error(
        DescribableError::Absent,
        Some(RunErrorCode::MaxDeliveriesExceeded),
    );
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::MaxDeliveriesExceeded);
    assert!(result.hint.unwrap().contains("max-delivery budget"));
}

#[test]
fn precomputed_world_contract_error_is_attributed_to_the_sdk() {
    let result = describe_error(
        DescribableError::Absent,
        Some(RunErrorCode::WorldContractError),
    );
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::WorldContractError);
    assert!(result.hint.unwrap().contains("SDK contract"));
}

#[test]
fn precomputed_max_events_is_attributed_to_the_user() {
    let result = describe_error(
        DescribableError::Absent,
        Some(RunErrorCode::MaxEventsExceeded),
    );
    assert_eq!(result.attribution, ErrorAttribution::User);
    assert_eq!(result.error_code, RunErrorCode::MaxEventsExceeded);
    assert!(result.hint.unwrap().contains("maximum number of events"));
}

#[test]
fn deployment_mismatch_is_attributed_to_the_sdk_with_a_pinning_hint() {
    let result = describe_error(DescribableError::DeploymentMismatch, None);
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::DeploymentMismatch);
    assert!(result.hint.unwrap().contains("deployment it is pinned to"));
}

#[test]
fn precomputed_error_code_wins_over_live_error_classification() {
    let result = describe_error(
        DescribableError::PlainUser,
        Some(RunErrorCode::ReplayTimeout),
    );
    assert_eq!(result.error_code, RunErrorCode::ReplayTimeout);
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
}

#[test]
fn persisted_plain_user_error_has_no_hint() {
    assert_description(
        describe_run_error(&signal(Some("USER_ERROR"), Some("Error"))),
        ErrorAttribution::User,
        RunErrorCode::UserError,
        None,
    );
}

#[test]
fn persisted_serialization_error_name_restores_the_hint() {
    let result = describe_run_error(&signal(Some("USER_ERROR"), Some("SerializationError")));
    assert_eq!(result.attribution, ErrorAttribution::User);
    assert!(result.hint.unwrap().contains("serialized"));
}

#[test]
fn persisted_context_error_name_restores_the_hint() {
    let result = describe_run_error(&signal(
        Some("USER_ERROR"),
        Some("NotInWorkflowContextError"),
    ));
    assert_eq!(result.attribution, ErrorAttribution::User);
    assert!(result.hint.unwrap().contains("wrong context"));
}

#[test]
fn persisted_workflow_runtime_name_is_attributed_to_the_sdk() {
    let result = describe_run_error(&signal(Some("RUNTIME_ERROR"), Some("WorkflowRuntimeError")));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert!(result.hint.unwrap().contains("internal workflow SDK error"));
}

#[test]
fn persisted_replay_timeout_code_is_attributed_to_the_sdk() {
    let result = describe_run_error(&signal(Some("REPLAY_TIMEOUT"), None));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert!(
        result
            .hint
            .unwrap()
            .contains("replay between step boundaries took too long")
    );
}

#[test]
fn persisted_corrupted_event_log_code_is_attributed_to_the_sdk() {
    let result = describe_run_error(&signal(Some("CORRUPTED_EVENT_LOG"), None));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert!(result.hint.unwrap().contains("event log contains"));
}

#[test]
fn corrupted_event_log_name_restores_the_distinct_code() {
    let result = describe_run_error(&signal(None, Some("CorruptedEventLogError")));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::CorruptedEventLog);
    assert!(result.hint.unwrap().contains("event log contains"));
}

#[test]
fn persisted_max_deliveries_code_is_attributed_to_the_sdk() {
    let result = describe_run_error(&signal(Some("MAX_DELIVERIES_EXCEEDED"), None));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert!(result.hint.unwrap().contains("max-delivery budget"));
}

#[test]
fn persisted_max_events_code_is_attributed_to_the_user() {
    let result = describe_run_error(&signal(Some("MAX_EVENTS_EXCEEDED"), None));
    assert_eq!(result.attribution, ErrorAttribution::User);
    assert_eq!(result.error_code, RunErrorCode::MaxEventsExceeded);
    assert!(result.hint.unwrap().contains("maximum number of events"));
}

#[test]
fn persisted_world_contract_code_is_attributed_to_the_sdk() {
    let result = describe_run_error(&signal(Some("WORLD_CONTRACT_ERROR"), None));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert!(result.hint.unwrap().contains("SDK contract"));
}

#[test]
fn persisted_deployment_mismatch_code_is_attributed_to_the_sdk() {
    let result = describe_run_error(&signal(
        Some("DEPLOYMENT_MISMATCH"),
        Some("WorkflowDeploymentMismatchError"),
    ));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert_eq!(result.error_code, RunErrorCode::DeploymentMismatch);
    assert!(result.hint.unwrap().contains("deployment it is pinned to"));
}

#[test]
fn runtime_code_without_an_error_name_still_maps_to_the_sdk() {
    let result = describe_run_error(&signal(Some("RUNTIME_ERROR"), None));
    assert_eq!(result.attribution, ErrorAttribution::Sdk);
    assert!(result.hint.unwrap().contains("internal workflow SDK error"));
}

#[test]
fn missing_persisted_code_defaults_to_user_error() {
    assert_description(
        describe_run_error(&PersistedErrorSignal::default()),
        ErrorAttribution::User,
        RunErrorCode::UserError,
        None,
    );
}

#[test]
fn plain_user_error_payload_shape_is_exact() {
    assert_description(
        describe_error(DescribableError::PlainUser, None),
        ErrorAttribution::User,
        RunErrorCode::UserError,
        None,
    );
}

#[test]
fn serialization_error_payload_shape_is_exact() {
    assert_description(
        describe_error(DescribableError::Serialization, None),
        ErrorAttribution::User,
        RunErrorCode::UserError,
        Some(SERIALIZATION_ERROR_HINT),
    );
}

#[test]
fn context_violation_payload_shape_is_exact() {
    assert_description(
        describe_error(DescribableError::ContextViolation, None),
        ErrorAttribution::User,
        RunErrorCode::UserError,
        Some(CONTEXT_ERROR_HINT),
    );
}

#[test]
fn workflow_runtime_payload_shape_is_exact() {
    assert_description(
        describe_error(DescribableError::WorkflowRuntime, None),
        ErrorAttribution::Sdk,
        RunErrorCode::RuntimeError,
        Some(RUNTIME_ERROR_HINT),
    );
}

#[test]
fn corrupted_event_log_payload_shape_is_exact() {
    assert_description(
        describe_error(DescribableError::CorruptedEventLog, None),
        ErrorAttribution::Sdk,
        RunErrorCode::CorruptedEventLog,
        Some(CORRUPTED_EVENT_LOG_HINT),
    );
}

#[test]
fn replay_timeout_payload_shape_is_exact() {
    assert_description(
        describe_error(DescribableError::Absent, Some(RunErrorCode::ReplayTimeout)),
        ErrorAttribution::Sdk,
        RunErrorCode::ReplayTimeout,
        Some(REPLAY_TIMEOUT_HINT),
    );
}

#[test]
fn max_deliveries_payload_shape_is_exact() {
    assert_description(
        describe_error(
            DescribableError::Absent,
            Some(RunErrorCode::MaxDeliveriesExceeded),
        ),
        ErrorAttribution::Sdk,
        RunErrorCode::MaxDeliveriesExceeded,
        Some(MAX_DELIVERIES_HINT),
    );
}

#[test]
fn world_contract_payload_shape_is_exact() {
    assert_description(
        describe_error(
            DescribableError::Absent,
            Some(RunErrorCode::WorldContractError),
        ),
        ErrorAttribution::Sdk,
        RunErrorCode::WorldContractError,
        Some(WORLD_CONTRACT_HINT),
    );
}

#[test]
fn remaining_static_hints_match_the_typescript_contract() {
    assert!(MAX_EVENTS_HINT.contains("maximum number of events"));
    assert!(DEPLOYMENT_MISMATCH_HINT.contains("deployment it is pinned to"));
}

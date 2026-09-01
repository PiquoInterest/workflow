use serde_json::json;
use workflow_world::queue::{
    QueuePayload, RunInput, get_queue_topic_prefix, is_valid_queue_name, is_valid_queue_prefix,
    parse_queue_name, parse_queue_payload,
};

#[test]
fn returns_default_workflow_prefix_without_namespace() {
    assert_eq!(
        get_queue_topic_prefix("workflow", None).unwrap(),
        "__wkf_workflow_"
    );
}

#[test]
fn returns_namespaced_workflow_prefix() {
    assert_eq!(
        get_queue_topic_prefix("workflow", Some("custom")).unwrap(),
        "__custom_wkf_workflow_"
    );
}

#[test]
fn accepts_multi_character_namespace() {
    assert_eq!(
        get_queue_topic_prefix("workflow", Some("myframework123")).unwrap(),
        "__myframework123_wkf_workflow_"
    );
}

#[test]
fn rejects_the_retired_step_queue_kind_at_runtime() {
    let error = get_queue_topic_prefix("step", None).unwrap_err();
    assert_eq!(error.message(), "Unsupported queue kind: step");
}

#[test]
fn rejects_a_namespace_starting_with_a_digit() {
    assert!(get_queue_topic_prefix("workflow", Some("123abc")).is_err());
}

#[test]
fn rejects_an_uppercase_namespace() {
    assert!(get_queue_topic_prefix("workflow", Some("Custom")).is_err());
}

#[test]
fn rejects_an_empty_namespace() {
    assert!(get_queue_topic_prefix("workflow", Some("")).is_err());
}

#[test]
fn rejects_namespaces_with_special_characters() {
    assert!(get_queue_topic_prefix("workflow", Some("my-framework")).is_err());
    assert!(get_queue_topic_prefix("workflow", Some("my_framework")).is_err());
}

#[test]
fn undefined_namespace_matches_no_namespace() {
    assert_eq!(
        get_queue_topic_prefix("workflow", None).unwrap(),
        get_queue_topic_prefix("workflow", None).unwrap()
    );
}

#[test]
fn queue_prefix_accepts_the_default_workflow_prefix() {
    assert!(is_valid_queue_prefix("__wkf_workflow_"));
}

#[test]
fn queue_prefix_accepts_a_namespaced_workflow_prefix() {
    assert!(is_valid_queue_prefix("__custom_wkf_workflow_"));
}

#[test]
fn queue_prefix_rejects_retired_step_prefixes() {
    assert!(!is_valid_queue_prefix("__wkf_step_"));
    assert!(!is_valid_queue_prefix("__custom_wkf_step_"));
}

#[test]
fn queue_prefix_rejects_an_invalid_prefix() {
    assert!(!is_valid_queue_prefix("bad_prefix"));
}

#[test]
fn queue_prefix_requires_the_trailing_underscore() {
    assert!(!is_valid_queue_prefix("__wkf_workflow"));
}

#[test]
fn queue_prefix_rejects_uppercase_namespaces() {
    assert!(!is_valid_queue_prefix("__Custom_wkf_workflow_"));
}

#[test]
fn queue_name_accepts_default_queue_names() {
    assert!(is_valid_queue_name("__wkf_workflow_myFlow"));
}

#[test]
fn queue_name_accepts_namespaced_queue_names() {
    assert!(is_valid_queue_name("__custom_wkf_workflow_myFlow"));
}

#[test]
fn queue_name_rejects_retired_step_queue_names() {
    assert!(!is_valid_queue_name("__wkf_step_myStep"));
}

#[test]
fn queue_name_rejects_a_prefix_without_an_id() {
    assert!(!is_valid_queue_name("__wkf_workflow_"));
}

#[test]
fn queue_name_rejects_invalid_names() {
    assert!(!is_valid_queue_name("not_a_queue_name"));
}

#[test]
fn parses_default_workflow_queue_names() {
    let parsed = parse_queue_name("__wkf_workflow_myFlow").unwrap();
    assert_eq!(parsed.prefix, "__wkf_workflow_");
    assert_eq!(parsed.id, "myFlow");
}

#[test]
fn parses_namespaced_workflow_queue_names() {
    let parsed = parse_queue_name("__custom_wkf_workflow_myFlow").unwrap();
    assert_eq!(parsed.prefix, "__custom_wkf_workflow_");
    assert_eq!(parsed.id, "myFlow");
}

#[test]
fn preserves_the_health_check_discriminator_when_a_probe_has_a_run_id() {
    let fixture = json!({
        "__healthCheck": true,
        "correlationId": "corr_123",
        "runId": "wrun_01ABC",
    });
    let parsed = parse_queue_payload(fixture.clone()).unwrap();

    assert!(matches!(parsed, QueuePayload::HealthCheck(_)));
    assert_eq!(serde_json::to_value(parsed).unwrap(), fixture);
}

#[test]
fn preserves_health_check_payloads_without_a_run_id() {
    let fixture = json!({
        "__healthCheck": true,
        "correlationId": "corr_123",
    });
    let parsed = parse_queue_payload(fixture.clone()).unwrap();

    assert!(matches!(parsed, QueuePayload::HealthCheck(_)));
    assert_eq!(serde_json::to_value(parsed).unwrap(), fixture);
}

#[test]
fn still_resolves_workflow_invoke_payloads() {
    let parsed = parse_queue_payload(json!({
        "runId": "wrun_01ABC",
        "stepId": "step_1",
    }))
    .unwrap();

    let QueuePayload::WorkflowInvoke(payload) = parsed else {
        panic!("expected workflow invoke payload");
    };
    assert_eq!(payload.run_id, "wrun_01ABC");
    assert_eq!(payload.step_id.as_deref(), Some("step_1"));
}

#[test]
fn round_trips_binary_step_dispatch_input() {
    let parsed = parse_queue_payload(json!({
        "runId": "wrun_01ABC",
        "stepId": "step_1",
        "stepName": "myStep",
        "stepInput": { "input": [1, 2, 3] },
    }))
    .unwrap();

    let QueuePayload::WorkflowInvoke(payload) = &parsed else {
        panic!("expected workflow invoke payload");
    };
    assert_eq!(
        payload.step_input.as_ref().map(|input| input.input.as_slice()),
        Some([1_u8, 2, 3].as_slice())
    );
    assert_eq!(
        serde_json::to_value(parsed).unwrap()["stepInput"]["input"],
        json!([1, 2, 3])
    );
}

#[test]
fn rejects_non_binary_step_dispatch_input() {
    assert!(
        parse_queue_payload(json!({
            "runId": "wrun_01ABC",
            "stepId": "step_1",
            "stepName": "myStep",
            "stepInput": { "input": "mangled-to-string" },
        }))
        .is_err()
    );
}

fn base_run_input() -> serde_json::Value {
    json!({
        "input": { "foo": "bar" },
        "deploymentId": "dpl_123",
        "workflowName": "myWorkflow",
        "specVersion": 5,
    })
}

#[test]
fn run_input_round_trips_the_creator_environment() {
    let mut fixture = base_run_input();
    fixture["environment"] = json!("preview");
    let parsed: RunInput = serde_json::from_value(fixture).unwrap();

    assert_eq!(parsed.environment.as_deref(), Some("preview"));
}

#[test]
fn run_input_leaves_environment_absent_when_omitted() {
    let parsed: RunInput = serde_json::from_value(base_run_input()).unwrap();
    assert!(parsed.environment.is_none());
    assert!(
        !serde_json::to_value(parsed)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("environment")
    );
}

#[test]
fn run_input_rejects_a_non_string_environment() {
    let mut fixture = base_run_input();
    fixture["environment"] = json!(1);
    assert!(serde_json::from_value::<RunInput>(fixture).is_err());
}

#[test]
fn run_input_strips_unknown_keys_for_older_consumers() {
    let mut fixture = base_run_input();
    fixture["someFutureField"] = json!("ignored");
    let parsed: RunInput = serde_json::from_value(fixture).unwrap();
    let encoded = serde_json::to_value(parsed).unwrap();

    assert!(encoded.get("someFutureField").is_none());
    assert_eq!(encoded["deploymentId"], "dpl_123");
    assert_eq!(encoded["workflowName"], "myWorkflow");
}

#[test]
fn invoke_payload_carries_the_creator_environment() {
    let mut run_input = base_run_input();
    run_input["environment"] = json!("production");
    let parsed = parse_queue_payload(json!({
        "runId": "wrun_01ABC",
        "runInput": run_input,
    }))
    .unwrap();

    let QueuePayload::WorkflowInvoke(payload) = parsed else {
        panic!("expected workflow invoke payload");
    };
    assert_eq!(
        payload
            .run_input
            .and_then(|input| input.environment)
            .as_deref(),
        Some("production")
    );
}

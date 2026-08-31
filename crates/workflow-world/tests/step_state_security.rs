use serde_json::{Value, json};
use workflow_world::steps::{StepState, parse_step_state};

fn step(status: &str, extra: Value) -> Value {
    let mut value = json!({
        "runId": "wrun_1",
        "stepId": "step_1",
        "stepName": "step//./src/workflows/order//processPayment",
        "status": status,
        "attempt": 1,
        "specVersion": 7,
        "createdAt": "2026-01-01T00:00:00.000Z",
        "updatedAt": "2026-01-01T00:00:01.000Z"
    });
    value
        .as_object_mut()
        .expect("test fixture must be an object")
        .extend(
            extra
                .as_object()
                .expect("test extension must be an object")
                .clone(),
        );
    value
}

#[test]
fn rejects_every_contradictory_state_characterized_in_typescript() {
    let fixtures = [
        (
            "pending output",
            step(
                "pending",
                json!({
                    "output": { "secret": "pending-output" },
                    "completedAt": "2026-01-01T00:00:02.000Z"
                }),
            ),
        ),
        (
            "running retry and completion timestamps",
            step(
                "running",
                json!({
                    "retryAfter": "2026-01-01T00:00:03.000Z",
                    "completedAt": "2026-01-01T00:00:04.000Z"
                }),
            ),
        ),
        (
            "modern completed step without output",
            step(
                "completed",
                json!({ "completedAt": "2026-01-01T00:00:05.000Z" }),
            ),
        ),
        (
            "modern failed step without error",
            step(
                "failed",
                json!({
                    "output": { "secret": "failed-output" },
                    "completedAt": "2026-01-01T00:00:06.000Z"
                }),
            ),
        ),
        (
            "cancelled output without completion time",
            step(
                "cancelled",
                json!({ "output": { "secret": "cancelled-output" } }),
            ),
        ),
    ];

    for (name, fixture) in fixtures {
        let error = parse_step_state(&fixture).expect_err(name);
        assert_eq!(error.code(), "invalid_step_state");
    }
}

#[test]
fn accepts_each_representable_state_without_cross_state_fields() {
    assert!(matches!(
        parse_step_state(&step(
            "pending",
            json!({
                "error": { "kind": "retry" },
                "retryAfter": "2026-01-01T00:00:10.000Z"
            })
        )),
        Ok(StepState::Pending { .. })
    ));
    assert!(matches!(
        parse_step_state(&step("running", json!({ "error": { "kind": "retry" } }))),
        Ok(StepState::Running { .. })
    ));
    assert!(matches!(
        parse_step_state(&step(
            "completed",
            json!({
                "output": { "kind": "binary" },
                "error": { "kind": "earlier-retry" },
                "completedAt": "2026-01-01T00:00:11.000Z"
            })
        )),
        Ok(StepState::Completed { .. })
    ));
    assert!(matches!(
        parse_step_state(&step(
            "failed",
            json!({
                "error": { "kind": "terminal" },
                "completedAt": "2026-01-01T00:00:12.000Z"
            })
        )),
        Ok(StepState::Failed { .. })
    ));
    assert!(matches!(
        parse_step_state(&step(
            "cancelled",
            json!({ "completedAt": "2026-01-01T00:00:13.000Z" })
        )),
        Ok(StepState::Cancelled { .. })
    ));
}

#[test]
fn preserves_legacy_undefined_terminal_payload_compatibility() {
    let mut completed = step(
        "completed",
        json!({ "completedAt": "2026-01-01T00:00:11.000Z" }),
    );
    completed["specVersion"] = json!(1);
    assert!(matches!(
        parse_step_state(&completed),
        Ok(StepState::Completed { output: None, .. })
    ));

    let mut failed = step(
        "failed",
        json!({ "completedAt": "2026-01-01T00:00:12.000Z" }),
    );
    failed["specVersion"] = json!(1);
    assert!(matches!(
        parse_step_state(&failed),
        Ok(StepState::Failed { error: None, .. })
    ));
}

#[test]
fn treats_present_null_as_present_and_forbidden_null_as_forbidden() {
    assert!(matches!(
        parse_step_state(&step(
            "completed",
            json!({
                "output": null,
                "completedAt": "2026-01-01T00:00:11.000Z"
            })
        )),
        Ok(StepState::Completed {
            output: Some(Value::Null),
            ..
        })
    ));

    let error = parse_step_state(&step("running", json!({ "retryAfter": null })))
        .expect_err("field presence must not be confused with a missing field");
    assert_eq!(error.code(), "invalid_step_state");
}

#[test]
fn validation_errors_never_reflect_serialized_payloads() {
    let error = parse_step_state(&step(
        "pending",
        json!({ "output": "TOP-SECRET-PAYLOAD" }),
    ))
    .expect_err("pending output must be rejected");

    assert_eq!(error.code(), "invalid_step_state");
    assert!(!error.message().contains("TOP-SECRET-PAYLOAD"));
}

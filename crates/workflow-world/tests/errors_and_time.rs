use serde_json::json;
use workflow_world::errors::{
    append_framed_details, corrupted_event_log_error, fatal_error, is_fatal_error,
    replay_divergence_error, retryable_error, run_not_supported_error, serialization_error,
    workflow_build_error, workflow_deployment_mismatch_error, workflow_error,
};
use workflow_world::serde_symbols::{WORKFLOW_DESERIALIZE, WORKFLOW_SERIALIZE};
use workflow_world::time::{DurationInput, parse_duration_to_unix_ms};

#[test]
fn serde_registry_keys_match_the_typescript_symbols() {
    assert_eq!(WORKFLOW_SERIALIZE, "workflow-serialize");
    assert_eq!(WORKFLOW_DESERIALIZE, "workflow-deserialize");
}

#[test]
fn framed_details_preserve_multiline_tree_geometry() {
    assert_eq!(
        append_framed_details(
            "Build failed",
            Some("install the package\nand retry"),
            Some("serialization-failed")
        ),
        "Build failed\n├▶ hint: install the package\n│  and retry\n╰▶ docs: https://workflow-sdk.dev/err/serialization-failed"
    );
}

#[test]
fn error_constructors_match_public_names_messages_and_fields() {
    let base = workflow_error("boom", None);
    assert_eq!(base.name, "WorkflowError");
    assert_eq!(base.message, "boom");

    let corrupted = corrupted_event_log_error("event mismatch");
    assert_eq!(corrupted.name, "CorruptedEventLogError");
    assert!(corrupted.message.ends_with(
        "╰▶ docs: https://workflow-sdk.dev/err/corrupted-event-log"
    ));

    let divergence = replay_divergence_error("consumer mismatch", "event-1");
    assert_eq!(divergence.fields.get("eventId"), Some(&json!("event-1")));

    let build = workflow_build_error("Build failed", Some("run pnpm install"));
    assert_eq!(
        build.message,
        "Build failed\n╰▶ hint: run pnpm install"
    );
    assert_eq!(build.fields.get("hint"), Some(&json!("run pnpm install")));

    let unsupported = run_not_supported_error(8, 7);
    assert_eq!(
        unsupported.message,
        "Run requires spec version 8, but world supports version 7. Please upgrade 'workflow' package."
    );
}

#[test]
fn fatal_classification_is_strict() {
    assert!(is_fatal_error(&fatal_error("stop")));
    assert!(is_fatal_error(&serialization_error("cannot encode", None)));
    assert!(!is_fatal_error(&workflow_error("retryable", None)));
}

#[test]
fn deployment_mismatch_pluralization_matches_typescript() {
    let one = workflow_deployment_mismatch_error("run-1", "dpl-a", "dpl-b", 1);
    assert!(one.message.contains("1 time and it kept arriving elsewhere"));

    let many = workflow_deployment_mismatch_error("run-1", "dpl-a", "dpl-b", 2);
    assert!(many.message.contains("2 times and it kept arriving elsewhere"));
}

#[test]
fn duration_parser_matches_retry_scheduling_and_rejects_invalid_dates() {
    let now = 1_700_000_000_000.0;
    assert_eq!(
        parse_duration_to_unix_ms(DurationInput::String("5s".to_owned()), now).unwrap(),
        1_700_000_005_000
    );
    assert_eq!(
        parse_duration_to_unix_ms(DurationInput::Milliseconds(1_500.0), now).unwrap(),
        1_700_000_001_500
    );
    assert!(
        parse_duration_to_unix_ms(
            DurationInput::DateMilliseconds(8_640_000_000_000_001.0),
            now
        )
        .is_err()
    );
    assert!(
        parse_duration_to_unix_ms(DurationInput::Milliseconds(f64::MAX), now).is_err()
    );

    let retry = retryable_error(
        "try again",
        Some(DurationInput::Milliseconds(250.0)),
        now,
    )
    .unwrap();
    assert_eq!(retry.fields.get("retryAfter"), Some(&json!(1_700_000_000_250_i64)));
}

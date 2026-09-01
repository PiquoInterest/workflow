use std::collections::BTreeMap;

use workflow_core_tdd::log_format::{LogMetadata, LogValue, compose_log_line};

const PREFIX: &str = "[workflow-sdk]";

fn metadata(values: &[(&str, LogValue)]) -> LogMetadata {
    values
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect::<BTreeMap<_, _>>()
}

fn text(value: &str) -> LogValue {
    LogValue::Text(value.to_owned())
}

#[test]
fn returns_only_prefixed_framing_without_metadata_or_stack() {
    assert_eq!(
        compose_log_line(PREFIX, "something happened", None),
        "[workflow-sdk] something happened"
    );
    assert_eq!(
        compose_log_line(PREFIX, "something happened", Some(&LogMetadata::new())),
        "[workflow-sdk] something happened"
    );
}

#[test]
fn renders_structured_fields_between_framing_and_stack() {
    let message = [
        "Step add (./workflows/x) threw a FatalError — bubbling up to parent workflow",
        "FatalError: User threw a FatalError",
        "    at maybeFailingStep (./workflows/x.ts:15:11)",
        "    at <unknown> (../../packages/core/src/runtime/step-executor.ts:535:32)",
    ]
    .join("\n");
    let fields = metadata(&[
        ("workflowRunId", text("wrun_01ABC")),
        ("stepId", text("step_01XYZ")),
        ("stepName", text("step//./workflows/x//add")),
        ("errorAttribution", text("user")),
        ("errorName", text("FatalError")),
        ("errorMessage", text("User threw a FatalError")),
    ]);
    assert_eq!(
        compose_log_line(PREFIX, &message, Some(&fields)),
        "[workflow-sdk] Step add (./workflows/x) threw a FatalError — bubbling up to parent workflow\n  user error · FatalError\n  run    wrun_01ABC\n  step   step_01XYZ · add (./workflows/x)\nFatalError: User threw a FatalError\n    at maybeFailingStep (./workflows/x.ts:15:11)\n    at <unknown> (../../packages/core/src/runtime/step-executor.ts:535:32)"
    );
}

#[test]
fn collapses_framework_internal_frames_with_summary_rows() {
    let stack = [
        "FatalError: boom",
        "    at userStep (./workflows/x.ts:15:11)",
        "    at <unknown> (../../packages/core/src/runtime/step-executor.ts:535:32)",
        "    at <unknown> (.../node_modules/.pnpm/next@16.2.1/dist/server/base-server.js:1454:9)",
        "    at <unknown> (.../node_modules/.pnpm/next@16.2.1/dist/server/dev/next-dev-server.js:394:20)",
        "    at <unknown> (.../node_modules/.pnpm/@opentelemetry+api@1.9.1/build/src/api/trace.js:160:25)",
        "    at <unknown> (../../packages/core/src/runtime/helpers.ts:414:12)",
        "    at <unknown> (node:internal/process/task_queues:64:5)",
        "    at <unknown> (.../node_modules/next/dist/server/lib/start-server.js:225:13)",
    ]
    .join("\n");
    assert_eq!(
        compose_log_line(PREFIX, &format!("Step blew up\n{stack}"), None),
        "[workflow-sdk] Step blew up\nFatalError: boom\n    at userStep (./workflows/x.ts:15:11)\n    at <unknown> (../../packages/core/src/runtime/step-executor.ts:535:32)\n        … 3 more frames in framework internals\n    at <unknown> (../../packages/core/src/runtime/helpers.ts:414:12)\n        … 2 more frames in framework internals"
    );
}

#[test]
fn renders_sdk_attributed_errors_with_the_sdk_badge() {
    let fields = metadata(&[
        ("errorCode", text("CORRUPTED_EVENT_LOG")),
        ("errorAttribution", text("sdk")),
        ("errorName", text("CorruptedEventLogError")),
        ("errorMessage", text("corrupted event log")),
        ("hint", text("This is an internal workflow SDK error.")),
    ]);
    assert_eq!(
        compose_log_line(
            PREFIX,
            "Workflow myFlow failed due to an SDK runtime error\nCorruptedEventLogError: corrupted event log",
            Some(&fields),
        ),
        "[workflow-sdk] Workflow myFlow failed due to an SDK runtime error\n  sdk error · CorruptedEventLogError\n  code   CORRUPTED_EVENT_LOG\n  hint: This is an internal workflow SDK error.\nCorruptedEventLogError: corrupted event log"
    );
}

#[test]
fn drops_error_message_when_the_framing_line_already_contains_it() {
    let fields = metadata(&[
        ("errorAttribution", text("user")),
        ("errorName", text("Error")),
        ("errorMessage", text("thing went wrong")),
    ]);
    let output = compose_log_line(
        PREFIX,
        "Workflow simple threw: thing went wrong\nError: thing went wrong",
        Some(&fields),
    );
    assert!(!output.lines().any(|line| line.trim_start().starts_with("message ")));
    assert_eq!(
        output,
        "[workflow-sdk] Workflow simple threw: thing went wrong\n  user error · Error\nError: thing went wrong"
    );
}

#[test]
fn falls_back_gracefully_for_unparseable_machine_names() {
    let fields = metadata(&[
        ("workflowRunId", text("wrun_X")),
        ("workflowName", text("not-a-machine-name")),
    ]);
    assert!(compose_log_line(PREFIX, "msg", Some(&fields)).contains("wrun_X"));
}

#[test]
fn renders_unknown_fields_as_a_sorted_key_value_tail() {
    let fields = metadata(&[
        ("zoo", text("last")),
        ("apple", text("first")),
        ("banana", LogValue::Number(42)),
    ]);
    assert_eq!(
        compose_log_line(PREFIX, "msg", Some(&fields)),
        "[workflow-sdk] msg\n  apple  first\n  banana 42\n  zoo    last"
    );
}

#[test]
fn renders_attempt_and_retry_count_on_one_retry_row() {
    let fields = metadata(&[
        ("workflowRunId", text("wrun_01ABC")),
        ("workflowName", text("workflow//./workflows/x//myWorkflow")),
        ("stepId", text("step_01XYZ")),
        ("stepName", text("step//./workflows/x//add")),
        ("attempt", LogValue::Number(4)),
        ("retryCount", LogValue::Number(3)),
        ("errorAttribution", text("user")),
        ("errorName", text("Error")),
        ("errorMessage", text("Transient failure")),
    ]);
    assert_eq!(
        compose_log_line(
            PREFIX,
            "Step add (./workflows/x) hit max retries — bubbling error",
            Some(&fields),
        ),
        "[workflow-sdk] Step add (./workflows/x) hit max retries — bubbling error\n  user error · Error\n  run    wrun_01ABC · myWorkflow (./workflows/x)\n  step   step_01XYZ · add (./workflows/x)\n  retry  4 attempts · 3 max retries"
    );
}

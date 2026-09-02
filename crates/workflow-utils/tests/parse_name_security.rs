#![forbid(unsafe_code)]

use workflow_utils::{format_step_name, format_workflow_name};

fn contains_log_breaking_character(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(character, '\u{0000}'..='\u{001f}' | '\u{007f}'..='\u{009f}' | '\u{2028}' | '\u{2029}')
    })
}

#[test]
fn escapes_controls_in_parsed_function_and_module_names() {
    let formatted = format_step_name("step//./jobs/\u{001b}[31mred//run\r\nforged\t\u{2028}");

    assert_eq!(
        formatted,
        "run\\r\\nforged\\t\\u2028 (./jobs/\\u001b[31mred)"
    );
    assert!(!contains_log_breaking_character(&formatted));
}

#[test]
fn escapes_controls_in_unrecognized_fallback_names() {
    let formatted = format_workflow_name("legacy\nforged\u{001b}]8;;target\u{0007}");

    assert_eq!(formatted, "legacy\\nforged\\u001b]8;;target\\u0007");
    assert!(!contains_log_breaking_character(&formatted));
}

#[test]
fn preserves_ordinary_parsed_and_fallback_rendering() {
    assert_eq!(
        format_step_name("step//./jobs/order//run"),
        "run (./jobs/order)"
    );
    assert_eq!(
        format_workflow_name("legacy-workflow-name"),
        "legacy-workflow-name"
    );
}

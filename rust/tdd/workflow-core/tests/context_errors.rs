use workflow_core_tdd::context_errors::{
    ContextError, ContextErrorKind, make_context_error, redirected_caller_frame,
};

fn make(kind: ContextErrorKind, function_name: &str, docs: &str) -> ContextError {
    make_context_error(kind, function_name, docs, None)
}

#[test]
fn workflow_context_error_frames_the_function_name_and_docs_link() {
    let error = make(
        ContextErrorKind::NotInWorkflow,
        "createHook()",
        "https://workflow-sdk.dev/docs/api-reference/workflow/create-hook",
    );
    assert_eq!(error.name, "NotInWorkflowContextError");
    assert_eq!(
        error.message,
        "`createHook()` can only be called inside a workflow function\n╰▶ docs: https://workflow-sdk.dev/docs/api-reference/workflow/create-hook"
    );
}

#[test]
fn workflow_context_error_does_not_serialize_function_name_as_an_own_field() {
    let error = make(
        ContextErrorKind::NotInWorkflow,
        "createHook()",
        "https://example.com/docs",
    );
    assert!(!error.serializable_fields.contains_key("functionName"));
}

#[test]
fn step_context_error_uses_step_function_phrasing() {
    let error = make(
        ContextErrorKind::NotInStep,
        "getStepMetadata()",
        "https://workflow-sdk.dev/docs/api-reference/workflow/get-step-metadata",
    );
    assert!(
        error
            .message
            .contains("can only be called inside a step function")
    );
    assert!(
        error.message.contains(
            "docs: https://workflow-sdk.dev/docs/api-reference/workflow/get-step-metadata"
        )
    );
}

#[test]
fn workflow_or_step_error_uses_combined_context_phrasing() {
    let error = make(
        ContextErrorKind::NotInWorkflowOrStep,
        "getWorkflowMetadata()",
        "https://workflow-sdk.dev/docs/api-reference/workflow/get-workflow-metadata",
    );
    assert!(
        error
            .message
            .contains("can only be called inside a workflow or step function")
    );
}

#[test]
fn unavailable_error_names_the_active_workflow() {
    let error = make_context_error(
        ContextErrorKind::UnavailableInWorkflow,
        "resumeHook()",
        "https://workflow-sdk.dev/docs/api-reference/workflow-api/resume-hook",
        Some("workflow//./src/workflows/example.ts//myWorkflow"),
    );
    assert!(
        error
            .message
            .contains("cannot be called from a workflow context")
    );
    assert!(
        error
            .message
            .contains("workflow//./src/workflows/example.ts//myWorkflow")
    );
}

#[test]
fn unavailable_error_uses_generic_phrasing_without_an_active_workflow() {
    let error = make(
        ContextErrorKind::UnavailableInWorkflow,
        "resumeHook()",
        "https://workflow-sdk.dev/docs/api-reference/workflow-api/resume-hook",
    );
    assert!(error.message.contains("from a workflow context"));
}

#[test]
fn stored_message_contains_no_ansi_escape_bytes() {
    let error = make(
        ContextErrorKind::NotInWorkflow,
        "createHook()",
        "https://example.com/docs",
    );
    assert!(!error.message.contains("\u{1b}["));
}

#[test]
fn stored_stack_contains_no_ansi_escape_bytes() {
    let error = make(
        ContextErrorKind::NotInWorkflow,
        "createHook()",
        "https://example.com/docs",
    );
    assert!(!error.stack.contains("\u{1b}["));
}

#[test]
fn pretty_rendering_contains_the_framed_terminal_form() {
    let error = make(
        ContextErrorKind::NotInWorkflow,
        "createHook()",
        "https://example.com/docs",
    );
    assert!(error.pretty.contains("NotInWorkflowContextError:"));
    assert!(error.pretty.contains("createHook()"));
    assert!(
        error
            .pretty
            .contains("can only be called inside a workflow function")
    );
    assert!(error.pretty.contains("╰▶"));
    assert!(error.pretty.contains("docs:"));
}

#[test]
fn pretty_rendering_does_not_duplicate_framed_detail_lines() {
    let error = make(
        ContextErrorKind::NotInWorkflow,
        "createHook()",
        "https://example.com/docs",
    );
    assert_eq!(error.pretty.matches("╰▶ docs:").count(), 1);
}

#[test]
fn display_returns_the_pretty_framed_form() {
    let error = make(
        ContextErrorKind::NotInWorkflow,
        "createHook()",
        "https://example.com/docs",
    );
    let rendered = error.to_string();
    assert!(rendered.contains("NotInWorkflowContextError:"));
    assert!(rendered.contains("╰▶"));
}

macro_rules! fatal_context_case {
    ($name:ident, $kind:expr, $function:literal) => {
        #[test]
        fn $name() {
            let error = make($kind, $function, "https://example.com");
            assert!(error.fatal);
        }
    };
}

fatal_context_case!(
    workflow_context_errors_are_fatal,
    ContextErrorKind::NotInWorkflow,
    "createHook()"
);
fatal_context_case!(
    step_context_errors_are_fatal,
    ContextErrorKind::NotInStep,
    "getStepMetadata()"
);
fatal_context_case!(
    combined_context_errors_are_fatal,
    ContextErrorKind::NotInWorkflowOrStep,
    "getWorkflowMetadata()"
);
fatal_context_case!(
    unavailable_context_errors_are_fatal,
    ContextErrorKind::UnavailableInWorkflow,
    "resumeHook()"
);

#[test]
fn throw_helper_redirects_the_first_stack_frame_to_the_user_caller() {
    let frame = redirected_caller_frame();
    assert!(frame.contains("userCallSite"));
    assert!(!frame.contains("frameworkGate"));
    assert!(!frame.contains("throwNotInWorkflowContext"));
}

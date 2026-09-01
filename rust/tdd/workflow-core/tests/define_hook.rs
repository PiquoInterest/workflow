use workflow_core_tdd::define_hook::{
    ApprovalInput, ApprovalPayload, HookFieldValue, HookSchema, ResumeHookOutcome,
    resume_defined_hook,
};

fn input(approved: HookFieldValue, comment: HookFieldValue) -> ApprovalInput {
    ApprovalInput { approved, comment }
}

#[test]
fn passes_payload_through_when_no_schema_is_defined() {
    let outcome = resume_defined_hook(
        HookSchema::None,
        "token",
        input(
            HookFieldValue::Bool(true),
            HookFieldValue::String("Looks good".to_owned()),
        ),
    );
    assert_eq!(
        outcome,
        ResumeHookOutcome::Resumed {
            token: "token".to_owned(),
            payload: ApprovalPayload {
                approved: true,
                comment: "Looks good".to_owned(),
            },
        }
    );
}

#[test]
fn parses_and_normalizes_payload_before_resuming() {
    let outcome = resume_defined_hook(
        HookSchema::Approval,
        "token",
        input(
            HookFieldValue::Bool(true),
            HookFieldValue::String("  Ready!  ".to_owned()),
        ),
    );
    assert_eq!(
        outcome,
        ResumeHookOutcome::Resumed {
            token: "token".to_owned(),
            payload: ApprovalPayload {
                approved: true,
                comment: "Ready!".to_owned(),
            },
        }
    );
}

#[test]
fn rejects_invalid_payload_before_calling_resume() {
    let outcome = resume_defined_hook(
        HookSchema::Approval,
        "token",
        input(HookFieldValue::String("yes".to_owned()), HookFieldValue::Number(123)),
    );
    assert_eq!(
        outcome,
        ResumeHookOutcome::ValidationFailed {
            message: "Hook payload did not match the defined schema:\n  Invalid input: expected boolean at \"approved\"\n  Invalid input: expected string at \"comment\"".to_owned(),
        }
    );
}

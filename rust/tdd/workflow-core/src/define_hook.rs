fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/define-hook.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookFieldValue {
    Bool(bool),
    String(String),
    Number(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalInput {
    pub approved: HookFieldValue,
    pub comment: HookFieldValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalPayload {
    pub approved: bool,
    pub comment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookSchema {
    None,
    Approval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeHookOutcome {
    Resumed {
        token: String,
        payload: ApprovalPayload,
    },
    ValidationFailed {
        message: String,
    },
}

/// Validates and resumes one defined hook payload.
pub fn resume_defined_hook(
    schema: HookSchema,
    token: &str,
    input: ApprovalInput,
) -> ResumeHookOutcome {
    let _ = (schema, token, input);
    pending()
}

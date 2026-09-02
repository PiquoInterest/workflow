use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/context-errors.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextErrorKind {
    NotInWorkflow,
    NotInStep,
    NotInWorkflowOrStep,
    UnavailableInWorkflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextError {
    pub name: String,
    pub message: String,
    pub stack: String,
    pub pretty: String,
    pub serializable_fields: BTreeMap<String, String>,
    pub fatal: bool,
}

impl Display for ContextError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.pretty)
    }
}

impl Error for ContextError {}

/// Builds a context violation with plain stored state and lazy pretty output.
pub fn make_context_error(
    kind: ContextErrorKind,
    function_name: &str,
    docs_url: &str,
    active_workflow: Option<&str>,
) -> ContextError {
    let _ = (kind, function_name, docs_url, active_workflow);
    pending()
}

/// Returns the first stack frame after the framework helper redirects capture.
pub fn redirected_caller_frame() -> String {
    pending()
}

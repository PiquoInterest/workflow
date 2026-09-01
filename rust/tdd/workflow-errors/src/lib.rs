#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CauseToken(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticValue {
    Text(String),
    Integer(u64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ErrorDescriptor {
    pub name: String,
    pub message: String,
    pub hint: Option<String>,
    pub cause: Option<CauseToken>,
    pub context: Option<BTreeMap<String, DiagnosticValue>>,
    pub event_id: Option<String>,
    pub fatal: bool,
    pub hierarchy: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValue {
    Missing,
    Bool(bool),
    Integer(i64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardCandidate {
    Workflow(ErrorDescriptor),
    ForeignError {
        name: String,
        fatal: PropertyValue,
        fields: BTreeSet<String>,
    },
    NonError,
}

pub fn workflow_error(
    message: &str,
    slug: Option<&str>,
    cause: Option<CauseToken>,
) -> ErrorDescriptor {
    let _ = (message, slug, cause);
    panic!("TDD RED: packages/errors/src/workflow-error.test.ts implementation pending")
}

pub fn workflow_build_error(
    message: &str,
    hint: Option<&str>,
    cause: Option<CauseToken>,
) -> ErrorDescriptor {
    let _ = (message, hint, cause);
    panic!("TDD RED: packages/errors/src/build-error.test.ts implementation pending")
}

pub fn corrupted_event_log_error(
    message: &str,
    cause: Option<CauseToken>,
) -> ErrorDescriptor {
    let _ = (message, cause);
    panic!(
        "TDD RED: packages/errors/src/corrupted-event-log-error.test.ts implementation pending"
    )
}

pub fn replay_divergence_error(message: &str, event_id: &str) -> ErrorDescriptor {
    let _ = (message, event_id);
    panic!(
        "TDD RED: packages/errors/src/replay-divergence-error.test.ts implementation pending"
    )
}

pub fn runtime_decryption_error(
    message: &str,
    cause: Option<CauseToken>,
    context: Option<BTreeMap<String, DiagnosticValue>>,
) -> ErrorDescriptor {
    let _ = (message, cause, context);
    panic!(
        "TDD RED: packages/errors/src/runtime-decryption-error.test.ts implementation pending"
    )
}

pub fn serialization_error(
    message: &str,
    hint: Option<&str>,
    cause: Option<CauseToken>,
) -> ErrorDescriptor {
    let _ = (message, hint, cause);
    panic!("TDD RED: packages/errors/src/serialization-error.test.ts implementation pending")
}

pub fn fatal_error(message: &str) -> ErrorDescriptor {
    let _ = message;
    panic!("TDD RED: packages/errors/src/fatal-error.test.ts implementation pending")
}

#[must_use]
pub fn is_named_error(candidate: &GuardCandidate, name: &str) -> bool {
    let _ = (candidate, name);
    false
}

#[must_use]
pub fn is_replay_divergence(candidate: &GuardCandidate) -> bool {
    let _ = candidate;
    false
}

#[must_use]
pub fn is_fatal(candidate: &GuardCandidate) -> bool {
    let _ = candidate;
    panic!("TDD RED: packages/errors/src/fatal-error.test.ts implementation pending")
}

pub fn scan_module_scope_state(package_path: &str) -> Vec<String> {
    let _ = package_path;
    panic!(
        "TDD RED: packages/errors/src/module-scope-state.test.ts implementation pending"
    )
}

pub mod ansi {
    #[must_use]
    pub fn frame(title: &str, contents: &[&str]) -> String {
        let _ = (title, contents);
        panic!("TDD RED: packages/errors/src/ansi.test.ts implementation pending")
    }

    #[must_use]
    pub fn code(token: &str) -> String {
        let _ = token;
        panic!("TDD RED: packages/errors/src/ansi.test.ts implementation pending")
    }

    #[must_use]
    pub fn hint(text: &str) -> String {
        let _ = text;
        panic!("TDD RED: packages/errors/src/ansi.test.ts implementation pending")
    }

    #[must_use]
    pub fn note(lines: &[&str]) -> String {
        let _ = lines;
        panic!("TDD RED: packages/errors/src/ansi.test.ts implementation pending")
    }

    #[must_use]
    pub fn help(text: &str) -> String {
        let _ = text;
        panic!("TDD RED: packages/errors/src/ansi.test.ts implementation pending")
    }

    #[must_use]
    pub fn docs(url: &str) -> String {
        let _ = url;
        panic!("TDD RED: packages/errors/src/ansi.test.ts implementation pending")
    }

    #[must_use]
    pub fn inline_annotation(
        source: &str,
        token_start: usize,
        token_len: usize,
        explanation: &str,
    ) -> String {
        let _ = (source, token_start, token_len, explanation);
        panic!("TDD RED: packages/errors/src/ansi.test.ts implementation pending")
    }
}

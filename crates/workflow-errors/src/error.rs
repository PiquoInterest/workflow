use std::collections::{BTreeMap, BTreeSet};

pub const WORKFLOW_ERROR_DOCS_URL: &str = "https://workflow-sdk.dev/err";

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

#[must_use]
pub fn workflow_error(
    message: &str,
    slug: Option<&str>,
    cause: Option<CauseToken>,
) -> ErrorDescriptor {
    make_descriptor(DescriptorParts {
        name: "WorkflowError",
        message: append_framed_details(message, None, slug),
        cause,
        hierarchy: &["WorkflowError"],
        ..DescriptorParts::default()
    })
}

#[must_use]
pub fn workflow_build_error(
    message: &str,
    hint: Option<&str>,
    cause: Option<CauseToken>,
) -> ErrorDescriptor {
    make_descriptor(DescriptorParts {
        name: "WorkflowBuildError",
        message: append_framed_details(message, hint, None),
        hint: hint.map(ToOwned::to_owned),
        cause,
        hierarchy: &["WorkflowError", "WorkflowBuildError"],
        ..DescriptorParts::default()
    })
}

#[must_use]
pub fn corrupted_event_log_error(message: &str, cause: Option<CauseToken>) -> ErrorDescriptor {
    make_descriptor(DescriptorParts {
        name: "CorruptedEventLogError",
        message: append_framed_details(message, None, Some("corrupted-event-log")),
        cause,
        hierarchy: &[
            "WorkflowError",
            "WorkflowRuntimeError",
            "CorruptedEventLogError",
        ],
        ..DescriptorParts::default()
    })
}

#[must_use]
pub fn replay_divergence_error(message: &str, event_id: &str) -> ErrorDescriptor {
    make_descriptor(DescriptorParts {
        name: "ReplayDivergenceError",
        message: append_framed_details(message, None, Some("replay-divergence")),
        event_id: Some(event_id.to_owned()),
        hierarchy: &[
            "WorkflowError",
            "WorkflowRuntimeError",
            "ReplayDivergenceError",
        ],
        ..DescriptorParts::default()
    })
}

#[must_use]
pub fn runtime_decryption_error(
    message: &str,
    cause: Option<CauseToken>,
    context: Option<BTreeMap<String, DiagnosticValue>>,
) -> ErrorDescriptor {
    make_descriptor(DescriptorParts {
        name: "RuntimeDecryptionError",
        message: append_framed_details(message, None, Some("runtime-decryption-failed")),
        cause,
        context,
        hierarchy: &[
            "WorkflowError",
            "WorkflowRuntimeError",
            "RuntimeDecryptionError",
        ],
        ..DescriptorParts::default()
    })
}

#[must_use]
pub fn serialization_error(
    message: &str,
    hint: Option<&str>,
    cause: Option<CauseToken>,
) -> ErrorDescriptor {
    make_descriptor(DescriptorParts {
        name: "SerializationError",
        message: append_framed_details(message, hint, None),
        hint: hint.map(ToOwned::to_owned),
        cause,
        fatal: true,
        hierarchy: &["WorkflowError", "SerializationError"],
        ..DescriptorParts::default()
    })
}

#[must_use]
pub fn fatal_error(message: &str) -> ErrorDescriptor {
    make_descriptor(DescriptorParts {
        name: "FatalError",
        message: message.to_owned(),
        fatal: true,
        hierarchy: &["FatalError"],
        ..DescriptorParts::default()
    })
}

#[must_use]
pub fn is_named_error(candidate: &GuardCandidate, name: &str) -> bool {
    match candidate {
        GuardCandidate::Workflow(error) => error.name == name,
        GuardCandidate::ForeignError {
            name: foreign_name, ..
        } => foreign_name == name,
        GuardCandidate::NonError => false,
    }
}

#[must_use]
pub fn is_replay_divergence(candidate: &GuardCandidate) -> bool {
    match candidate {
        GuardCandidate::Workflow(error) => {
            error.name == "ReplayDivergenceError" && error.event_id.is_some()
        }
        GuardCandidate::ForeignError { name, fields, .. } => {
            name == "ReplayDivergenceError" && fields.contains("eventId")
        }
        GuardCandidate::NonError => false,
    }
}

#[must_use]
pub fn is_fatal(candidate: &GuardCandidate) -> bool {
    match candidate {
        GuardCandidate::Workflow(error) => error.name == "FatalError" || error.fatal,
        GuardCandidate::ForeignError { name, fatal, .. } => {
            name == "FatalError" || matches!(fatal, PropertyValue::Bool(true))
        }
        GuardCandidate::NonError => false,
    }
}

#[derive(Default)]
struct DescriptorParts {
    name: &'static str,
    message: String,
    hint: Option<String>,
    cause: Option<CauseToken>,
    context: Option<BTreeMap<String, DiagnosticValue>>,
    event_id: Option<String>,
    fatal: bool,
    hierarchy: &'static [&'static str],
}

fn make_descriptor(parts: DescriptorParts) -> ErrorDescriptor {
    ErrorDescriptor {
        name: parts.name.to_owned(),
        message: parts.message,
        hint: parts.hint,
        cause: parts.cause,
        context: parts.context,
        event_id: parts.event_id,
        fatal: parts.fatal,
        hierarchy: parts
            .hierarchy
            .iter()
            .copied()
            .map(str::to_owned)
            .collect(),
    }
}

fn append_framed_details(message: &str, hint: Option<&str>, slug: Option<&str>) -> String {
    let mut details = Vec::with_capacity(2);
    if let Some(hint) = hint.filter(|value| !value.is_empty()) {
        details.push(("hint", hint.to_owned()));
    }
    if let Some(slug) = slug.filter(|value| !value.is_empty()) {
        details.push(("docs", format!("{WORKFLOW_ERROR_DOCS_URL}/{slug}")));
    }

    let mut result = message.to_owned();
    for (index, (label, value)) in details.iter().enumerate() {
        let is_last = index + 1 == details.len();
        let first_prefix = if is_last { "╰▶ " } else { "├▶ " };
        let continuation_prefix = if is_last { "   " } else { "│  " };
        let detail = format!("{label}: {value}");

        for (line_index, line) in detail.split('\n').enumerate() {
            result.push('\n');
            result.push_str(if line_index == 0 {
                first_prefix
            } else {
                continuation_prefix
            });
            result.push_str(line);
        }
    }
    result
}

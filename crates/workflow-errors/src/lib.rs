#![forbid(unsafe_code)]

pub mod ansi;
mod error;

pub use error::{
    CauseToken, DiagnosticValue, ErrorDescriptor, GuardCandidate, PropertyValue,
    WORKFLOW_ERROR_DOCS_URL, corrupted_event_log_error, fatal_error, is_fatal, is_named_error,
    is_replay_divergence, replay_divergence_error, runtime_decryption_error, serialization_error,
    workflow_build_error, workflow_error,
};

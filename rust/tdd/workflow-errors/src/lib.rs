#![forbid(unsafe_code)]

pub use workflow_errors::{
    CauseToken, DiagnosticValue, ErrorDescriptor, GuardCandidate, PropertyValue,
    WORKFLOW_ERROR_DOCS_URL, ansi, corrupted_event_log_error, fatal_error, is_fatal,
    is_named_error, is_replay_divergence, replay_divergence_error, runtime_decryption_error,
    serialization_error, workflow_build_error, workflow_error,
};

pub fn scan_module_scope_state(package_path: &str) -> Vec<String> {
    let _ = package_path;
    panic!("TDD RED: packages/errors/src/module-scope-state.test.ts implementation pending")
}

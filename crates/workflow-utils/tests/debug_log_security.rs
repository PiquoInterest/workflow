#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use workflow_utils::{DebugArgument, DebugSink, debug_log, is_workflow_debug_enabled};

#[derive(Default)]
struct RecordingSink {
    log_calls: usize,
    debug_calls: usize,
    info_calls: usize,
    warn_calls: usize,
    error_calls: usize,
}

impl RecordingSink {
    fn total_calls(&self) -> usize {
        self.log_calls + self.debug_calls + self.info_calls + self.warn_calls + self.error_calls
    }
}

impl DebugSink for RecordingSink {
    fn log(&mut self, _arguments: &[DebugArgument]) {
        self.log_calls += 1;
    }

    fn debug(&mut self, _arguments: &[DebugArgument]) {
        self.debug_calls += 1;
    }

    fn info(&mut self, _arguments: &[DebugArgument]) {
        self.info_calls += 1;
    }

    fn warn(&mut self, _arguments: &[DebugArgument]) {
        self.warn_calls += 1;
    }

    fn error(&mut self, _arguments: &[DebugArgument]) {
        self.error_calls += 1;
    }
}

#[test]
fn rejects_substring_and_explicitly_negative_selectors() {
    for selector in [
        "myworkflow:*",
        "app:*,-workflow:*",
        "workflow:*,-workflow:*",
        "*,-workflow:*",
        "-workflow:*,workflow:*",
    ] {
        assert!(
            !is_workflow_debug_enabled(Some(selector)),
            "selector must not enable workflow diagnostics: {selector}"
        );
    }
}

#[test]
fn disabled_selectors_never_invoke_the_sink() {
    let arguments = [
        DebugArgument::Text("diagnostic breadcrumb".to_owned()),
        DebugArgument::Fields(BTreeMap::from([(
            "runId".to_owned(),
            "wrun_private".to_owned(),
        )])),
    ];

    for selector in ["myworkflow:*", "app:*,-workflow:*"] {
        let mut sink = RecordingSink::default();
        debug_log(Some(selector), &arguments, &mut sink);
        assert_eq!(sink.total_calls(), 0, "unexpected sink call for {selector}");
    }
}

#[test]
fn debug_output_does_not_echo_diagnostic_values() {
    let text = DebugArgument::Text("credential-like-value".to_owned());
    let fields = DebugArgument::Fields(BTreeMap::from([(
        "authorization".to_owned(),
        "Bearer highly-sensitive-token".to_owned(),
    )]));

    let rendered = format!("{text:?} {fields:?}");

    assert!(!rendered.contains("credential-like-value"));
    assert!(!rendered.contains("authorization"));
    assert!(!rendered.contains("highly-sensitive-token"));
    assert!(rendered.contains("<redacted>"));
}

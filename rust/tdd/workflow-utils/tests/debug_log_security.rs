use std::collections::BTreeMap;
use workflow_utils_tdd::{DebugArgument, DebugSink, debug_log, is_workflow_debug_enabled};

#[derive(Debug, Default)]
struct RecordingSink {
    debug: Vec<Vec<DebugArgument>>,
}

impl DebugSink for RecordingSink {
    fn log(&mut self, _arguments: &[DebugArgument]) {}

    fn debug(&mut self, arguments: &[DebugArgument]) {
        self.debug.push(arguments.to_vec());
    }

    fn info(&mut self, _arguments: &[DebugArgument]) {}

    fn warn(&mut self, _arguments: &[DebugArgument]) {}

    fn error(&mut self, _arguments: &[DebugArgument]) {}
}

#[test]
fn rejects_unrelated_and_negative_selector_tokens() {
    for selector in ["myworkflow:*", "app:*,-workflow:*"] {
        assert!(
            !is_workflow_debug_enabled(Some(selector)),
            "selector must not enable workflow diagnostics: {selector}"
        );
    }
}

#[test]
fn rejected_selectors_never_forward_sensitive_diagnostics() {
    let arguments = [
        DebugArgument::Text("diagnostic breadcrumb".to_owned()),
        DebugArgument::Fields(BTreeMap::from([(
            "runId".to_owned(),
            "wrun_private".to_owned(),
        )])),
    ];
    let mut sink = RecordingSink::default();

    debug_log(Some("app:*,-workflow:*"), &arguments, &mut sink);

    assert!(sink.debug.is_empty());
}

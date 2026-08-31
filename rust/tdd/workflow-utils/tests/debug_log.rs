use std::collections::BTreeMap;
use workflow_utils_tdd::{
    DebugArgument, DebugSink, debug_log, is_workflow_debug_enabled,
};

#[derive(Debug, Default)]
struct RecordingSink {
    log: Vec<Vec<DebugArgument>>,
    debug: Vec<Vec<DebugArgument>>,
    info: Vec<Vec<DebugArgument>>,
    warn: Vec<Vec<DebugArgument>>,
    error: Vec<Vec<DebugArgument>>,
}

impl DebugSink for RecordingSink {
    fn log(&mut self, arguments: &[DebugArgument]) {
        self.log.push(arguments.to_vec());
    }

    fn debug(&mut self, arguments: &[DebugArgument]) {
        self.debug.push(arguments.to_vec());
    }

    fn info(&mut self, arguments: &[DebugArgument]) {
        self.info.push(arguments.to_vec());
    }

    fn warn(&mut self, arguments: &[DebugArgument]) {
        self.warn.push(arguments.to_vec());
    }

    fn error(&mut self, arguments: &[DebugArgument]) {
        self.error.push(arguments.to_vec());
    }
}

#[test]
fn debug_is_off_when_the_selector_is_unset_or_empty() {
    assert!(!is_workflow_debug_enabled(None));
    assert!(!is_workflow_debug_enabled(Some("")));
}

#[test]
fn accepts_selectors_a_namespaced_logger_would_match() {
    for selector in [
        "workflow:*",
        "*",
        "workflow:runtime:debug",
        "app:*,workflow:world-vercel:*",
    ] {
        assert!(
            is_workflow_debug_enabled(Some(selector)),
            "selector should enable workflow debugging: {selector}"
        );
    }
}

#[test]
fn ignores_another_librarys_debug_selector() {
    assert!(!is_workflow_debug_enabled(Some("express:*")));
}

#[test]
fn re_evaluates_the_selector_on_every_call() {
    assert!(!is_workflow_debug_enabled(Some("")));
    assert!(is_workflow_debug_enabled(Some("workflow:*")));
}

#[test]
fn writes_nothing_when_debugging_is_disabled() {
    let mut sink = RecordingSink::default();
    let arguments = [
        DebugArgument::Text("a breadcrumb".to_owned()),
        DebugArgument::Fields(BTreeMap::from([(
            "runId".to_owned(),
            "wrun_1".to_owned(),
        )])),
    ];

    debug_log(Some(""), &arguments, &mut sink);

    assert!(sink.log.is_empty());
    assert!(sink.debug.is_empty());
    assert!(sink.info.is_empty());
    assert!(sink.warn.is_empty());
    assert!(sink.error.is_empty());
}

#[test]
fn forwards_every_argument_to_the_debug_sink_when_enabled() {
    let mut sink = RecordingSink::default();
    let arguments = vec![
        DebugArgument::Text("a breadcrumb".to_owned()),
        DebugArgument::Fields(BTreeMap::from([(
            "runId".to_owned(),
            "wrun_1".to_owned(),
        )])),
    ];

    debug_log(Some("workflow:*"), &arguments, &mut sink);

    assert_eq!(sink.debug, vec![arguments]);
    assert!(sink.log.is_empty());
    assert!(sink.info.is_empty());
    assert!(sink.warn.is_empty());
    assert!(sink.error.is_empty());
}

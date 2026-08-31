use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugArgument {
    Text(String),
    Fields(BTreeMap<String, String>),
}

pub trait DebugSink {
    fn log(&mut self, arguments: &[DebugArgument]);
    fn debug(&mut self, arguments: &[DebugArgument]);
    fn info(&mut self, arguments: &[DebugArgument]);
    fn warn(&mut self, arguments: &[DebugArgument]);
    fn error(&mut self, arguments: &[DebugArgument]);
}

#[must_use]
pub fn is_workflow_debug_enabled(debug_selector: Option<&str>) -> bool {
    let _ = debug_selector;
    panic!("TDD RED: packages/utils/src/debug-log.test.ts implementation pending")
}

pub fn debug_log(
    debug_selector: Option<&str>,
    arguments: &[DebugArgument],
    sink: &mut dyn DebugSink,
) {
    let _ = (debug_selector, arguments, sink);
    panic!("TDD RED: packages/utils/src/debug-log.test.ts implementation pending")
}

use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum DebugArgument {
    Text(String),
    Fields(BTreeMap<String, String>),
}

impl fmt::Debug for DebugArgument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(_) => formatter.write_str("Text(<redacted>)"),
            Self::Fields(fields) => formatter
                .debug_struct("Fields")
                .field("len", &fields.len())
                .finish(),
        }
    }
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
    let Some(selector) = debug_selector else {
        return false;
    };

    let mut enabled = false;
    for token in selector
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|token| !token.is_empty())
    {
        if token == "-*" || token.starts_with("-workflow:") {
            return false;
        }
        if token == "*" || token.starts_with("workflow:") {
            enabled = true;
        }
    }
    enabled
}

pub fn debug_log(
    debug_selector: Option<&str>,
    arguments: &[DebugArgument],
    sink: &mut dyn DebugSink,
) {
    if !is_workflow_debug_enabled(debug_selector) {
        return;
    }
    sink.debug(arguments);
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogValue {
    Text(String),
    Number(i64),
}

pub type LogMetadata = BTreeMap<String, LogValue>;

/// Composes one structured, human-readable runtime log line.
pub fn compose_log_line(prefix: &str, message: &str, metadata: Option<&LogMetadata>) -> String {
    let _ = (prefix, message, metadata);
    panic!("TDD RED: packages/core/src/log-format.test.ts implementation pending")
}

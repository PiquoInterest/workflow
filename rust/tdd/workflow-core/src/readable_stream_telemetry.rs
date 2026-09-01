use std::collections::BTreeMap;

fn pending<T>() -> T {
    panic!(
        "TDD RED: packages/core/src/readable-stream-telemetry.test.ts implementation pending"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamReadKind {
    WorkflowServerReadable,
    ReconnectingFramed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamReadFixture {
    pub kind: StreamReadKind,
    pub run_id: String,
    pub stream_name: String,
    /// Chunks returned by the World stream. Framed reads include each frame's
    /// four-byte big-endian length prefix in this byte vector.
    pub transport_chunks: Vec<Vec<u8>>,
    pub connect_ms: u64,
    pub ttfc_ms: u64,
    pub total_ms: u64,
    pub reconnects: Option<u64>,
}

impl StreamReadFixture {
    pub fn workflow_server(
        run_id: &str,
        stream_name: &str,
        transport_chunks: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            kind: StreamReadKind::WorkflowServerReadable,
            run_id: run_id.to_owned(),
            stream_name: stream_name.to_owned(),
            transport_chunks,
            connect_ms: 2,
            ttfc_ms: 4,
            total_ms: 6,
            reconnects: None,
        }
    }

    pub fn reconnecting_framed(
        run_id: &str,
        stream_name: &str,
        transport_chunks: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            kind: StreamReadKind::ReconnectingFramed,
            run_id: run_id.to_owned(),
            stream_name: stream_name.to_owned(),
            transport_chunks,
            connect_ms: 2,
            ttfc_ms: 4,
            total_ms: 6,
            reconnects: Some(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanAttribute {
    Text(String),
    Integer(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedSpan {
    pub name: String,
    pub kind: SpanKind,
    pub attributes: BTreeMap<String, SpanAttribute>,
}

impl FinishedSpan {
    pub fn text(&self, key: &str) -> Option<&str> {
        match self.attributes.get(key) {
            Some(SpanAttribute::Text(value)) => Some(value),
            _ => None,
        }
    }

    pub fn integer(&self, key: &str) -> Option<u64> {
        match self.attributes.get(key) {
            Some(SpanAttribute::Integer(value)) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamReadObservation {
    pub delivered_chunks: Vec<Vec<u8>>,
    pub spans: Vec<FinishedSpan>,
}

impl StreamReadObservation {
    pub fn span(&self, name: &str) -> Option<&FinishedSpan> {
        self.spans.iter().find(|span| span.name == name)
    }
}

/// Drives the future Rust readable-stream path and captures its finished spans.
pub fn observe_stream_read(fixture: &StreamReadFixture) -> StreamReadObservation {
    let _ = fixture;
    pending()
}

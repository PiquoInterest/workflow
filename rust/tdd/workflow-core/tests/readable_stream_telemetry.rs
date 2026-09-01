use workflow_core_tdd::readable_stream_telemetry::{
    SpanKind, StreamReadFixture, observe_stream_read,
};

fn frame(payload: &[u8]) -> Vec<u8> {
    let length = u32::try_from(payload.len()).expect("fixture payload length fits in u32");
    let mut framed = Vec::with_capacity(4 + payload.len());
    framed.extend_from_slice(&length.to_be_bytes());
    framed.extend_from_slice(payload);
    framed
}

#[test]
fn workflow_server_read_emits_client_span_with_ttfc_and_connect_durations() {
    let fixture = StreamReadFixture::workflow_server("run-123", "test-stream", vec![vec![1, 2, 3]]);
    let observation = observe_stream_read(&fixture);
    let span = observation
        .span("workflow.stream.read")
        .expect("read span should finish after the first non-empty chunk");

    assert_eq!(span.kind, SpanKind::Client);
    assert_eq!(span.text("workflow.run.id"), Some("run-123"));
    assert_eq!(span.text("workflow.stream.name"), Some("test-stream"));
    assert_eq!(span.text("workflow.stream.operation"), Some("read"));

    let ttfc = span
        .integer("workflow.stream.read.ttfc_ms")
        .expect("TTFC must be numeric");
    let connect = span
        .integer("workflow.stream.read.connect_ms")
        .expect("connect duration must be numeric");
    assert_eq!(ttfc, fixture.ttfc_ms);
    assert_eq!(connect, fixture.connect_ms);
    assert!(connect <= ttfc.saturating_add(1));
}

#[test]
fn workflow_server_read_complete_span_reports_drained_payload_totals() {
    let fixture = StreamReadFixture::workflow_server("run-123", "test-stream", vec![vec![1, 2, 3]]);
    let observation = observe_stream_read(&fixture);
    let span = observation
        .span("workflow.stream.read.complete")
        .expect("completion span should finish only after the read drains");

    assert_eq!(span.kind, SpanKind::Client);
    assert_eq!(
        span.text("workflow.stream.operation"),
        Some("read_complete")
    );
    assert_eq!(span.integer("workflow.stream.read.chunks"), Some(1));
    assert_eq!(span.integer("workflow.stream.read.bytes"), Some(3));
    assert_eq!(
        span.integer("workflow.stream.read.total_ms"),
        Some(fixture.total_ms)
    );
    assert_eq!(span.integer("workflow.stream.read.reconnects"), None);
}

#[test]
fn reconnecting_framed_read_reports_wire_bytes_and_zero_reconnects() {
    let fixture = StreamReadFixture::reconnecting_framed(
        "run-123",
        "test-stream",
        vec![frame(&[1, 2, 3]), frame(&[4, 5])],
    );
    let observation = observe_stream_read(&fixture);

    let read_span = observation
        .span("workflow.stream.read")
        .expect("framed read should emit a first-chunk span");
    assert_eq!(
        read_span.integer("workflow.stream.read.ttfc_ms"),
        Some(fixture.ttfc_ms)
    );
    assert_eq!(
        read_span.integer("workflow.stream.read.connect_ms"),
        Some(fixture.connect_ms)
    );

    let complete_span = observation
        .span("workflow.stream.read.complete")
        .expect("framed read should emit a completion span after drain");
    assert_eq!(
        complete_span.integer("workflow.stream.read.chunks"),
        Some(2)
    );
    assert_eq!(
        complete_span.integer("workflow.stream.read.bytes"),
        Some(13)
    );
    assert_eq!(
        complete_span.integer("workflow.stream.read.reconnects"),
        Some(0)
    );
}

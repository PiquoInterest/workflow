use workflow_world_tdd::node_http::{
    BodyObservation, ContentCoding, NodeHttpCase, NodeHttpError, NodeHttpObservation,
    ResponseObservation, exercise_node_http_case,
};

fn observation(case: NodeHttpCase) -> NodeHttpObservation {
    exercise_node_http_case(case).unwrap()
}

fn failure(case: NodeHttpCase) -> NodeHttpError {
    exercise_node_http_case(case).unwrap_err()
}

fn response(observation: &NodeHttpObservation) -> &ResponseObservation {
    observation.response.as_ref().expect("expected response")
}

fn body(response: &ResponseObservation) -> &BodyObservation {
    response.body.as_ref().expect("expected response body")
}

#[test]
fn round_trips_a_get_and_exposes_the_fetch_response_surface() {
    let observation = observation(NodeHttpCase::GetResponseSurface);
    let response = response(&observation);

    assert!(response.ok);
    assert_eq!(response.status, 201);
    assert_eq!(response.status_text, "Created");
    assert_eq!(
        response.headers.get("x-workflow-test").map(String::as_str),
        Some("present")
    );
    assert_eq!(body(response).bytes, br#"{"hello":"world"}"#);
}

#[test]
fn sends_method_headers_and_a_buffered_body() {
    let observation = observation(NodeHttpCase::BufferedPost);

    assert_eq!(observation.request.method, "POST");
    assert_eq!(
        observation
            .request
            .headers
            .get("x-custom")
            .map(String::as_str),
        Some("value")
    );
    assert_eq!(observation.request.body, b"payload");
}

#[test]
fn declares_content_length_instead_of_chunking_the_body() {
    let observation = observation(NodeHttpCase::ContentLength);

    assert_eq!(observation.request.content_length, Some(12));
    assert_eq!(observation.request.transfer_encoding, None);
}

#[test]
fn advertises_and_decodes_gzip() {
    let observation = observation(NodeHttpCase::Decode(ContentCoding::Gzip));
    let response = response(&observation);

    assert!(
        observation
            .request
            .headers
            .get("accept-encoding")
            .is_some_and(|value| value.contains("gzip"))
    );
    assert_eq!(body(response).bytes, b"compressed payload");
    assert!(!response.headers.contains_key("content-encoding"));
    assert!(!response.headers.contains_key("content-length"));
}

#[test]
fn decodes_brotli() {
    let observation = observation(NodeHttpCase::Decode(ContentCoding::Brotli));
    assert_eq!(body(response(&observation)).bytes, b"brotli payload");
}

#[test]
fn decodes_zlib_wrapped_and_raw_deflate() {
    for coding in [ContentCoding::DeflateZlib, ContentCoding::DeflateRaw] {
        let observation = observation(NodeHttpCase::Decode(coding));
        assert_eq!(body(response(&observation)).bytes, b"deflated payload");
    }
}

#[test]
fn decodes_a_compressed_body_whose_trailer_never_arrives() {
    let observation = observation(NodeHttpCase::TruncatedGzipTrailer);
    assert_eq!(body(response(&observation)).bytes, b"hello gzip body");
}

#[test]
fn streams_the_body_incrementally_instead_of_buffering_it() {
    let observation = observation(NodeHttpCase::IncrementalBody);
    let body = body(response(&observation));

    assert!(body.was_incremental);
    assert_eq!(body.chunks, vec![b"first".to_vec(), b"second".to_vec()]);
}

#[test]
fn resolves_null_body_statuses_without_a_body() {
    let observation = observation(NodeHttpCase::NoContent);
    let response = response(&observation);

    assert_eq!(response.status, 204);
    assert!(response.body.is_none());
}

#[test]
fn resolves_a_head_request_without_a_body() {
    let observation = observation(NodeHttpCase::HeadResponse);
    let response = response(&observation);

    assert_eq!(response.status, 200);
    assert!(response.body.is_none());
    assert_eq!(
        response.headers.get("content-length").map(String::as_str),
        Some("42")
    );
}

#[test]
fn rejects_with_the_original_abort_reason_mid_flight() {
    let error = failure(NodeHttpCase::AbortBeforeHeaders);

    assert_eq!(error.kind, "AbortError");
    assert!(error.preserved_abort_reason);
}

#[test]
fn errors_the_body_stream_with_the_original_abort_reason() {
    let observation = observation(NodeHttpCase::AbortDuringBody);
    assert!(body(response(&observation)).preserved_abort_reason);
}

#[test]
fn rejects_immediately_when_the_signal_is_already_aborted() {
    let error = failure(NodeHttpCase::AlreadyAborted);
    assert!(error.preserved_abort_reason);
}

#[test]
fn raises_etimedout_when_headers_do_not_arrive_in_time() {
    let error = failure(NodeHttpCase::HeaderTimeout);
    assert_eq!(error.code.as_deref(), Some("ETIMEDOUT"));
}

#[test]
fn errors_the_body_stream_when_it_stalls_past_the_body_timeout() {
    let observation = observation(NodeHttpCase::BodyTimeout);
    assert_eq!(
        body(response(&observation)).error_code.as_deref(),
        Some("ETIMEDOUT")
    );
}

#[test]
fn errors_the_body_stream_when_the_socket_drops_mid_body() {
    let observation = observation(NodeHttpCase::SocketDrop(ContentCoding::Identity));
    assert_eq!(
        body(response(&observation)).error_code.as_deref(),
        Some("ECONNRESET")
    );
}

#[test]
fn errors_the_body_stream_when_the_socket_drops_mid_compressed_body() {
    let observation = observation(NodeHttpCase::SocketDrop(ContentCoding::Gzip));
    assert_eq!(
        body(response(&observation)).error_code.as_deref(),
        Some("ECONNRESET")
    );
}

#[test]
fn reports_the_body_deadline_instead_of_a_reset_on_compressed_bodies() {
    let observation = observation(NodeHttpCase::CompressedBodyTimeout);
    assert_eq!(
        body(response(&observation)).error_code.as_deref(),
        Some("ETIMEDOUT")
    );
}

#[test]
fn rejects_a_connection_that_cannot_be_established() {
    let error = failure(NodeHttpCase::ConnectionFailure);
    assert!(matches!(
        error.code.as_deref(),
        Some("ECONNREFUSED" | "EACCES" | "ECONNRESET")
    ));
}

#[test]
fn refuses_a_protocol_it_cannot_speak() {
    let error = failure(NodeHttpCase::UnsupportedProtocol);
    assert_eq!(error.kind, "TypeError");
}

#[test]
fn releases_identity_and_gzip_sockets_when_the_body_is_cancelled_mid_read() {
    for coding in [ContentCoding::Identity, ContentCoding::Gzip] {
        let observation = observation(NodeHttpCase::CancelBody(coding));
        assert!(observation.pool.socket_released_after_cancel);
        assert_eq!(observation.pool.next_request_status, Some(200));
    }
}

#[test]
fn does_not_spend_the_header_deadline_waiting_for_a_socket() {
    let observation = observation(NodeHttpCase::QueuedBehindBusySocket);

    assert!(!observation.pool.queued_request_timed_out);
    assert_eq!(observation.pool.next_request_status, Some(200));
    assert_eq!(body(response(&observation)).bytes, b"ok");
}

#[test]
fn reuses_a_keep_alive_socket_across_requests_on_the_shared_pool() {
    let observation = observation(NodeHttpCase::KeepAliveReuse);
    assert_eq!(observation.pool.unique_socket_count, 1);
}

use std::collections::BTreeMap;

pub type HeaderMap = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentCoding {
    Identity,
    Gzip,
    Brotli,
    DeflateZlib,
    DeflateRaw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeHttpCase {
    GetResponseSurface,
    BufferedPost,
    ContentLength,
    Decode(ContentCoding),
    TruncatedGzipTrailer,
    IncrementalBody,
    NoContent,
    HeadResponse,
    AbortBeforeHeaders,
    AbortDuringBody,
    AlreadyAborted,
    HeaderTimeout,
    BodyTimeout,
    SocketDrop(ContentCoding),
    CompressedBodyTimeout,
    ConnectionFailure,
    UnsupportedProtocol,
    CancelBody(ContentCoding),
    QueuedBehindBusySocket,
    KeepAliveReuse,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequestObservation {
    pub method: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub content_length: Option<u64>,
    pub transfer_encoding: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BodyObservation {
    pub bytes: Vec<u8>,
    pub chunks: Vec<Vec<u8>>,
    pub was_incremental: bool,
    pub error_code: Option<String>,
    pub preserved_abort_reason: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResponseObservation {
    pub ok: bool,
    pub status: u16,
    pub status_text: String,
    pub headers: HeaderMap,
    pub body: Option<BodyObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PoolObservation {
    pub socket_released_after_cancel: bool,
    pub next_request_status: Option<u16>,
    pub queued_request_timed_out: bool,
    pub unique_socket_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeHttpObservation {
    pub request: RequestObservation,
    pub response: Option<ResponseObservation>,
    pub pool: PoolObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeHttpError {
    pub code: Option<String>,
    pub kind: String,
    pub message: String,
    pub preserved_abort_reason: bool,
}

pub fn exercise_node_http_case(case: NodeHttpCase) -> Result<NodeHttpObservation, NodeHttpError> {
    let _ = case;
    panic!("TDD RED: packages/world/src/node-http.test.ts implementation pending")
}

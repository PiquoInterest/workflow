fn pending<T>() -> T {
    panic!("TDD RED: packages/core/src/byte-stream-framing.test.ts implementation pending")
}

/// Number of bytes in the network-order frame length prefix.
pub const FRAME_HEADER_SIZE: usize = 4;
/// Maximum accepted frame payload size.
pub const MAX_FRAME_SIZE: usize = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteFramingErrorKind {
    FrameTooLarge,
    TruncatedFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteFramingError {
    pub kind: ByteFramingErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ByteStreamRoundTrip {
    pub serialized_reference: String,
    pub stored_chunks: Vec<Vec<u8>>,
    pub user_chunks: Vec<Vec<u8>>,
    pub sink_closed: bool,
}

/// Frames every non-empty chunk with a big-endian u32 payload length.
pub fn frame_chunks(chunks: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, ByteFramingError> {
    let _ = chunks;
    pending()
}

/// Validates a declared frame length before any payload-sized allocation.
pub fn validate_frame_length(length: u64) -> Result<(), ByteFramingError> {
    let _ = length;
    pending()
}

/// Incrementally decodes frames from arbitrary transport read boundaries.
pub fn unframe_reads(reads: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, ByteFramingError> {
    let _ = reads;
    pending()
}

/// Exercises the future Rust stream-reference dehydration and hydration path.
pub fn dehydrate_byte_stream(
    chunks: &[Vec<u8>],
    framed: bool,
) -> Result<ByteStreamRoundTrip, ByteFramingError> {
    let _ = (chunks, framed);
    pending()
}

/// Persists one frame per stored chunk and decodes the resulting wire stream.
pub fn persist_framed_chunks(
    chunks: &[Vec<u8>],
) -> Result<ByteStreamRoundTrip, ByteFramingError> {
    let _ = chunks;
    pending()
}

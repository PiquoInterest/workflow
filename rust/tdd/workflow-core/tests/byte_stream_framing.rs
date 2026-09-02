use workflow_core_tdd::byte_stream_framing::{
    ByteFramingErrorKind, FRAME_HEADER_SIZE, MAX_FRAME_SIZE, dehydrate_byte_stream, frame_chunks,
    persist_framed_chunks, unframe_reads, validate_frame_length,
};

fn header(length: u32) -> Vec<u8> {
    length.to_be_bytes().to_vec()
}

fn framed(payload: &[u8]) -> Vec<u8> {
    let mut output = header(payload.len() as u32);
    output.extend_from_slice(payload);
    output
}

fn concat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.iter().flatten().copied().collect()
}

#[test]
fn framer_wraps_each_chunk_in_a_four_byte_big_endian_prefix() {
    let chunks = vec![vec![1, 2, 3], vec![4, 5], vec![6]];
    let actual = frame_chunks(&chunks).expect("valid chunks must frame");
    assert_eq!(
        actual,
        vec![framed(&[1, 2, 3]), framed(&[4, 5]), framed(&[6])]
    );
}

#[test]
fn framer_drops_empty_chunks() {
    let chunks = vec![vec![1], Vec::new(), vec![2]];
    let actual = frame_chunks(&chunks).expect("empty chunks must be ignored");
    assert_eq!(actual, vec![framed(&[1]), framed(&[2])]);
}

#[test]
fn framer_handles_a_large_chunk() {
    let chunk: Vec<u8> = (0..64_000).map(|index| (index & 0xff) as u8).collect();
    let actual = frame_chunks(std::slice::from_ref(&chunk)).expect("64 KiB must frame");
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].len(), FRAME_HEADER_SIZE + chunk.len());
    assert_eq!(
        u32::from_be_bytes(actual[0][..FRAME_HEADER_SIZE].try_into().expect("header")),
        chunk.len() as u32
    );
    assert_eq!(&actual[0][FRAME_HEADER_SIZE..], chunk.as_slice());
}

#[test]
fn framer_handles_clean_eof() {
    assert!(
        frame_chunks(&[])
            .expect("clean EOF must succeed")
            .is_empty()
    );
}

#[test]
fn framer_rejects_chunks_larger_than_the_safety_cap() {
    let error = validate_frame_length((MAX_FRAME_SIZE + 1) as u64)
        .expect_err("the producer must reject oversized chunks");
    assert_eq!(error.kind, ByteFramingErrorKind::FrameTooLarge);
    assert!(error.message.contains("maximum"));
}

#[test]
fn unframer_round_trips_framed_chunks() {
    let chunks = vec![b"hello".to_vec(), b", ".to_vec(), b"world".to_vec()];
    let wire = frame_chunks(&chunks).expect("framing must succeed");
    assert_eq!(
        unframe_reads(&wire).expect("unframing must succeed"),
        chunks
    );
}

#[test]
fn unframer_reassembles_a_frame_split_across_reads() {
    let reads: Vec<Vec<u8>> = framed(b"hello")
        .into_iter()
        .map(|byte| vec![byte])
        .collect();
    assert_eq!(
        unframe_reads(&reads).expect("split frame must reassemble"),
        vec![b"hello".to_vec()]
    );
}

#[test]
fn unframer_emits_multiple_frames_coalesced_into_one_read() {
    let coalesced = concat(&[framed(&[1, 2, 3]), framed(&[4, 5]), framed(&[6])]);
    assert_eq!(
        unframe_reads(&[coalesced]).expect("coalesced frames must split"),
        vec![vec![1, 2, 3], vec![4, 5], vec![6]]
    );
}

#[test]
fn unframer_rejects_a_truncated_frame() {
    let mut truncated = header(5);
    truncated.extend_from_slice(&[1, 2]);
    let error = unframe_reads(&[truncated]).expect_err("mid-frame EOF must fail");
    assert_eq!(error.kind, ByteFramingErrorKind::TruncatedFrame);
    assert!(error.message.to_ascii_lowercase().contains("truncated"));
}

#[test]
fn unframer_rejects_an_advertised_oversized_frame_before_allocation() {
    let mut bogus = header(200_000_000);
    bogus.extend_from_slice(&[1, 2, 3]);
    let error = unframe_reads(&[bogus]).expect_err("oversized frame header must fail fast");
    assert_eq!(error.kind, ByteFramingErrorKind::FrameTooLarge);
}

#[test]
fn unframer_accepts_clean_eof_without_buffered_data() {
    assert!(
        unframe_reads(&[])
            .expect("clean EOF must succeed")
            .is_empty()
    );
}

#[test]
fn unframer_preserves_one_hundred_small_chunk_boundaries() {
    let chunks: Vec<Vec<u8>> = (0..100).map(|value| vec![value]).collect();
    let wire = frame_chunks(&chunks).expect("small chunks must frame");
    assert_eq!(
        unframe_reads(&wire).expect("round-trip must succeed"),
        chunks
    );
}

#[test]
fn legacy_dehydration_omits_the_framing_field() {
    let observation =
        dehydrate_byte_stream(&[vec![1, 2, 3]], false).expect("legacy dehydration must succeed");
    assert!(observation.serialized_reference.contains("ReadableStream"));
    assert!(!observation.serialized_reference.contains("framing"));
    assert!(!observation.serialized_reference.contains("framed-v1"));
}

#[test]
fn framed_dehydration_emits_framed_v1() {
    let observation =
        dehydrate_byte_stream(&[vec![1, 2, 3]], true).expect("framed dehydration must succeed");
    assert!(observation.serialized_reference.contains("framed-v1"));
}

#[test]
fn framed_dehydrate_and_hydrate_round_trip_user_chunks() {
    let chunks = vec![vec![1, 2, 3], vec![4, 5], vec![6, 7, 8, 9]];
    let observation = dehydrate_byte_stream(&chunks, true).expect("framed round-trip must succeed");
    assert_eq!(observation.user_chunks, chunks);
    assert!(observation.sink_closed);
    assert!(
        observation
            .stored_chunks
            .iter()
            .all(|chunk| chunk.len() >= FRAME_HEADER_SIZE)
    );
}

#[test]
fn raw_dehydrate_and_hydrate_round_trip_user_chunks() {
    let chunks = vec![vec![10, 20, 30]];
    let observation = dehydrate_byte_stream(&chunks, false).expect("raw round-trip must succeed");
    assert_eq!(observation.stored_chunks, chunks);
    assert_eq!(observation.user_chunks, chunks);
}

#[test]
fn persisted_frames_unframe_to_the_original_chunks() {
    let chunks = vec![vec![1, 2], vec![3, 4, 5], vec![6]];
    let observation = persist_framed_chunks(&chunks).expect("persisted frames must round-trip");
    assert_eq!(observation.user_chunks, chunks);
    assert_eq!(observation.stored_chunks.len(), 3);
    assert!(observation.sink_closed);
}

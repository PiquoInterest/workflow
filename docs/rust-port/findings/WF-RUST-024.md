# WF-RUST-024: Framed byte streams must reject oversized and truncated input

**Status:** TypeScript regression exists; Rust regression suite is TDD RED.

## Protocol boundary

Byte streams use a four-byte big-endian unsigned length followed by that many
payload bytes. The current TypeScript runtime caps a frame at 100,000,000 bytes,
drops empty producer chunks, buffers headers and payloads across transport read
boundaries, and rejects cleanly when the stream ends with an incomplete frame.

The length prefix is untrusted transport data. A decoder must validate it before
allocating a payload-sized buffer or waiting indefinitely for a body that is not
allowed by the protocol.

## Security and integrity impact

Accepting an attacker-controlled oversized length can cause memory exhaustion or
long-lived buffering. Treating an incomplete frame as clean EOF can silently
truncate serialized workflow data and desynchronize every later frame. Emitting
zero-length frames also collides with the legacy framed-stream sniff and can make
raw and framed representations ambiguous.

## Required Rust invariant

- Encode each non-empty chunk with one network-order u32 length prefix.
- Reject producer chunks and advertised consumer frames above 100,000,000 bytes.
- Check the declared size before any payload-sized allocation.
- Reassemble split reads and split coalesced reads without changing chunk order.
- Reject EOF with a partial header or partial payload.
- Preserve legacy raw stream references and emit `framed-v1` only when enabled.

## Regression evidence

- TypeScript source: `packages/core/src/byte-stream-framing.test.ts`.
- Rust expected-RED translation:
  `rust/tdd/workflow-core/tests/byte_stream_framing.rs`.

All 17 TypeScript declarations are represented, including direct framing,
incremental decoding, malformed input, reference serialization, and world-stream
round trips.

## Closure condition

This finding closes for Rust only when the production incremental decoder,
stream-reference serializer, world stream writer/reader, and hydration path pass
the translated suite. Returning fixture-specific observations or merely checking
a complete in-memory buffer is insufficient.

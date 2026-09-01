# WF-RUST-097: Replay cache aliases conflicting payload bytes

**Status:** TypeScript characterization committed; Rust secure target is TDD RED.

## Original behavior

`ReplayPayloadCache` indexes prepared event payloads by `eventId` and field. Once
one binary payload has populated that key, a later call with the same key returns
the original in-flight or completed promise without comparing the supplied
bytes. The second payload is never passed to `prepareReplayPayload`.

The characterization test in
`packages/core/src/replay-payload-cache-security.test.ts` supplies two different
`Uint8Array` values under the same event ID and `result` field. It proves that
TypeScript returns the first promise and first prepared bytes for both calls.

## Security and correctness impact

Event IDs are expected to bind immutable event content. A stale snapshot,
corrupted backend response, collision, or compromised adapter can violate that
assumption. Reusing the first preparation hides the conflict and skips the
second payload's authentication, decryption, decompression, and parse path. The
runtime can therefore associate bytes from one event version with another log
version instead of reporting corruption.

This does not make unauthenticated bytes valid. It bypasses the validation path
entirely by returning a previously accepted value for a now-conflicting key.

## Required Rust invariant

A workflow-input or event cache key is immutable for the invocation lifetime.
The production Rust cache must retain the original binary bytes in its shared
preparation cell. Reusing the key with byte-for-byte identical input shares the
same preparation. Reusing it with different bytes transitions the cell to a
terminal conflict state and returns a typed integrity error.

The error must be static and must not include the run ID, event ID, payload, or
cryptographic material. A conflict is not evicted for retry because retrying the
same contradictory log cannot repair it.

## Regression evidence

- Unsafe TypeScript characterization:
  `packages/core/src/replay-payload-cache-security.test.ts`
- Rust secure RED target:
  `rust/tdd/workflow-core/tests/replay_payload_cache_security.rs`
- Existing cache behavior suite:
  `packages/core/src/replay-payload-cache.test.ts`
  and `rust/tdd/workflow-core/tests/replay_payload_cache.rs`

## Closure condition

WF-RUST-097 is GREEN only when the production `workflow-core` cache shares
identical bytes, rejects conflicting bytes before invoking the preparer, keeps
the conflict terminal across later lookups, emits no attacker-controlled data in
the error, and the TypeScript characterization remains available as evidence of
the intentional security divergence.

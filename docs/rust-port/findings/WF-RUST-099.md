# WF-RUST-099: Replay cache keys were not bound to payload identity

**Status:** Fixed in TypeScript and Rust; branch CI pending.

## Original behavior

`ReplayPayloadCache` indexed prepared workflow inputs by run ID and event
payloads by `(eventId, field)`. Once one binary payload populated a key, a later
lookup using the same logical key returned the original in-flight or completed
promise without comparing the supplied bytes. The second payload was never
passed through authentication, decryption, decompression, or parsing.

The cache also invoked its configurable preparer before publishing the new cache
entry. A synchronous preparer that re-entered the same key could therefore start
a second preparation instead of observing the first in-flight binding.

## Security and correctness impact

A stale snapshot, corrupted backend response, collision, cross-wired adapter, or
compromised transport can violate the assumption that one logical event identity
always carries one byte sequence. Returning previously accepted plaintext hides
the conflict and makes the result depend on cache timing. Preparer re-entry could
also bypass single-preparation guarantees or recurse through the same key.

This does not make unauthenticated bytes valid. It bypasses the validation path
by returning a value prepared for a different byte sequence.

## TypeScript fix

The active TypeScript cache now binds every binary key to an immutable byte
snapshot. It gives the preparer a separate copy, so caller mutation, an
overridden typed-array `slice()`, or preparer mutation cannot rewrite the stored
identity. Byte-for-byte identical lookups share the exact promise.

A different byte sequence creates a terminal `ReplayPayloadConflictError` with
code `REPLAY_PAYLOAD_CONFLICT`. The message is constant and contains no run ID,
event ID, payload byte, or cryptographic material. Conflicts reject an original
in-flight waiter, cannot be overwritten by later preparation completion, never
invoke the preparer for the conflicting bytes, and are not evicted for retry.

The entry is published before custom preparation code runs. Synchronous
same-key re-entry is rejected with `ReplayPayloadReentrantError` and code
`REPLAY_PAYLOAD_REENTRANT`, while a re-entry using different bytes creates the
same terminal conflict as any other contradictory lookup.

## Rust fix

The production Rust cache stores the original bytes in each shared preparation
cell. `PayloadConflict` is terminal and redacted, wakes in-flight waiters, and
cannot be replaced when the original preparation later finishes. Rust also
rejects same-thread same-key re-entry with `ReentrantPreparation`.

## TDD evidence

1. The new TypeScript regression compiles against the old implementation and
   fails because conflicting keys resolve successfully and synchronous re-entry
   starts duplicate preparation.
2. The TypeScript implementation is patched until the regression and the
   existing replay-cache behavior suite are green.
3. Production Rust behavior and security tests exercise the same immutable-key,
   in-flight conflict, terminal-state, and re-entry invariants.
4. The permanent Rust-core workflow runs the TypeScript oracles before the Rust
   behavior/security targets, all targets, Rustfmt, Clippy, and differential
   checks.

## Regression evidence

- `packages/core/src/replay-payload-cache.test.ts`
- `packages/core/src/replay-payload-cache-security.test.ts`
- `crates/workflow-core/src/replay_payload_cache.rs`
- `crates/workflow-core/tests/replay_payload_cache.rs`
- `crates/workflow-core/tests/replay_payload_cache_security.rs`

## Closure condition

WF-RUST-099 is GREEN only when both active implementations share identical
bytes, reject contradictory bytes before a second preparation, keep conflicts
terminal, reject synchronous same-key re-entry, redact diagnostics, and pass the
permanent TypeScript and Rust CI gates. The TypeScript implementation may not
remain intentionally vulnerable and still count as issue closure.

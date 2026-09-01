# WF-RUST-020: Permanently missing event payloads must not retry

**Status:** Fixed in the TypeScript World boundary; Rust regression suite is TDD
RED.

## Original behavior

A backend can report that an event still exists but its separately stored
payload is permanently gone. Treating that response like a transient transport
failure causes every redelivery to request the same absent object again. The
captured production incident described by the TypeScript regression performed
12,932 reads of one run in 26 minutes without any path to recovery.

## Security and availability impact

Retrying permanent corruption amplifies one missing object into sustained
backend load, queue traffic, and run starvation. An attacker who can induce or
select corrupted references could turn a bounded integrity failure into an
availability problem. The run also never reaches a terminal state visible to
operators.

## Required Rust invariant

The Rust World client must convert the backend's payload-missing terminal frame
into a typed `CorruptedEventLog` error. `is_retryable_world_error` must return
false for that type regardless of message text. Unknown terminal stream error
codes must also fail closed unless a typed policy explicitly marks them
transient.

## Regression evidence

- TypeScript classification and retry tests:
  `packages/core/src/classify-error.test.ts`.
- Rust expected-RED translation:
  `rust/tdd/workflow-core/tests/classify_error.rs`.

The same suite separately proves that 429, 5xx, transport, and timeout failures
remain retryable, while schema and parse failures remain terminal.

## Closure condition

This finding closes for Rust only when the production World stream parser emits
the typed corruption error, the queue redelivery policy consumes the typed
classification, and an end-to-end test proves a missing payload terminates the
run without repeated reads.

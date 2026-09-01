# WF-RUST-019: Abort hydration can race idle argument serialization

**Status:** Fixed in the TypeScript runtime; Rust regression suite is TDD RED.

## Original behavior

An abort controller is updated after a `hook_received` event hydrates its
serialized reason. That hydration is asynchronous. Before the TypeScript fix,
the abort delivery did not participate in `pendingDeliveries`, so
`scheduleWhenIdle()` could dehydrate a later step's arguments while the abort
was still in flight. The downstream signal was then persisted with
`aborted: false` even though the controller became aborted shortly afterward.

## Security and correctness impact

Cancellation and abort signals often guard expensive, privileged, or externally
visible work. Serializing stale non-aborted state can let a downstream step run
after the workflow has revoked it. Variable decryption or deserialization
latency made the race nondeterministic, which also weakens replay consistency.

## Required Rust invariant

The Rust event consumer must hold an RAII-style pending-delivery guard from the
moment an abort receipt is claimed until reason hydration and signal mutation
finish. The guard must be released exactly once on success, hydration failure,
cancellation, or panic conversion. Idle/suspension logic must not serialize
queued arguments while that guard is held.

## Regression evidence

- TypeScript source and fixed regression:
  `packages/core/src/abort-replay-ordering.test.ts`.
- Rust expected-RED translation:
  `rust/tdd/workflow-core/tests/abort_replay_ordering.rs`.

The Rust test injects a 50 ms hydration boundary and requires
`pending_during_hydration == 1`, `aborted_during_hydration == false`, an idle
capture of `true` only after hydration, the original abort reason, and a final
pending count of zero.

## Closure condition

This finding is complete for Rust only when the real event consumer, payload
hydrator, idle gate, abort-controller state, and every error exit pass the
translated tests without fixture-specific observations.

# WF-RUST-025: Stream completion must wait for accepted-prefix durability

**Status:** TypeScript regression exists; Rust regression suite is TDD RED.

## Durability boundary

Some World stream sinks acknowledge `write()` when a chunk enters a local
buffer, then expose a separate drain barrier for the group commit that makes the
accepted prefix durable. The runtime may observe the user-side stream lock being
released before that barrier has resolved.

A flushable stream therefore has two different milestones: transport acceptance
and durable completion. They must not be treated as the same event.

## Security and integrity impact

If an invocation persists step success or returns while accepted bytes remain
volatile, a process crash can leave durable workflow state referring to missing
stream data. The same rule applies to failure: reporting a producer error before
the accepted prefix drains can lose the prefix while replay assumes it was
committed. This creates persistent state/data disagreement, replay failures, and
availability loss.

## Required Rust invariant

- A rejection is observed internally before user code awaits the state, avoiding
  an unhandled-rejection analogue.
- Writable/readable lock polling is singleton per state and never resolves while
  writes are still in flight.
- Natural close, source failure, and sink failure preserve the accepted prefix.
- A sink drain barrier is adopted when present and omitted for per-write durable
  sinks.
- Neither success nor failure settles before the barrier resolves.
- Barrier failure rejects completion.
- Chunks remain ordered, the sink closes on successful EOF, and pending operation
  accounting returns to zero.

## Regression evidence

- TypeScript source: `packages/core/src/flushable-stream.test.ts`.
- Rust expected-RED translation:
  `rust/tdd/workflow-core/tests/flushable_stream.rs`.

All 16 TypeScript declarations are represented, including early-ack drains,
accepted-prefix failure ordering, concurrent writes, delayed writes, polling,
and cleanup.

## Closure condition

This finding closes for Rust only when the production async stream pipe, World
sink abstraction, drain barrier, lock-release detector, and invocation
completion path pass the translated suite under deterministic tests. A mock that
returns precomputed observations is not sufficient.

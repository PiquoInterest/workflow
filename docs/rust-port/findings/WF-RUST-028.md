# WF-RUST-028: Dual-channel abort state must converge across replay

**Status:** TypeScript characterization exists; Rust regression suite is TDD RED.

## Consistency boundary

Workflow cancellation is represented through two channels with different
purposes:

- a stream carries low-latency abort notification to running steps;
- a hook receipt records the durable replay fact consumed by the workflow.

Either channel can fail independently. Local AbortController state, serialized
metadata, stream identity, hook identity, event-log state, and replay listeners
must therefore follow an explicit convergence model rather than assuming both
channels complete together.

## Security and integrity impact

Confusing the channels can produce stale authorization or cancellation state. A
step may stop while the workflow later replays as live, or the workflow may
replay as aborted while a running step never received the real-time signal. A
double abort must not enqueue duplicate messages, overwrite the first reason, or
rebind stream and hook identifiers. Queue failures must not turn a local abort
into event-log corruption or an unhandled process failure.

Replay ordering is also observable. First-run listeners fire synchronously at
the abort call site. On replay, the durable event sets state and fires listeners
at the event's log position. Listeners attached after a replayed abort observe it
immediately. Changing those rules can make workflow branching depend on whether
execution is fresh or replayed.

## Required Rust invariant

- Pre-aborted state is captured in serialization without installing a redundant
  stream listener.
- A later abort enqueues at most one stream packet and preserves the first
  reason.
- Stream-only success reaches the step but does not invent a durable workflow
  abort.
- Hook-only success becomes an aborted replay fact even if real-time delivery
  failed.
- Failure of both channels is isolated from process failure and does not corrupt
  bound stream/hook identity.
- First-run and replay listener order matches the TypeScript contract.
- Fire-and-forget internal items do not block normal workflow completion.
- Pending step, hook, and wait items are retained in suspension output.

## Regression evidence

- TypeScript source: `packages/core/src/abort-consistency.test.ts`.
- Rust expected-RED translation:
  `rust/tdd/workflow-core/tests/abort_consistency.rs`.

All 22 executable TypeScript cases are represented across serialization races,
partial failures, idempotency, replay delivery, listener timing, workflow
completion, and suspension collection.

## Closure condition

This finding closes for Rust only when the production serializer, abort state
machine, stream and hook transports, event consumer, replay scheduler, and
suspension collector pass the translated suite and cross-execution E2E tests. A
fixture-specific observation layer is not sufficient.

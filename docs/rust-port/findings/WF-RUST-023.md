# WF-RUST-023: Correlation draws must be monotonic under log extension

**Status:** Fixed by default in TypeScript; Rust regression suite is TDD RED.

## Production failure

Concurrent replays can observe different dense prefixes of one event log. With
a shared arrival-order counter, a branch that is still blocked in the shorter
prefix mints nothing, while a woken sibling can take the ordinal that a fresher
replay assigns to the blocked branch's wait. The same correlation ID then names
a finalize step in one replay and a wait in another. Later replays encounter an
unconsumable event and terminate a run whose work had otherwise succeeded with
`CORRUPTED_EVENT_LOG`.

## Security and integrity impact

Correlation IDs are the join key between events and consumers. Rebinding one ID
to a different entity lets scheduling order change event ownership, violating
deterministic replay. Under adversarial or merely high concurrency this turns a
transient race into durable log corruption and availability loss.

## Required Rust invariant

The Rust runtime must allocate correlation draws in event-log order within the
delivery cascade that made a branch runnable. Extending a dense prefix may append
new bindings or consume old pending entities, but any ID present in both replays
must name the same entity. Replaying the same prefix must be deterministic. The
arrival-order mode remains only as an explicit compatibility control and must
not be the production default.

## Regression evidence

- TypeScript production-shape regression:
  `packages/core/src/log-order-draws.test.ts`.
- Rust expected-RED translation:
  `rust/tdd/workflow-core/tests/log_order_draws.rs`.

The Rust suite checks the original rebound control, the shorter/longer prefix
pair, every consecutive dense prefix, and repeated evaluation of one prefix.

## Closure condition

This finding closes for Rust only when the real scheduler, event consumer,
correlation allocator, step/hook/wait primitives, and concurrent replay E2E pass
the translated tests with log-order draws enabled by default.

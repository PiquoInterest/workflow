# WF-RUST-026: Delivery barriers must retire in log order and survive queue failure

**Status:** TypeScript regression exists; Rust regression suite is TDD RED.

## Scheduler boundary

An event delivery can park behind an earlier unclaimed payload barrier. At the
end of a replay pass, a safety-net dispenser retires barriers that no longer
have a legitimate owner so the parked delivery can run before the workflow is
allowed to suspend.

Several parked chains may exist at once. Their retirement order affects which
branch becomes runnable first and therefore which correlation identifiers later
workflow operations draw.

## Security and integrity impact

Suspending before a parked delivery runs can leave an otherwise recoverable run
dormant indefinitely. Retiring several barriers according to task scheduling
rather than event-log position makes correlation ownership timing-dependent and
can produce replay divergence. If one rejected promise queue permanently kills
the dispenser, its registry keeps delivery-idle false and wedges every later
suspension or completion attempt.

## Required Rust invariant

- End-of-log suspension waits until eligible parked deliveries run and their
  woken branches make follow-up draws.
- Multiple barrier heads retire lowest event-log position first, re-blocking
  while each awakened chain drains.
- Correlation identifiers drawn after wake-up remain ordered by log position.
- A rejected internal promise queue is isolated; the dispenser re-arms, retires
  eligible entries, and restores the delivery-idle predicate.

## Regression evidence

- TypeScript source: `packages/core/src/delivery-barrier-dispenser.test.ts`.
- Rust expected-RED translation:
  `rust/tdd/workflow-core/tests/delivery_barrier_dispenser.rs`.

All three production-shape TypeScript cases are represented.

## Closure condition

This finding closes for Rust only when the real event consumer, barrier registry,
promise queue, suspension detector, scheduler, and correlation allocator pass
the translated tests and the concurrent replay E2E lanes. A scenario-specific
observation function is insufficient.

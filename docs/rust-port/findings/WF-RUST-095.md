# WF-RUST-095: Warmup deadlines must include probe creation

**Status:** TypeScript characterization retained; Rust regression suite is TDD RED.

## Affected surface

- `packages/core/e2e/utils.ts`
- `packages/core/e2e/utils.test.ts`
- Rust replacement warmup, run-status polling, cancellation, and test-state helpers

## Original behavior

`warmDeployment()` computes a total deadline, then awaits `startProbe()` before
checking the remaining time. Probe creation is therefore outside the claimed
total budget. If it completes after the deadline, the helper passes a zero or
negative pickup budget into `waitForRunPickup()`. That helper performs one
status read before its deadline check, so a late probe can still be accepted as
picked up after the total warmup budget expired. A never-resolving `startProbe()`
can hang the warmup indefinitely.

Stalled-probe cancellation is also launched as an unawaited best-effort promise.
The suite may proceed while abandoned probes or their cancellation requests are
still active.

## Security and correctness impact

Warmup runs share a deployment with the actual E2E suite. Unbounded probe
creation or cancellation can consume queue capacity, leak work into later tests,
and make a nominally bounded CI stage hang. Accepting a status after the total
deadline also makes timeout behavior dependent on remote response timing.

## Required Rust invariant

The Rust warmup path must apply one checked deadline to probe creation, pickup
polling, and cancellation. It must not start another probe or perform a status
read when no budget remains. Status-read failures use the same exponential
500 ms to 5 s schedule, sleeps are capped by the remaining budget, and every
stalled probe receives the documented cancellation reason through a bounded
cleanup path.

## Regression evidence

`rust/tdd/workflow-core/tests/e2e_utils.rs` translates all 15 source
declarations and adds explicit deadline-boundary and late-probe cases.

## Closure condition

The finding closes only when the real Rust run handle, timer, status reader,
probe starter, cancellation path, infra-event recorder, and task-local test state
pass the translated suite. A scripted observation is not sufficient.

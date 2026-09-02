# WF-RUST-027: Step abort publication must be durable before completion

**Status:** TypeScript regression exists; Rust regression suite is TDD RED.

## Cancellation boundary

An AbortController passed into a step is backed by two operations:

- a stream write for real-time notification of in-flight sibling steps;
- a hook receipt for durable workflow replay.

The stream write may remain background work. The hook receipt may not. A step
that aborts and then completes must not publish `step_completed` until the
abort's hook receipt exists durably in the event log.

The distinction became more important when the ordinary hook-resume entry point
became lazy: waiting for publication is not the same as waiting for the durable
event write. The abort path therefore requires the durable entry point and must
place that operation in the executor's pre-completion queue.

## Security and integrity impact

If `step_completed` becomes visible first, the workflow continuation can replay
and dispatch a later step with a stale non-aborted signal. Cancellation then
loses its ordering guarantee even though the aborting step reported success.
Depending on the later step, this can execute work after revocation, duplicate a
side effect, or make replay depend on queue timing.

Incorrectly treating every failure in a step carrying an AbortSignal as fatal
would create a separate denial-of-service risk, so only actual abort failures
may bypass retries.

## Required Rust invariant

- A deserialized live signal registers its stream reader; a pre-aborted signal
  does not.
- Stream packets preserve reasons, fire listeners, and make throw-if-aborted
  fail.
- Local abort flips state synchronously and is safe outside step context.
- The real-time stream write is background work.
- The durable hook receipt uses the durable resume path and is queued exactly
  once in pre-completion work.
- The executor drains pre-completion work before writing step completion.
- Abort failures are fatal and skip retries; unrelated errors remain subject to
  ordinary retry policy.

## Regression evidence

- TypeScript source: `packages/core/src/abort-controller-step.test.ts`.
- Rust expected-RED translation:
  `rust/tdd/workflow-core/tests/abort_controller_step.rs`.

All 19 executable TypeScript cases are represented, including the production
serialization/hydration regression that distinguishes background operations from
pre-completion operations and distinguishes lazy from durable hook resume.

## Closure condition

This finding closes for Rust only when the production serialization bridge,
step context, stream reader/writer, durable hook writer, and step executor pass
the translated suite and cancellation E2E tests. A scenario function returning
precomputed observations is not sufficient.

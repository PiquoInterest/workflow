# WF-RUST-098: Retry accounting must separate racers from real attempts

**Status:** Rust counting contract implemented; executor and World-event adoption pending.

## TypeScript reference behavior

`packages/core/src/runtime/count-step-started-events.ts` contains the corrected
retry accounting introduced for workflow#3069. A raw count of every
`step_started` record is not a valid attempt number when multiple invocations
race over the same pending batch. Each racer may stamp one duplicate start for
the same logical attempt.

The TypeScript implementation now offers two scoped counts:

- `ownedBy` counts starts written by one owning queue message;
- `totalAttempts` counts bare/background starts plus the largest count from one
  owner message.

This preserves the genuine owner phase without accumulating one-off racer IDs.

## Security and availability impact

Treating racing duplicates as retries can fail healthy work with a false
"exceeded max retries" result. An attacker or overloaded queue that increases
concurrent replays can therefore turn ordinary duplicate delivery into a durable
availability failure. The opposite mistake, dropping all owner history, removes
the ceiling from genuine timeout retries and permits unbounded work.

Attempt counters are also persisted across the JavaScript/Rust boundary. Values
outside JavaScript's exact integer range can change identity after conversion
and make retry decisions non-deterministic.

## Rust implementation

`crates/workflow-core/src/runtime/count_step_started_events.rs`:

- stores internal counts in the private `StepAttemptCount(u64)` newtype;
- permits only values below `Number.MAX_SAFE_INTEGER`, because the next action
  advances the count by one;
- accepts `f64` only at the JavaScript compatibility boundary;
- rejects negative, fractional, non-finite, maximum, and above-maximum inputs;
- uses checked integer increments and additions, never saturation;
- preserves unscoped, exact-owner, and total-attempt semantics;
- keeps error text static and never reflects an untrusted count or owner ID.

## TDD evidence

1. The translated Rust suite was committed under
   `rust/tdd/workflow-core/tests/count_step_started_events.rs`.
2. GitHub run `33500259481` proved Rustfmt, compilation, and Clippy succeeded and
   every case failed only at the registered source-specific RED marker.
3. The unchanged behavioral requirements moved to
   `crates/workflow-core/tests/count_step_started_events.rs`.
4. `rust/conformance/count-step-started-events-parity.test.ts` compares the
   production Rust binary with the TypeScript oracle over valid, invalid,
   owner-scoped, racer, and mixed-retry fixtures.

## Closure condition

The finding is fully closed only when the Rust executor uses this typed
transition directly and persisted `workflow-world` events adapt into this
counter without lossy numbers, copied ownership rules, or a TypeScript fallback.

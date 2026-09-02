# WF-RUST-090: Benchmark measurement and retry integrity

**Status:** TDD RED for the Rust runtime.

## Affected surface

- `packages/core/e2e/benchmark.test.ts`
- the benchmark trigger route used by the workbench deployment
- Rust replacement runtime, stream transport, timing collection, and artifact writer

## Trust boundary

Benchmark responses come from a remote deployment and contain a run identifier,
deployment-side timestamp, step timings, chunk timing arrays, and replay-cadence
metadata. Those fields are not trustworthy merely because the request itself
succeeded. They drive performance regressions, baseline updates, and retry
behavior in CI.

## Failure modes preserved as implementation gates

1. A missing or malformed `runId` must reject the sample.
2. A missing, non-finite, or non-numeric deployment-side `clientStart` must
   reject the sample. CI request-start time must not be substituted because it
   includes ingress and queue latency that the benchmark intentionally excludes.
3. Step and chunk arrays must contain exactly the requested number of records.
   Partial success cannot be treated as a faster complete run.
4. Replay cadence IDs must remain coupled to the captured workload's semantic
   hash. A changed event stream must not inherit an old baseline under the same
   display name.
5. Scenarios run sequentially. Parallel scenarios can contaminate one another's
   latency distributions and turn resource exhaustion into misleading product
   regressions.
6. Retry work is bounded by the configured failure ratio, and a scenario aborts
   after three attempts with zero accepted samples. A wedged or adversarial
   endpoint cannot consume an unbounded CI budget.

## Rust contract

`rust/tdd/workflow-core/tests/benchmark.rs` translates all ten TypeScript
scenario declarations. It preserves exact workflow names and arguments,
methodology version 2, deployment-side clock anchoring, negative-skew clamping,
TTFS thresholds, CRTT index buckets, size/write-slip profiles, captured replay
identities and spans, fan-out first/last completion metrics, sequential
inline-versus-queue-hop gaps, and whole-run overhead.

The TDD module deliberately panics at a source-specific marker. A fabricated
observation or local-only timing implementation must not satisfy the final
contract.

## Closure condition

The finding closes only when the real Rust deployment trigger, run execution,
stream reader, timing validator, retry controller, cadence verifier, and result
artifact path pass the translated suite. Rustfmt, compilation, Clippy, the
source-specific expected-RED gate, and later the GREEN integration lane must all
pass on the same commit.

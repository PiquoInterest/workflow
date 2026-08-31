# TypeScript logic and security findings

This ledger separates exact compatibility work from intentional corrections.
Every closed item must name the old behavior, its impact, the replacement
behavior, and the tests that prevent regression.

## WF-RUST-001: `__proto__` was not an ordinary attribute key

**Status:** Fixed in TypeScript and Rust.

**Affected code:** `packages/world/src/attributes-validation.ts`,
`applyAttributeChanges()`.

**Old behavior:** The helper cloned attributes into a normal JavaScript object
and applied a set with `next[key] = value`. For the legacy magic key
`__proto__`, assignment invokes the inherited prototype setter instead of
creating an own data property. A string value was silently lost. An object value
from a caller that bypassed contextual validation could replace the output
object's prototype.

**Impact:** Attribute integrity failure and a prototype-manipulation primitive
inside the materialization helper. The input schema normally limits values to
strings, which reduces exploitability, but the helper is exported and its
runtime boundary accepted ordinary JavaScript values when called without prior
validation.

**Fix:** TypeScript now uses `Object.defineProperty()` for every upsert. Rust
uses `BTreeMap`, where all strings are data keys by construction.

**Regression evidence:**

- `packages/world/src/attributes-security.test.ts`
- `crates/workflow-world/src/attributes.rs`
- `rust/conformance/world-parity.test.ts`

## WF-RUST-002: Modern serialized payload validation was structurally inert

**Status:** Closed at the Rust boundary; TypeScript compatibility schema remains
until its callers are retired or become spec-aware.

**Affected code:** `packages/world/src/serialization.ts`.

**Old behavior:** `SerializedDataSchema` unions `Uint8Array` with the legacy
`z.any()` schema. Because `z.any()` accepts every value, the union does not
actually enforce binary payloads for spec-version 2 and newer. Callers need
outside protocol context to distinguish legitimate legacy JSON from a mangled
modern payload, but the shared schema does not require that context.

**Impact:** A transport or adapter can silently downgrade modern opaque binary
data to an arbitrary JavaScript value. The failure is delayed until hydration,
which makes corruption harder to attribute and can move malformed data into
persistent storage.

**Fix:** Rust represents binary and legacy payloads as separate enum variants
and `validate_serialized_data_for_spec()` rejects legacy JSON for modern runs.
Legacy spec-version 1 reads remain supported.

**Regression evidence:** `crates/workflow-world/src/serialization.rs`.

**Remaining TypeScript retirement condition:** Every modern create/update path
must use the spec-aware Rust validator before the broad compatibility schema is
removed.

## WF-RUST-003: A malformed health-check could fall through as an invocation

**Status:** Closed at the Rust queue boundary; TypeScript queue parser remains a
migration oracle for valid payloads.

**Affected code:** `packages/world/src/queue.ts`, `QueuePayloadSchema`.

**Old behavior:** Putting the health-check member first fixes valid probes that
also carry `runId`, but the parser is still an ordered union. An object containing
`__healthCheck: false` (or another malformed health-check discriminator) and a
valid `runId` fails the probe member, then satisfies the workflow-invoke member,
which strips the unknown probe fields.

**Impact:** Malformed or adversarial probe-shaped messages can be reinterpreted
as requests to replay a workflow run instead of being rejected. This can cause
unwanted `run_started` traffic and repeated delivery behavior.

**Fix:** Rust treats presence of the `__healthCheck` key as a hard protocol
discriminator. Such a payload must be a valid probe with literal `true`; it
never falls through to workflow invocation parsing.

**Regression evidence:** `crates/workflow-world/src/queue.rs`.

## WF-RUST-004: Bulk cancellation summaries could contradict their results

**Status:** Closed at the Rust boundary.

**Affected code:** `packages/world/src/runs.ts`,
`BulkCancelWorkflowRunsResultSchema`.

**Old behavior:** The TypeScript schema validates the numeric fields and each
result variant independently, but does not prove that `summary.requested`
equals `results.length` or that outcome counters equal the actual result
variants.

**Impact:** A malformed backend response can drive incorrect accounting,
observability, or retry decisions while still passing schema validation.

**Fix:** `BulkCancelWorkflowRunsResult::validate_consistency()` derives the
summary from results and rejects any mismatch.

**Regression evidence:** `crates/workflow-world/src/runs.rs`.

## Open findings tracked for later port stages

| ID | TypeScript condition | Required Rust closure |
| --- | --- | --- |
| WF-RUST-005 | `StepSchema` explicitly has a TODO for a status-discriminated union, so contradictory terminal fields are representable. | Model step states as a Rust enum and add negative fixtures for impossible combinations. |
| WF-RUST-006 | Several date and numeric schemas accept broad coercion or unconstrained numbers at wire boundaries. | Inventory each producer, preserve required legacy coercions, and use bounded integer/newtype validation for modern writes. |
| WF-RUST-007 | Queue telemetry uses intentionally forgiving `.catch(undefined)` behavior. | Keep telemetry non-fatal while making execution-authoritative fields strict and independently bounded. |

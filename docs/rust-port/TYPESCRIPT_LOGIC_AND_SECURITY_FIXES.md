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

## WF-RUST-008: `WorkflowError.is()` rejected a real `WorkflowError`

**Status:** Fixed in TypeScript and Rust.

**Affected code:** `packages/errors/src/index.ts`, `WorkflowError`.

**Old behavior:** The base constructor did not set its public error name, so a
direct `new WorkflowError()` inherited `Error.prototype.name === "Error"`.
The class's own static guard requires the name `"WorkflowError"`, which made
`WorkflowError.is(new WorkflowError())` return `false`.

**Impact:** Catch blocks using the documented type guard could miss the base SDK
error and route it through generic or user-error handling. That weakens
classification and can make retry and telemetry decisions inconsistent.

**Fix:** The TypeScript entry point now assigns the stable name on the
`WorkflowError` prototype before exporting the class. Rust constructors emit the
same public name directly.

**Regression evidence:**

- `packages/errors/src/workflow-error.test.ts`
- `crates/workflow-world/tests/errors_and_time.rs`
- `rust/conformance/errors-parity.test.ts`

## WF-RUST-009: Invalid retry dates could enter scheduling state

**Status:** Fixed in TypeScript and Rust.

**Affected code:** `packages/utils/src/time.ts`, `parseDurationToDate()` and
`RetryableError`.

**Old behavior:** Numeric inputs were checked for finiteness before addition,
but the resulting JavaScript `Date` was never checked after TimeClip. A finite
but enormous duration therefore produced an invalid date. Existing `Date`
objects and date-like deserialized objects were also accepted when `getTime()`
returned `NaN` or infinity.

**Impact:** An invalid retry timestamp could be serialized, persisted, or
compared by queue scheduling code. Depending on the backend this can create an
immediately repeated delivery, a permanently stuck step, or an opaque
serialization failure far from the original input.

**Fix:** TypeScript validates the final `Date.getTime()` for every input form.
Rust validates both the relative calculation and absolute timestamp against the
ECMAScript TimeClip range before returning an integer millisecond timestamp.

**Regression evidence:**

- `packages/utils/src/time.test.ts`
- `crates/workflow-world/src/time.rs`
- `crates/workflow-world/tests/errors_and_time.rs`
- `rust/conformance/errors-parity.test.ts`

## WF-RUST-010: A response-only hook capability shared the entity schema

**Status:** Persistence-safe boundaries implemented; TypeScript adapter adoption
remains part of the world-backend migration.

**Affected code:** `packages/world/src/hooks.ts`, `HookSchema`.

**Old behavior:** `HookSchema` contains both persistent hook state and
`resumeCapabilities`, even though the latter is explicitly response-only and
must be recomputed by the live backend. The invariant existed only in comments;
the exported schema and inferred type still made a capability-bearing lookup
response valid input to generic hook-record processing.

**Impact:** This does not prove that a current adapter persists the field, but it
made that security-sensitive mistake representable. If a stale dedup capability
were written to storage and later returned by a rolled-back or kill-switched
server, the client could keep selecting lazy hook resume without a live backend
attestation. That would weaken the exactly-once convergence guarantee the field
is meant to advertise.

**Fix:** TypeScript now exports `PersistedHookSchema`, which omits and strips
`resumeCapabilities`. Rust uses distinct `PersistedHookProtocolFields` and
`HookLookupProtocolFields` types; the only conversion to persistence discards
the transient capability by construction.

**Regression evidence:**

- `packages/world/src/hooks.test.ts`
- `crates/workflow-world/tests/hooks_contract.rs`
- `rust/conformance/hooks-parity.test.ts`

**Remaining TypeScript retirement condition:** Existing TypeScript storage
adapters must use the persistence-safe schema or be replaced by Rust adapters.
The finding is not considered closed repository-wide until no write path accepts
the response-only field.

## Open findings tracked for later port stages

| ID | TypeScript condition | Required Rust closure |
| --- | --- | --- |
| WF-RUST-005 | `StepSchema` explicitly has a TODO for a status-discriminated union, so contradictory terminal fields are representable. | Model step states as a Rust enum and add negative fixtures for impossible combinations. |
| WF-RUST-006 | Several date and numeric schemas accept broad coercion or unconstrained numbers at wire boundaries. | Inventory each producer, preserve required legacy coercions, and use bounded integer/newtype validation for modern writes. |
| WF-RUST-007 | Queue telemetry uses intentionally forgiving `.catch(undefined)` behavior. | Keep telemetry non-fatal while making execution-authoritative fields strict and independently bounded. |

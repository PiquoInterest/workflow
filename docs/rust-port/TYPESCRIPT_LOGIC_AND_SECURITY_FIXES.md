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

## WF-RUST-005: Step lifecycle states were not discriminated

**Status:** Implemented at the Rust boundary; branch CI pending.

**Affected code:** `packages/world/src/steps.ts`, `StepSchema`.

**Old behavior:** `StepSchema` is one object whose lifecycle-dependent fields
are independently optional. Its own source contains a TODO to replace it with a
discriminated union. The schema accepts records such as:

- a `pending` step carrying `output` and `completedAt`;
- a `running` step carrying `retryAfter` and `completedAt`;
- a modern `completed` step with no `output`;
- a modern `failed` step with `output` and no `error`;
- a `cancelled` step carrying `output` but no `completedAt`.

It also cannot distinguish an omitted forbidden field from a deliberately
present `null` unless every state-specific rule checks property presence.

**Impact:** A corrupted or malicious storage/transport response can pass the
public TypeScript schema while making status, retry scheduling, terminal time,
output, and error disagree. Different consumers can then choose different
sources of truth, causing incorrect retry/terminal decisions, stale result or
error presentation, and persistence of a state that no event sequence can
legitimately produce.

**Proof of the TypeScript issue:**
`packages/world/src/steps-security.test.ts` deliberately asserts that the
legacy `StepSchema` accepts the five contradictory fixtures. This is a permanent
characterization test, not an endorsement of the behavior.

**Fix:** Rust parses step records into the status-discriminated `StepState`
enum. The boundary rejects forbidden field presence even when the value is
`null`, requires `completedAt` for terminal states, requires `output` for modern
completed steps, requires `error` for modern failed steps, and preserves legacy
spec-version 1 records whose terminal payload was absent. Error text names only
the status and field and never reflects serialized payload values.

**Regression evidence:**

- `packages/world/src/steps-security.test.ts`
- `crates/workflow-world/src/steps.rs`
- `crates/workflow-world/tests/step_state_security.rs`
- `crates/workflow-world/examples/steps_conformance.rs`
- `rust/conformance/step-state-security.test.ts`

**Remaining TypeScript retirement condition:** Every step entity read/write path
must cross the Rust state parser, or be replaced by a Rust World adapter, before
the permissive TypeScript schema can be removed. Full date coercion and payload
representation validation remain tracked under WF-RUST-006 and WF-RUST-002.

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

## WF-RUST-011: Hook protocol versions accepted undefined revisions

**Status:** Fixed in TypeScript and Rust; branch CI pending.

**Affected code:** `packages/world/src/hooks.ts`,
`HookResumeContextSchema`, and `HookResumeCapabilitiesSchema`.

**Old behavior:** Hook resume protocol markers used unconstrained `z.number()`
fields in TypeScript and `f64` fields in Rust. Zero, negative, fractional, and
out-of-range values were therefore representable. A fractional attestation such
as `1.5` could satisfy a `>= 1` feature gate even though no revision 1.5 exists.
Rust also serialized integral inputs as JSON floats (`1.0`), which did not match
the TypeScript wire representation (`1`).

**Impact:** A malformed or compromised backend could advertise an undefined
lazy-resume consumer or deduplication revision and make a caller select a path
whose protocol contract was never implemented. For deduplication capabilities,
that can weaken the exactly-once convergence assumption used for repeated hook
resume delivery. The float serialization drift also broke exact cross-language
wire parity.

**Fix:** TypeScript and Rust now require positive unsigned 32-bit protocol
versions. Rust normalizes integral JSON float forms such as `1.0` to the same
integer representation while rejecting zero, negative, fractional, non-finite,
and overflowing values.

**Regression evidence:**

- `packages/world/src/hooks.test.ts`
- `crates/workflow-world/src/hooks.rs`
- `crates/workflow-world/tests/hooks_contract.rs`
- `rust/conformance/hooks-parity.test.ts`

## WF-RUST-012: Step attempt counters accepted unsafe numbers

**Status:** Implemented at the Rust boundary; branch CI pending.

**Affected code:** `packages/world/src/steps.ts`, `StepSchema` and
`UpdateStepRequest.attempt`.

**Old behavior:** The persisted step schema models `attempt` as unconstrained
`z.number()`. Zod 4.3.6 rejects `NaN` and infinities, but accepts finite negative,
fractional, and above-safe-integer values. The local World creates every step at
attempt zero and increments the counter on each `step_started`, so those other
values do not describe a legitimate event history.

**Impact:** Retry ceilings, telemetry, and redelivery ownership use the attempt
as an execution-authoritative counter. A negative value can move retry accounting
backwards, a fraction can represent a start that never happened, and a value
above `Number.MAX_SAFE_INTEGER` can lose identity when incremented or compared.
A malformed persisted step could therefore bypass a retry ceiling, produce an
ambiguous next attempt, or make different implementations disagree about which
execution is authoritative.

**Proof of the TypeScript issue:**
`packages/world/src/steps-security.test.ts` deliberately asserts that the legacy
schema accepts `-1`, `1.5`, and `Number.MAX_SAFE_INTEGER + 1`, while documenting
that the exact Zod version already rejects non-finite numbers.

**Fix:** Rust stores the counter as `u64` and accepts only exact integers in
`0..=9_007_199_254_740_991`. Zero remains valid because it is the canonical
created-but-not-started state. Integral JSON float forms are normalized to an
integer on output. Rejections use a constant diagnostic and never reflect the
untrusted value.

**Regression evidence:**

- `packages/world/src/steps-security.test.ts`
- `crates/workflow-world/src/steps.rs`
- `crates/workflow-world/tests/step_state_security.rs`
- `crates/workflow-world/examples/steps_conformance.rs`
- `rust/conformance/step-state-security.test.ts`

**Remaining TypeScript retirement condition:** Every step read and attempt update
must cross the Rust safe-integer boundary. The Rust World update path must use a
checked increment before the TypeScript producer and its unconstrained request
type can be removed.

## WF-RUST-099: Replay cache keys were not bound to binary payload identity

**Status:** Closed at the production Rust replay boundary. The TypeScript
characterization remains intentionally permissive until the TypeScript replay
cache is retired.

**Affected code:** `packages/core/src/replay-payload-cache.ts`,
`ReplayPayloadCache.prepareWorkflowInput()`, and
`ReplayPayloadCache.prepareEventPayload()`.

**Old behavior:** TypeScript keyed prepared binary payloads only by workflow run
ID or by `(eventId, field)`. Reusing either logical key with different bytes
returned the first cached preparation. The second payload was never
authenticated, decrypted, decompressed, or deserialized.

**Impact:** A corrupted or cross-wired workflow/event record could cause one
identity to consume plaintext prepared from another byte sequence. This weakens
payload integrity, hides storage or transport corruption, and makes the failure
dependent on cache timing.

**TypeScript GREEN characterization:**
`packages/core/src/replay-payload-cache-security.test.ts` proves the existing
workflow-run and event-key behavior remains characterized and green: conflicting
bytes resolve from the first cached payload, the preparer runs once, and the
second byte sequence is not prepared.

**Rust fix:** Each production Rust binary cache cell stores the exact original
bytes. Reusing a logical key with different bytes moves the cell into a terminal
`PayloadConflict` state, wakes current waiters, returns the constant
`PAYLOAD_CONFLICT_MESSAGE`, never invokes the preparer for the conflicting
payload, and does not reflect run IDs, event IDs, or payload bytes. The original
in-flight preparation rechecks terminal state before publishing, so its later
completion cannot overwrite the integrity failure.

**Regression evidence:**

- `packages/core/src/replay-payload-cache-security.test.ts`
- `crates/workflow-core/src/replay_payload_cache.rs`
- `crates/workflow-core/tests/replay_payload_cache.rs`
- `crates/workflow-core/tests/replay_payload_cache_security.rs`
- GitHub Actions run `33516555780`, where the TypeScript characterization,
  production Rust behavior/security tests, all Rust targets, Rustfmt, Clippy,
  and differential checks passed.

**Remaining TypeScript retirement condition:** All replay payload preparation
must cross the Rust cache boundary before the permissive TypeScript cache and
its characterization test can be removed.

## Open findings tracked for later port stages

| ID | TypeScript condition | Required Rust closure |
| --- | --- | --- |
| WF-RUST-006 | Several other date and numeric schemas accept broad coercion or unconstrained numbers at wire boundaries; hook protocol versions are closed by WF-RUST-011 and step attempts by WF-RUST-012. | Inventory each remaining producer, preserve required legacy coercions, and use bounded integer/newtype validation for modern writes. |
| WF-RUST-007 | Queue telemetry uses intentionally forgiving `.catch(undefined)` behavior. | Keep telemetry non-fatal while making execution-authoritative fields strict and independently bounded. |

## WF-RUST-100: Workflow data path type and suffix confusion

TypeScript previously used `fs.access()` and raw `endsWith()` checks while
discovering local workflow data. The security regression was committed first;
TypeScript and Rust now require a directory at exact path-component boundaries.
The full evidence, impact, tests, and retirement condition are recorded in
`docs/rust-port/findings/WF-RUST-100.md`.

## WF-RUST-101: Machine-name control-character log injection

The pre-fix TypeScript formatter emitted parsed and legacy machine names
verbatim into log-oriented strings. The committed regression is observed RED
against that source. TypeScript and Rust now escape log-breaking controls while
preserving ordinary parser and display-name behavior. Full evidence and closure
requirements are in `docs/rust-port/findings/WF-RUST-101.md`.

# WF-RUST-103: Error normalization executes hooks and can mask failures

## Status

Closed at the production Rust AI error-normalization boundary.

The expected-RED workflow run `33583543050` succeeded before production implementation. It compiled and linted the translated Rust targets, then proved that both the compatibility suite and the security suite failed only at their registered source-specific panic markers.

The permanent read-only AI lane then passed in workflow run `33584024158` at commit `31426849bbd4e67771a7bc3fa6c2e915ae963f06`. That run covered both TypeScript oracle files, the translated Rust compatibility and security suites, direct production hardening tests, Rustfmt, and Clippy with warnings denied.

## Affected boundary

- `packages/ai/src/get-error-message.ts`
- `packages/ai/src/get-error-message.test.ts`
- `packages/ai/src/get-error-message-security.test.ts`
- `crates/workflow-ai/src/lib.rs`
- `crates/workflow-ai/tests/get_error_message_security.rs`
- `rust/tdd/workflow-ai/tests/get_error_message.rs`
- `rust/tdd/workflow-ai/tests/get_error_message_security.rs`

## Existing TypeScript behavior

The compatibility helper returns strings and `Error.message` directly, then delegates every other non-nullish value to `JSON.stringify`:

```ts
return JSON.stringify(error);
```

That preserves the historical AI SDK-compatible output for ordinary numbers, booleans, arrays, and plain objects. It also gives an object-controlled `toJSON` method execution during failure handling. `JSON.stringify` additionally throws for cyclic object graphs and BigInt values.

The TypeScript security characterization intentionally proves all three legacy behaviors:

1. an object-supplied `toJSON` callback executes and rewrites the diagnostic value;
2. a cyclic object throws a `TypeError`; and
3. a BigInt throws a `TypeError`.

Those assertions are evidence of the pre-existing behavior, not the Rust target contract.

## Security impact

Error values can originate from provider adapters, tool integrations, transport layers, or user code. Normalizing such a value occurs while another failure is already being handled. Executing a value-controlled hook at that point can introduce re-entrant side effects, mutate state, or replace the original diagnostic. Throwing on a cycle or BigInt can mask the underlying failure and turn one recoverable diagnostic path into a secondary availability failure.

The finding does not claim arbitrary code injection. A callable `toJSON` body is already executable code inside the process. The security boundary failure is that diagnostic normalization invokes it implicitly and that unsupported shapes can abort normalization instead of returning a bounded message. The closest general weakness class is CWE-755, improper handling of exceptional conditions.

## Implemented Rust behavior

Rust uses a typed, side-effect-free value model. Ordinary compatibility values retain the TypeScript contract, including object-property omission for `undefined`, array conversion of `undefined` to `null`, insertion order, and JSON string escaping.

The hardened cases intentionally diverge:

- callable-shaped fields are rendered as inert strings such as `[Function toJSON]`; the serializer never invokes the probe;
- shared object identity is tracked during traversal, and an active reference is rendered as `[Circular]`;
- BigInt input is validated as a signed decimal, canonicalized, and rendered as a stable `n`-suffixed diagnostic;
- released references receive a fixed placeholder;
- recursive composite values are bounded to 64 levels and terminate with `[Max depth exceeded]`;
- invalid BigInt text receives a fixed placeholder rather than being reflected into logs.

The production crate remains `#![forbid(unsafe_code)]`.

## TDD evidence

- `packages/ai/src/get-error-message.test.ts` is the 11-case compatibility oracle.
- `packages/ai/src/get-error-message-security.test.ts` is the three-case legacy security characterization.
- `rust/tdd/workflow-ai/tests/get_error_message.rs` translates the compatibility contract, including extra escaping and nested-`undefined` cases.
- `rust/tdd/workflow-ai/tests/get_error_message_security.rs` requires inert callbacks, cycle-safe traversal, and non-panicking BigInt handling.
- `crates/workflow-ai/tests/get_error_message_security.rs` repeats the three source hazards directly against production and adds released-reference, depth-bound, and invalid-BigInt regressions.
- Expected-RED proof: workflow run `33583543050`.
- Permanent GREEN proof: workflow run `33584024158`.

## Implementation commits

The transition was kept sequential:

- `d02af9c983336b0de23e6c17f639918102fddf71` added the hostile-value RED model.
- `da0e1d787daec687c149ee38717869e886398884` translated the security cases.
- `3d3fabca44a7868540a9a13cbf8db1d188fefa32` registered the security translation.
- `6a83b380eabb13d502adbc8a4002d7471388df21` registered its expected-RED marker.
- `36046b3d3b4ea0d17e16315074963d92ac1f00ed` corrected the compatibility harness to exercise the RED shim.
- `8c2c9a798955ae1e5cf1219ab025d8c4aae00d03` corrected the security harness to exercise the RED shim.
- `6196c1b76b2527a6a6d8ad8dbbc2b04bf9ee1552` implemented hardened production normalization.
- `f461e094db334cbaecc1442eca568b1bdb66cd94` added direct production regressions.
- `474d096cd473bb9f9e6e6ba21842109679815252` moved the compatibility translation to production.
- `9901cf78dd54cc418e663846adabc246dfb60d65` moved the security translation to production.
- `31426849bbd4e67771a7bc3fa6c2e915ae963f06` made both suites mandatory in the permanent AI lane.

## Closure requirements

This finding is closed because:

1. RED was observed only after clean compilation, formatting, and Clippy checks;
2. the TypeScript compatibility and security oracles remain executable and documented;
3. the translated compatibility and security suites execute production Rust;
4. direct production tests cover the source hazards and additional bounded-failure cases;
5. the permanent read-only AI workflow passed with warnings denied; and
6. the manifest and expected-RED registries are promoted only after the GREEN proof.

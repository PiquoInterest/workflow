# WF-RUST-102: Debug selector substring confusion exposes diagnostics

## Status

Closed at the production Rust debug boundary.

The expected-RED workflow run `33557800452` succeeded before production implementation. It proved the current TypeScript characterization, compiled the translated Rust security target, observed the exact registered panic marker, regenerated the manifest, and committed the TDD-RED security record as `33b85b2a90a30b5ddca5e788560685b2f27794dc`.

Production Rust security tests, implementation, exports, and translated-suite wiring were then committed sequentially as:

- `6bb44ae74d12af43023f422aa33f359a7c85339f`
- `585db13272de07c7f54d11e681e4536ad2a22ee3`
- `03e73e20f6fb2db979af7a34e1c2c4929fc06338`
- `97a446af6fc99f7b484fbd4d65a2ff3d65bf3cfc`
- `2859337d4a6cc1c25bd4a4a36e334acb952f0816`

Canonical Rust 1.87 formatting was applied without behavioral changes in `51f557aa1b10d2e057548751b8327525fff199d6`. The permanent read-only utility lane passed on the formatted implementation in workflow run `33558964834`. A branch-head-guarded promotion then removed the expected-RED registrations and updated the security and parity ledgers.

## Affected boundary

- `packages/utils/src/debug-log.ts`
- `packages/utils/src/debug-log-security.test.ts`
- `crates/workflow-utils/src/debug_log.rs`
- `crates/workflow-utils/tests/debug_log_security.rs`
- `rust/tdd/workflow-utils/tests/debug_log.rs`
- `rust/tdd/workflow-utils/tests/debug_log_security.rs`

## Existing TypeScript behavior

`isWorkflowDebugEnabled()` currently enables diagnostics when the complete `DEBUG` string merely contains `workflow:` or is exactly `*`:

```ts
return debug.includes('workflow:') || debug === '*';
```

This does not respect selector-token boundaries. An unrelated token such as `myworkflow:*`, and an explicit negative token inside `app:*,-workflow:*`, both enable workflow diagnostics. `debugLog()` then forwards every argument to `console.debug`.

The TypeScript security characterization intentionally proves that legacy behavior. It is evidence of the defect, not an endorsement of it.

## Security impact

Diagnostic arguments can contain run identifiers, transport details, URLs, retry context, and other operational metadata. A process that explicitly negates workflow logging, or enables an unrelated namespace containing the same substring, can therefore emit workflow diagnostics unexpectedly. This is a privacy and log-confidentiality boundary failure.

The impact is bounded by the diagnostic arguments supplied by each caller. The finding does not claim that secrets are always present, only that the gate can expose data its operator attempted not to enable.

## Implemented Rust behavior

Rust parses the selector as comma- or whitespace-delimited tokens. It enables the sink only when at least one positive token is either:

- exactly `*`; or
- prefixed with `workflow:`.

An exact `-*` token or any token prefixed by `-workflow:` disables workflow diagnostics and takes precedence over positive tokens. Larger unrelated tokens that merely contain `workflow:` do not match. Rejected selectors return before invoking any sink method.

`DebugArgument` has a custom `Debug` implementation that redacts text values and reports only the number of structured fields. This prevents assertion failures or derived sink diagnostics from echoing the payload values while preserving equality and cloning for parity tests.

## TDD evidence

- `packages/utils/src/debug-log-security.test.ts` characterizes the unsafe TypeScript acceptance and proves that arguments reach `console.debug` under an explicitly negated selector.
- `rust/tdd/workflow-utils/tests/debug_log_security.rs` requires both selectors to be rejected and requires the injected sink to remain empty.
- `crates/workflow-utils/tests/debug_log_security.rs` additionally requires negative-selector precedence, a zero-call sink, and redacted `Debug` output.
- `rust/tdd-red.d/utils-debug-log-security.json` records the exact expected-RED marker until guarded GREEN promotion.
- `rust/test-port-overrides.d/utils-debug-log-security.json` maps both TypeScript declarations to the translated Rust target.

## Closure requirements

This finding is closed only after:

1. the permanent read-only utility workflow passes the six existing compatibility tests, both translated security tests, and the three direct production security tests;
2. Rustfmt and Clippy pass with warnings denied;
3. the TypeScript characterization remains explicitly documented as an intentional security difference, or TypeScript is patched with an updated regression;
4. the base and security expected-RED registrations are removed only after the GREEN proof;
5. `security.txt`, the security rules, the logic/security ledger, and the generated test-port manifest are promoted by a branch-head-guarded workflow; and
6. the temporary promotion workflow is removed afterward in a separate commit.

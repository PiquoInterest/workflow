# WF-RUST-102: Debug selector substring confusion exposes diagnostics

## Status

TDD RED. The TypeScript characterization and translated Rust rejection tests are committed. Production Rust behavior is not yet implemented, so this entry is not a mitigation claim.

## Affected boundary

- `packages/utils/src/debug-log.ts`
- `packages/utils/src/debug-log-security.test.ts`
- `rust/tdd/workflow-utils/tests/debug_log_security.rs`
- the future production `workflow-utils` debug gate

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

## Required Rust behavior

Rust must parse the selector as comma- or whitespace-delimited tokens and enable the sink only when at least one positive token is either:

- exactly `*`; or
- prefixed with `workflow:`.

Tokens prefixed by `-` must never enable logging, and a larger unrelated token that merely contains `workflow:` must not match. Rejected selectors must not invoke any sink method or format diagnostic arguments.

The Rust implementation must also avoid logging or exposing arguments through `Debug`, panic text, or validation errors while determining whether the selector is enabled.

## TDD evidence

- `packages/utils/src/debug-log-security.test.ts` characterizes the unsafe TypeScript acceptance and proves that arguments reach `console.debug` under an explicitly negated selector.
- `rust/tdd/workflow-utils/tests/debug_log_security.rs` requires both selectors to be rejected and requires the injected sink to remain empty.
- `rust/tdd-red.d/utils-debug-log-security.json` records the exact expected-RED marker until production Rust exists.
- `rust/test-port-overrides.d/utils-debug-log-security.json` maps both TypeScript declarations to the translated Rust target.

## Closure requirements

This finding is closed only after:

1. the expected-RED Rust test has been observed failing for the registered implementation marker;
2. production Rust implements token-aware matching and silent suppression;
3. the six existing compatibility tests and both security tests pass against production Rust;
4. Rustfmt and Clippy pass with warnings denied;
5. the TypeScript characterization remains explicitly documented as an intentional security difference, or TypeScript is patched with an updated regression; and
6. `security.txt`, the security rules, the logic/security ledger, and the generated test-port manifest are promoted only after CI evidence is green.

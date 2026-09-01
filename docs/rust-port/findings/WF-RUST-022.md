# WF-RUST-022: Context-error state must remain plain and non-enumerating

**Status:** Fixed in TypeScript; Rust regression suite is TDD RED.

## Original regressions

A context-error constructor previously exposed `functionName` as an enumerable
parameter property. Node's default inspection and framework overlays therefore
printed an internal field in addition to the curated message. Separately,
terminal styling bytes placed directly in `.message` or `.stack` leaked ANSI
escape sequences into JSON logs, log drains, and persisted CBOR error payloads.
A multiline message could also make lazy pretty rendering duplicate its framed
detail lines.

## Security and observability impact

Error state crosses trust and persistence boundaries. Hidden implementation
fields can disclose identifiers the public error contract did not intend to
serialize. Terminal control bytes in stored messages can corrupt log rendering
or produce misleading output when replayed by another terminal. Duplicate
framing obscures the actual first stack frame during incident response.

## Required Rust invariant

Rust context errors must store only plain text and explicitly selected
serializable fields. The function/API name may appear in the human message but
must not become an additional enumerable field. Styling belongs exclusively to
a lazy terminal renderer. Context violations are fatal because replay cannot
change the calling context, and helper-generated stacks must begin at the user
call site rather than an internal gate.

## Regression evidence

- TypeScript source: `packages/core/src/context-errors.test.ts`.
- Rust expected-RED translation: `rust/tdd/workflow-core/tests/context_errors.rs`.

The Rust suite separately checks message and stack bytes, serialized fields,
pretty rendering, duplicate details, all four fatal context variants, and stack
redirection.

## Closure condition

This finding closes for Rust only when the production error type, log/CBOR
serialization, terminal renderer, fatal classification, and caller-stack helper
pass the translated suite and a round-trip test proves no ANSI or hidden field
enters persisted error data.

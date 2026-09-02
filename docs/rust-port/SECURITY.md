# Rust migration security rules

These rules apply to every new Rust crate in the migration.

1. `unsafe` is forbidden by default. A future exception requires a dedicated
   threat analysis, tests, and review; it must not be introduced only for
   performance.
2. Untrusted input is bounded before allocation or recursion. The conformance
   process currently caps one request at 1 MiB.
3. Execution-authoritative fields fail closed. Telemetry and explicitly
   forward-compatible metadata may degrade to absent only where the TypeScript
   contract already requires non-fatal parsing.
4. Protocol variants use enums or newtypes so World-only events, terminal-state
   combinations, and legacy/modern payload representations cannot be confused.
5. Length checks preserve the original unit: JavaScript UTF-16 code units where
   public compatibility requires `String.length`, UTF-8 bytes where wire size is
   the contract, and bounded integers for counters, slots, and protocol-version
   attestations.
6. No path, URL, command, SQL fragment, header, queue name, or identifier is
   trusted because it originated in another package. Validate at the boundary
   that consumes it.
7. Race behavior is part of correctness. Event-slot allocation, replay
   idempotency, hook resume, step ownership, cancellation, and queue retries
   require concurrency tests, not only single-threaded unit tests.
8. Dependencies are kept minimal and checked in CI. Cryptographic and
   serialization implementations should use established crates rather than
   custom primitives.
9. Sensitive payloads must never be emitted through `Debug`, logs, validation
   messages, telemetry, or panic text.
10. TypeScript is removed only after Rust-only E2E and upgrade/rollback tests
    pass. A fallback that silently re-enters TypeScript does not count as Rust
    parity.
11. Live capability attestations are response-only values. They use types and
    schemas distinct from persistent entities, and every response-to-storage
    conversion must discard them. A cached capability must never survive a
    backend rollback, downgrade, or kill switch.
12. Cache keys for authenticated or execution-authoritative payloads bind to an
    immutable copy of the exact bytes. Same-key/different-bytes reuse is a
    terminal integrity conflict, and diagnostics must be constant and redacted.
    Cache entries are published before extensible preparation code runs, and
    same-key preparer re-entry must fail without duplicate work or recursion.

## CI integrity

Migration validation is read-only. CI may generate and upload canonical
formatting or oracle snapshots for diagnosis, but it must not commit, push, or
rewrite source code. Formatting is enforced with `cargo fmt -- --check`, so the
checked commit, rather than an uncommitted runner mutation, is what compilation,
Clippy, and differential tests validate.

Every third-party action is pinned to a full commit SHA. Workflow-level token
permissions stay at `contents: read`; a future write permission requires a
separate, narrowly scoped workflow and threat review. TypeScript workspace
packages used as compatibility oracles are built explicitly when their package
exports point at generated `dist` files, rather than bypassing the published
package boundary with source-only aliases.

## Filesystem discovery boundaries

A path is not a trusted directory merely because it is accessible or its text
ends with a familiar suffix. Discovery code must validate the filesystem object
type and compare complete path components. Security regressions must cover
regular files, suffix lookalikes, normalization, and platform-root termination.
WF-RUST-100 applies this rule to workflow-data discovery in TypeScript and Rust.

## Log-oriented identifier rendering

Identifiers read from workflow state, adapters, or legacy records remain
untrusted when rendered for logs. Single-line formatters must escape C0, C1,
DEL, CR, LF, tab, U+2028, and U+2029 rather than allowing terminal controls or
line injection. WF-RUST-101 applies this rule to machine workflow and step names.

## Debug selector gating and diagnostic redaction

Debug namespace configuration is untrusted text. Match complete comma- or
whitespace-delimited tokens, apply explicit negative selectors before positive
selectors, and return before formatting or forwarding arguments when disabled.
Diagnostic wrapper types must not expose payload values through derived or
custom `Debug` output. WF-RUST-102 applies this rule to workflow utility logs.

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
   the contract, and bounded integers for counters/slots.
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

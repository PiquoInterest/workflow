# Repository-wide Rust migration

This directory is the control plane for replacing the TypeScript runtime in
this repository with Rust without changing observable behavior.

## Authority

The migration started from `main` commit
`855e47990c0da35419325da27976bae925afb0e9`. TypeScript remains the behavioral
oracle until each row in [PARITY_MATRIX.md](./PARITY_MATRIX.md) is proven by
both direct test ports and differential tests. Rust hardening that intentionally
changes unsafe behavior is recorded in
[TYPESCRIPT_LOGIC_AND_SECURITY_FIXES.md](./TYPESCRIPT_LOGIC_AND_SECURITY_FIXES.md).

A source file being translated is not parity. A component is complete only when:

1. Its public inputs, outputs, errors, ordering, persistence effects, retry
   behavior, and compatibility behavior are represented in Rust.
2. Existing TypeScript tests have a Rust equivalent or execute both
   implementations with the same fixtures.
3. Relevant end-to-end and race tests pass with Rust selected.
4. Intentional behavior changes have a regression test and a ledger entry.
5. No TypeScript fallback is used by the tested path.

## Layout

- `crates/workflow-world`: Rust implementation of the shared World protocol and
  validation contracts.
- `rust/conformance`: differential tests that execute TypeScript and Rust with
  identical inputs.
- `.github/workflows/rust-port.yml`: compiler, Clippy, Rust tests, TypeScript
  oracle tests, and differential parity checks.

## Current test commands

```bash
cargo test -p workflow-world --all-targets
cargo clippy -p workflow-world --all-targets -- -D warnings
cargo build -p workflow-world --example conformance
pnpm --filter @workflow/world test
WORKFLOW_RUST_CONFORMANCE_BIN=target/debug/examples/conformance \
  pnpm exec vitest run rust/conformance/world-parity.test.ts
```

## Migration order

The dependency direction determines the order:

1. World wire contracts and validation.
2. Serialization, event log, replay, and deterministic runtime primitives.
3. Local and Postgres Worlds, followed by the remote Vercel World client.
4. Queue/HTTP boundaries and framework adapters.
5. CLI/build tooling and JavaScript compatibility bindings.
6. Full workbench E2E, race, soak, and upgrade/rollback matrices.
7. Removal of TypeScript runtime implementations after the Rust-only gates are
   green.

The existing Rust SWC plugin remains in place and is not counted as a port of
the runtime.

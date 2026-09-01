# Test-first Rust migration

The Rust migration ports tests before production behavior. TypeScript remains
the executable compatibility oracle until the corresponding Rust-only suite is
green and the TypeScript runtime can be retired safely.

## Corpus lock

`scripts/rust-port-test-inventory.mjs` scans tracked files with `git ls-files`,
records every TypeScript `*.test.*` and `*.spec.*` source, hashes the exact file
bytes, and writes the checked-in `rust/test-port-manifest.json`.

At the start of this phase the repository contains:

- 334 tracked TypeScript test files;
- 5,100 source-level `it(...)` and `test(...)` declarations;
- 25 package, workbench, E2E, and migration-harness surfaces.

Parameterized tests can execute more cases than the declaration count, so the
number is a lower-bound inventory rather than a runtime assertion count.

The manifest check fails when a test is added, renamed, deleted, or edited
without an explicit Rust-port review. Deleted tests are never dropped
silently.

## Current progress

At branch commit `cfc3e6e79b07c143016e182e066419aa868ec78f` the checked-in manifest records:

- 334 TypeScript test files and 5,100 declared tests in the locked corpus;
- 40 source files fully translated into expected-RED Rust suites;
- 12 source files green against production Rust behavior;
- 282 source files not yet translated;
- no entries marked partial or blocked.

The latest AI tranche translates all 66 declarations from:

- `packages/ai/src/agent/do-stream-step.test.ts`;
- `packages/ai/src/agent/tools-to-model-tools.test.ts`;
- `packages/ai/src/agent/telemetry.test.ts`;
- `packages/ai/src/agent/stream-text-iterator.test.ts`.

The translated suites cover provider-stream normalization, malformed tool-call
input retention, partial metadata merging, model-tool projection, telemetry
parity, telemetry privacy controls, provider metadata retention, reasoning
ordering, and dynamic system-message replacement. These tests remain
intentionally RED. They do not count as production implementation or as closed
security findings.

Large generated registries are split into reviewed fragments:

- `rust/tdd-red.d/*.json` registers source-specific expected-RED commands and
  failure markers;
- `rust/test-port-overrides.d/*.json` records the Rust test files, review notes,
  and status for each translated TypeScript source.

The runner and manifest bootstrap load fragments in deterministic filename
order and reject duplicate source paths. This lets each migration chunk remain
small without weakening the complete-corpus checks.

## Statuses

- `unported`: inventoried, but no Rust test target exists yet.
- `partial`: a real Rust test target exists, but not every behavior in the
  TypeScript source has been translated.
- `red`: every claimed assertion has been translated and the Rust target is
  intentionally failing because production behavior is absent.
- `green`: the translated Rust target passes against production Rust behavior.
- `blocked`: the test requires an external deployment, backend, browser, or
  framework harness that has not yet been reproduced in Rust.

A non-`unported` or non-`blocked` entry must reference an existing Rust test
file. A `red` entry must also appear in the base `rust/tdd-red.json` file or a
reviewed `rust/tdd-red.d/*.json` fragment.

## RED verification

`scripts/run-rust-tdd-red.mjs` executes every recorded RED command separately.
A RED test is accepted only when:

1. the process exits unsuccessfully;
2. the output contains its exact recorded failure marker;
3. it is not terminated by a signal; and
4. it does not unexpectedly pass.

Before any expected failure is accepted, CI runs Rustfmt, compiles every test
target without executing it, and runs Clippy with warnings denied. This keeps a
syntax error, missing dependency, type error, or lint failure from being
misreported as a successful TDD RED state.

This distinguishes a missing implementation from compilation failures,
misconfigured runners, missing dependencies, timeouts, and unrelated panics.
An unexpected pass fails the RED lane and requires moving the test into the
normal green Rust suite.

## Per-slice order

1. Capture the exact TypeScript test source hash in the manifest.
2. Translate fixtures, assertions, error boundaries, and platform conditions to
   Rust without implementing production behavior.
3. Commit and observe the Rust target RED for the recorded reason.
4. Implement the smallest production slice needed to turn it green.
5. Run Rust unit tests, Clippy, TypeScript oracle tests, and differential tests.
6. Change the manifest status to `green` only after CI proves the transition.
7. Record any intentional logic or security correction in
   `TYPESCRIPT_LOGIC_AND_SECURITY_FIXES.md` and `security.txt`.

## Security findings

For a TypeScript security defect, the TypeScript characterization test must
continue to prove the old unsafe acceptance unless the TypeScript compatibility
layer is intentionally patched. The Rust test proves rejection or safe
normalization. Differential tests cover valid inputs, while intentional
security differences are named and documented instead of being hidden as
parity failures.

Security-sensitive requirements that are only translated, but not implemented,
are explicitly labeled `TDD RED` in `security.txt`. A panic marker proves that
the intended implementation is still absent. It never counts as mitigation.

## Retirement gate

TypeScript cannot be deprecated merely because every source file has a Rust test
target. Every applicable test must be green in a Rust-only run, external harness
blocks must be resolved, and the persistence, replay, race, framework, upgrade,
rollback, and security matrices must pass with TypeScript runtime fallback
disabled.

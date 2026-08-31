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
file. A `red` entry must also appear in `rust/tdd-red.json`.

## RED verification

`scripts/run-rust-tdd-red.mjs` executes every recorded RED command separately.
A RED test is accepted only when:

1. the process exits unsuccessfully;
2. the output contains its exact recorded failure marker;
3. it is not terminated by a signal; and
4. it does not unexpectedly pass.

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

## Retirement gate

TypeScript cannot be deprecated merely because every source file has a Rust test
target. Every applicable test must be green in a Rust-only run, external harness
blocks must be resolved, and the persistence, replay, race, framework, upgrade,
rollback, and security matrices must pass with TypeScript runtime fallback
disabled.

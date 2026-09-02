# WF-RUST-094: Route isolation harness must reject marker spoofing and preload state

**Status:** TypeScript characterization retained; Rust regression suite is TDD RED.

## Affected surface

- `packages/core/e2e/route-bundle-isolation.test.ts`
- the isolated Next Turbopack route harness
- Rust replacement process runner and world-initialization boundary

## Original behavior

The TypeScript harness searches stdout with `line.includes(RESULT_MARKER)` and
uses the last matching line. Output emitted by the route bundle is untrusted by
the harness, so an unrelated line containing the marker as a substring can be
selected as the authoritative JSON record. Multiple exact records are also not
rejected.

The supposedly fresh Node subprocess inherits the parent environment. In
particular, `NODE_OPTIONS=--require ...` or `NODE_PATH` can preload code that
registers a workflow world before the route bundle loads. That recreates the
same masking condition the isolation test was introduced to eliminate.

## Security and correctness impact

A noisy, compromised, or simply buggy route dependency can forge the harness
result or initialize global state outside the bundle under test. The test can
then pass while the production cold-start path still fails with the Turbopack
dynamic-require stub or with an uninitialized world runtime.

## Required Rust invariant

The Rust harness must:

1. direct-exec `pnpm build` and the Node harness without a shell;
2. strip Vercel selection variables and `NODE_OPTIONS`/`NODE_PATH`;
3. force deterministic non-colored output;
4. run both processes in killable process groups with hard deadlines and output
   caps;
5. accept exactly one line whose first bytes are the result marker;
6. ignore marker substrings in other output and reject duplicate or empty exact
   result records;
7. reject harness errors, the dynamic-require failure, missing world
   initialization, wrong status, and any body other than the clean missing-hook
   outcome.

## Regression evidence

`rust/tdd/workflow-core/tests/route_bundle_isolation.rs` preserves the source
scenario and adds independent environment, marker, duplicate, output, and result
validation tests.

## Closure condition

The finding closes only when the real Rust production build, sanitized process
launcher, isolated route loader, bounded stdout parser, and workflow-world
initialization path pass the translated suite. Fixture-only `HarnessResult`
values do not satisfy the integration contract.

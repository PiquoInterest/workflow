# WF-RUST-091: Stranded HMR registrations can poison a shared dev server

**Status:** TDD RED for the Rust builder and development watcher.

## Affected surface

- `packages/core/e2e/dev.test.ts`
- generated step registrations
- Next flow-route development builds
- Rust replacement file watcher, builder, manifest writer, and test cleanup

## Original failure mode

A generated step-registration artifact imports every discovered step file by
path. Deleting a fixture is therefore not enough to repair the development
server. If the watcher misses the unlink, the generated artifact keeps importing
a path that no longer exists. The flow route then fails to compile and every
later workflow dispatch in the shared test job returns 500, even though those
later tests did not touch the deleted fixture.

The TypeScript suite contains an explicit recovery: wait for every deleted path
to disappear from the generated registration, restore only paths that remain
stranded if convergence times out, and fail at the cleanup boundary where the
cause is still diagnosable.

## Security and correctness impact

This is an availability and stale-code boundary. A missed or reordered file
event can poison a long-lived developer process, mask unrelated regressions,
and leave generated code referring to state that no longer exists. Unbounded
prewarm or convergence calls can also hang CI indefinitely when a dev server is
wedged.

## Rust contract

`rust/tdd/workflow-core/tests/dev_hmr.rs` preserves all nine top-level source
cases. The source manifest counts only the two plain `test(...)` calls and misses
seven conditional `test.runIf(...)` registrations, so the Rust manifest override
records the corrected count. The final fuzz case is split into 13 mutations:
body-only skip paths, workflow/serde hot rebuilds, definition/import-graph full
rediscovery, workflow-file add/remove, and unrelated-file add/remove.

The suite also requires:

- 5-second bounds on every prewarm/trigger request;
- platform and canary-specific teardown and rediscovery budgets;
- UTF-16LE BOM and null-byte log decoding;
- exact log counts on stable and lower bounds on canary;
- rebuild completion plus a 2-second quiet period before opening a log cursor;
- bounded polling that preserves the last diagnostic;
- restoration of only still-imported deleted files;
- failure when generated workflow output is absent.

## Closure condition

The finding closes only when the real Rust watcher, rediscovery graph, builder,
step-registration writer, manifest writer, workflow execution path, log reader,
and RAII/finally-equivalent cleanup pass the translated tests. A table-driven
function that merely returns the expected observation is not sufficient.

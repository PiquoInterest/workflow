# WF-RUST-092: Corrupt manifests can be mistaken for absent manifests

**Status:** TDD RED for the Rust build and workbench matrix.

## Affected surface

- `packages/core/e2e/manifest.test.ts`
- generated `manifest.json` artifacts
- Rust builder and framework integrations

## TypeScript behavior

`tryReadManifest()` wraps path resolution, file reading, and `JSON.parse()` in a
single broad `catch` and returns `null` for every failure. Callers interpret
`null` as an optional project whose manifest does not exist and return early.
A malformed JSON document, permission error, or other read failure therefore
silently skips all structure and graph assertions just like `ENOENT`.

## Impact

A corrupted or unreadable artifact can make CI appear green without validating
the manifest consumed by deployment tooling. That weakens the migration oracle
and can hide malformed step/workflow identifiers, missing control-flow metadata,
or graphs that cannot be rendered or executed correctly.

## Rust correction

The Rust contract has a typed read gate:

- `NotFound` becomes `Ok(None)` and preserves the intended optional-project skip.
- parse failures remain parse failures;
- permission and other I/O failures remain I/O failures.

The translated matrix expands the six parameterized declarations into 36
independent tests across Next webpack, Next turbopack, Nitro, Vite, SvelteKit,
Nuxt, Hono, and Express. It preserves manifest version 1.0.0, step/workflow ID
shape, graph node and edge fields, start/end nodes for non-empty graphs,
dot-directory discovery, conditional Then/Else metadata, loop IDs, and loop
back-edges.

## Closure condition

The finding closes only when the production Rust builders emit real manifest
JSON for every applicable framework, the Rust reader parses those files, and
the 36-case matrix passes without fixture-only observations or broad error
suppression.

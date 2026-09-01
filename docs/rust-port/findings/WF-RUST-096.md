# WF-RUST-096: Local deployment detection must parse the hostname

**Status:** Rust correction specified; regression suite is TDD RED only for the runtime-integrated helpers.

## Affected surface

- `packages/core/e2e/utils.ts`
- source-map capability gating in the E2E suite
- Rust replacement deployment URL classifier

## Original behavior

`isLocalDeployment()` classifies a deployment as local when the complete URL
string contains `localhost` or `127.0.0.1`. Hostname lookalikes such as
`https://localhost.attacker.example`, path segments, user-info, or query values
can therefore select the local branch even though the request targets a remote
host.

## Impact

The current use is test-capability selection rather than production routing, but
an incorrect classification can enable local-only source-map expectations and
hide the actual remote-deployment matrix. Reusing the helper for a stronger
trust decision would be unsafe.

## Rust correction

The Rust contract extracts the URL authority and compares the parsed hostname
exactly, case-insensitively for `localhost`, or exactly to `127.0.0.1`. It does
not accept suffixes, path text, query text, or user-info matches.

## Regression evidence

`rust/tdd/workflow-core/tests/e2e_utils.rs` retains the six existing source-map
matrix cases and adds remote lookalike and query-string regressions.

## Closure condition

The correction closes when every Rust caller uses the exact-host classifier and
the cross-language differential suite records the intentional divergence from
the TypeScript substring helper.

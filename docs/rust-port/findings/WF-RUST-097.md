# WF-RUST-097: Producer capability negotiation must fail closed

**Status:** Rust contract implemented; runtime adoption and CI proof pending.

## TypeScript reference behavior

`packages/core/src/capabilities.ts` chooses the serialization and stream
representations that a target workflow run can decode. Missing or invalid
`workflowCoreVersion` metadata receives only the baseline `devl` format and raw
byte streams. Optional features become available at explicit semantic-version
cutoffs.

The TypeScript implementation is the correct compatibility oracle. This finding
records a security-sensitive migration invariant rather than a TypeScript flaw.

## Security and correctness impact

A false-positive capability is more dangerous than a false negative. If a new
producer treats malformed or legacy metadata as modern, it can send encrypted,
compressed, sealed, or length-framed bytes to a consumer that cannot decode
them. The result is durable payload loss, replay failure, or an availability
incident. A conservative false negative only delays an optimization.

## Rust implementation

`crates/workflow-core/src/capabilities.rs`:

- uses the `semver` crate for real prerelease ordering;
- matches npm-semver normalization for surrounding whitespace and one lower-case
  `v` prefix;
- rejects raw inputs above npm semver's 256 UTF-16-code-unit limit;
- rejects major, minor, or patch components above
  `Number.MAX_SAFE_INTEGER`;
- always includes `devl`;
- enables `encr` at `4.2.0-beta.64`;
- enables framed byte streams at `5.0.0-beta.15`;
- enables `gzip` and `zstd` together at `5.0.0-beta.18`;
- enables `encp` at `5.0.0-beta.37`;
- otherwise fails closed without reflecting untrusted metadata in an error.

## Regression evidence

- TypeScript oracle: `packages/core/src/capabilities.test.ts`
- Direct Rust tests: `crates/workflow-core/tests/capabilities.rs`
- Differential tests: `rust/conformance/capabilities-parity.test.ts`
- Dedicated CI: `.github/workflows/rust-core.yml`

## Closure condition

The contract becomes fully closed only after the branch workflow passes and
every Rust producer that selects serialization or byte-stream framing uses this
single implementation. Until then, runtime adoption remains a separate parity
gate.

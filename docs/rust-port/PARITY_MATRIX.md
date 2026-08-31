# Rust parity matrix

Status meanings:

- **PROVEN**: direct tests and TypeScript/Rust differential fixtures pass.
- **IMPLEMENTED, CI PENDING**: code and tests are committed but have not yet
  passed the branch workflow.
- **PARTIAL**: only a bounded subset is implemented.
- **NOT STARTED**: TypeScript remains the only implementation.

| Surface | Rust status | Evidence / remaining work |
| --- | --- | --- |
| World attribute validation and materialization | IMPLEMENTED, CI PENDING | Key/value limits, UTF-16/UTF-8 length semantics, reserved keys, duplicate batches, exact post-merge count, immutable application, and prototype-key regression tests. |
| World environment tuning | IMPLEMENTED, CI PENDING | Number/flag parsing, fallback, clamping, integer mode, warning deduplication, and max-event fallback. |
| Event type categories and metadata | IMPLEMENTED, CI PENDING | All current event names, World-only event restriction, replay classes, terminal classes, data-ref fields, lazy child creation, and resolve-data stripping. |
| Slot event identity | IMPLEMENTED, CI PENDING | Fixed-width ids, safe-integer bound, legacy prefixes, parsing, required-slot failure, and ULID separation. |
| Spec-version negotiation | IMPLEMENTED, CI PENDING | Versions 1 through 7, sealed-log kill switch, legacy and newer-reader predicates. |
| Queue naming and payload envelope | PARTIAL | Prefix/name contracts and core invoke/probe envelopes are implemented. Remaining queue timing, all transport representations, handler interfaces, and backend delivery behavior are not yet ported. |
| Run contracts | PARTIAL | Status predicates and bulk-cancel requests/results are implemented. Full run entity schemas, queries, cancellation APIs, and wait-for-terminal behavior remain. |
| Serialized data contract | PARTIAL | Legacy-versus-modern representation and strict modern binary validation are implemented. Devalue hydration, compression, encryption envelopes, and CBOR transport remain. |
| ULID utilities | IMPLEMENTED, CI PENDING | Exact run-id shape, Crockford parsing, slot rejection, timestamp decoding, and asymmetric skew validation. |
| Steps, hooks, waits, analytics | NOT STARTED | Port entity schemas, state invariants, request/response contracts, and tests. |
| World interfaces | NOT STARTED | Port storage, queue, streamer, encryption, environment, and lifecycle traits. |
| world-local | NOT STARTED | Port filesystem persistence, queue, locking, event slots, recovery, and all local-world tests. |
| world-postgres | NOT STARTED | Port SQL state transitions, migrations/client boundary, workers, slot arbitration, and concurrency tests. |
| world-vercel | NOT STARTED | Port remote protocol client, errors, regional routing, encryption-key and queue behavior. |
| Core deterministic replay runtime | NOT STARTED | Port event consumer, replay, discontinuations, workflow/step/hook/wait primitives, retries, cancellation, streams, and encryption. |
| Compiler/build pipeline | PARTIAL | Existing SWC plugin is already Rust. JavaScript builders, Rollup/Vite/TypeScript plugins, manifest generation, and bindings remain. |
| Framework integrations | NOT STARTED | Next, Nitro, Nuxt, SvelteKit, Astro, Nest, and remaining adapters. |
| CLI and workbenches | NOT STARTED | Port CLI behavior and make all examples execute through the Rust runtime. |
| Repository-wide Rust-only E2E | NOT STARTED | Existing package tests, workbench E2E, race reproduction, upgrade/rollback, and security tests must pass with TypeScript runtime disabled. |
| TypeScript runtime deletion | BLOCKED | Allowed only after every applicable row is PROVEN and Rust-only CI is green. |

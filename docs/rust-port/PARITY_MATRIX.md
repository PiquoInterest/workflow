# Rust parity matrix

Status meanings:

- **PROVEN**: direct tests and TypeScript/Rust differential fixtures pass in
  the dedicated branch workflow.
- **IMPLEMENTED, CI PENDING**: code and tests are committed but have not yet
  passed the branch workflow.
- **PARTIAL**: only a bounded subset is implemented.
- **NOT STARTED**: TypeScript remains the only implementation.
- **BLOCKED**: a prerequisite has not reached proven parity.

| Surface | Rust status | Evidence / remaining work |
| --- | --- | --- |
| World attribute validation and materialization | PROVEN | Key/value limits, UTF-16/UTF-8 length semantics, reserved keys, duplicate batches, exact post-merge count, immutable application, and prototype-key regression tests passed at `db090ee41b0785afac9bfe771d2df869c8012b73`. |
| World environment tuning | PROVEN | Number/flag parsing, fallback, clamping, integer mode, warning deduplication, and max-event fallback passed at `db090ee41b0785afac9bfe771d2df869c8012b73`. |
| Event type categories and metadata | PROVEN | All current event names, World-only event restriction, replay classes, terminal classes, data-ref fields, lazy child creation, and resolve-data stripping passed at `db090ee41b0785afac9bfe771d2df869c8012b73`. |
| Slot event identity | PROVEN | Fixed-width ids, safe-integer bound, legacy prefixes, parsing, required-slot failure, and ULID separation passed at `db090ee41b0785afac9bfe771d2df869c8012b73`. |
| Spec-version negotiation | PROVEN | Versions 1 through 7, sealed-log kill switch, legacy and newer-reader predicates passed at `db090ee41b0785afac9bfe771d2df869c8012b73`. |
| ULID utilities | PROVEN | Exact run-id shape, Crockford parsing, slot rejection, timestamp decoding, and asymmetric skew validation passed at `db090ee41b0785afac9bfe771d2df869c8012b73`. |
| Error names, messages, fields, and retry classification | PROVEN | All exported `@workflow/errors` constructors, stable names, framed hints/docs, fatal classification, deployment mismatch diagnostics, structured fields, and TypeScript/Rust differential fixtures passed at `6d07fa883aa6d21d83d91e086b28eb8ed489a475`. Native stack/cause identity and ANSI rendering remain JavaScript-host concerns. |
| Serde custom-class registry symbols | PROVEN | Exact `Symbol.for()` registry keys and Rust protocol constants passed direct and differential tests at `6d07fa883aa6d21d83d91e086b28eb8ed489a475`. Full class hydration belongs to the serialized-data/runtime rows. |
| Retry duration parsing | PROVEN | Strings, relative milliseconds, absolute dates, default retry delay, JavaScript TimeClip truncation, invalid-date rejection, overflow rejection, and TypeScript/Rust differential fixtures passed at `6d07fa883aa6d21d83d91e086b28eb8ed489a475`. |
| Core run capability negotiation | IMPLEMENTED, CI PENDING | Production Rust preserves conservative invalid-version fallback and the exact encryption, framing, compression, and sealed-format semantic-version cutoffs. Direct tests are green; the corrected TypeScript build and differential lane are rerunning under WF-RUST-097. Runtime producer adoption remains. |
| Core step retry accounting | IMPLEMENTED, CI PENDING | Production Rust uses checked integer counting and a private advanceable-count type, preserves exact owner/racer semantics, and has direct plus differential tests under WF-RUST-098. Executor and persisted World-event adoption remain. |
| Hook resume context and backend capability contracts | IMPLEMENTED, CI PENDING | Protocol constants, positive 32-bit version validation, exact integer JSON normalization, context/capability parsing, unknown-field stripping, negative fixtures, and a persistence-safe type conversion are implemented. Full hook entities, token retention, storage methods, and resume execution remain. |
| Step lifecycle state invariants | IMPLEMENTED, CI PENDING | The TypeScript characterization suite proves `StepSchema` accepts contradictory states and unsafe finite attempt counters. Rust `StepState` rejects cross-state fields, null-smuggled forbidden fields, missing modern terminal payloads, and attempts outside the exact `0..=Number.MAX_SAFE_INTEGER` range while preserving valid states, zero-based creation, integral normalization, and legacy spec-version 1 projections. Full step request/response contracts, date coercion, serialized payload decoding, and backend adoption remain. |
| Queue naming and payload envelope | PARTIAL | Prefix/name contracts and core invoke/probe envelopes are implemented. Remaining queue timing, all transport representations, handler interfaces, and backend delivery behavior are not yet ported. |
| Run contracts | PARTIAL | Status predicates and bulk-cancel requests/results are implemented. Full run entity schemas, queries, cancellation APIs, and wait-for-terminal behavior remain. |
| Serialized data contract | PARTIAL | Legacy-versus-modern representation and strict modern binary validation are implemented. Devalue hydration, compression, encryption envelopes, and CBOR transport remain. |
| Waits and analytics | NOT STARTED | Port entity schemas, state invariants, request/response contracts, and tests. |
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

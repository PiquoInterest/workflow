# WF-RUST-100: Workflow data path type and suffix confusion

**Status:** TypeScript and Rust fixes committed; GREEN promotion requires the focused CI lane.

## Original behavior

`packages/utils/src/check-data-dir.ts` used `fs.access()` as its directory test.
Any accessible filesystem object therefore satisfied the check, including a
regular file placed at `.workflow-data` or `workflow-data`.

The direct-path classifier also used raw string `endsWith()` checks. Existing
directories named `not.workflow-data` and `not-workflow-data` were consequently
classified as trusted workflow-data directories even though the expected path
components were absent.

## Security and correctness impact

Workflow-data discovery selects the local project and storage root consumed by
CLI and inspection paths. Treating an attacker-controlled file or lookalike
directory as that root creates a path-confusion boundary: later code can read,
report, or diagnose the wrong project location while believing discovery has
validated it. This finding does not by itself prove arbitrary file read or
privilege escalation, but it invalidates the trust decision that precedes those
operations.

## TypeScript TDD evidence and fix

`packages/utils/src/check-data-dir-security.test.ts` was committed before the
implementation change. Against the previous source, its four cases fail because
regular files and suffix lookalikes receive a `dataDir`.

The TypeScript implementation now:

- requires `stat(path).isDirectory()` instead of mere accessibility;
- reconstructs each candidate from complete path components before accepting a
  direct workflow-data path;
- expands only a standalone `~` or a `~/...` path, leaving user-qualified names
  such as `~service` as literal relative paths; and
- stops parent traversal only when `dirname(current) === current`, which is
  correct for POSIX roots and Windows drive roots.

The existing 23-case compatibility suite remains unchanged and must stay green.

## Rust implementation

`crates/workflow-utils/src/check_data_dir.rs` implements the same contract with:

- `std::fs::metadata(...).is_dir()` for object-type validation;
- `Path::ends_with()` for component-aware suffix matching;
- lexical normalization before classification;
- explicit injected current-directory and home-directory context; and
- no `unsafe` code or untrusted-value reflection in errors.

The original 23 translated tests execute against the production crate through
`rust/tdd/workflow-utils/tests/check_data_dir.rs`. Additional Rust security tests
cover regular files, both suffix-lookalike forms, and literal user-qualified
tilde names.

## Regression evidence

- TypeScript compatibility:
  `packages/utils/src/check-data-dir.test.ts`
- TypeScript security:
  `packages/utils/src/check-data-dir-security.test.ts`
- Complete translated Rust compatibility suite:
  `rust/tdd/workflow-utils/tests/check_data_dir.rs`
- Rust security:
  `crates/workflow-utils/tests/check_data_dir_security.rs`
- Permanent verification lane:
  `.github/workflows/rust-utils.yml`

## Closure condition

WF-RUST-100 is GREEN only when both TypeScript suites pass, the complete
translated Rust suite passes against the production crate, the Rust security
tests pass, Rustfmt and Clippy are clean, the RED registry no longer contains
the source suite, and the checked test-port manifest reports both TypeScript
files as GREEN.

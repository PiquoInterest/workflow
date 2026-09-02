# WF-RUST-101: Machine-name control-character log injection

**Status:** TypeScript and Rust fixes committed; GREEN promotion requires focused CI.

## Original behavior

`formatStepName()` and `formatWorkflowName()` describe their output as a
single-line, human-readable log representation. The previous implementation
interpolated the parsed short name and module specifier directly, and returned
an unrecognized input verbatim.

A machine name containing CR, LF, an ANSI escape, a C0/C1 control, or a Unicode
line separator therefore emitted that character unchanged. A malformed backend
record, compromised adapter, or user-controlled legacy name could forge an
additional log line, alter terminal rendering, or hide surrounding diagnostics.

## TypeScript TDD evidence and fix

`packages/utils/src/parse-name-security.test.ts` was committed before the
production change. Against the previous implementation, the parsed-name and
fallback cases fail because their outputs contain real control characters.

The TypeScript implementation now escapes:

- line feed as `\\n`;
- carriage return as `\\r`;
- tab as `\\t`; and
- all other C0, DEL/C1, U+2028, and U+2029 characters as lowercase four-digit
  `\\uXXXX` text.

Escaping is applied to parsed short names, module specifiers, and raw fallback
names. Ordinary output remains byte-for-byte unchanged.

## Rust implementation

`crates/workflow-utils/src/parse_name.rs` ports the complete parser, formatter,
and queue-sanitized display-name behavior. It preserves the existing empty
function-part compatibility, scoped package/default-export handling, nested
function leaf names, and raw display-name fallbacks.

The Rust formatter applies the same single-line escaping contract without
`unsafe` code. It never reflects unescaped attacker-controlled controls into an
error or log-oriented return value.

## Regression evidence

- TypeScript compatibility:
  `packages/utils/src/parse-name.test.ts`
- TypeScript security:
  `packages/utils/src/parse-name-security.test.ts`
- Complete translated Rust compatibility suite:
  `rust/tdd/workflow-utils/tests/parse_name.rs`
- Production Rust security:
  `crates/workflow-utils/tests/parse_name_security.rs`
- Permanent verification lane:
  `.github/workflows/rust-utils.yml`

## Closure condition

WF-RUST-101 is GREEN only when the security test is observed failing against the
pre-fix TypeScript source, all TypeScript compatibility and security tests pass
against the fixed source, all 34 translated Rust compatibility tests pass
against the production crate, all Rust security tests pass, Rustfmt and Clippy
are clean, the parse-name RED registration is removed, and the generated
manifest reports both TypeScript suites as GREEN.

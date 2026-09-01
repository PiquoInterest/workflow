#!/usr/bin/env python3
"""Promote WF-RUST-102 after its RED and GREEN proofs have passed."""

from __future__ import annotations

import json
from pathlib import Path

BASE_TARGET = "packages/utils/src/debug-log.test.ts"
SECURITY_TARGET = "packages/utils/src/debug-log-security.test.ts"
GREEN_RUN = "33558964834"


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def promote_red_registrations() -> None:
    red_path = Path("rust/tdd-red.json")
    red = json.loads(red_path.read_text())
    matches = [case for case in red["cases"] if case.get("typescript") == BASE_TARGET]
    if len(matches) != 1:
        raise SystemExit(
            f"expected one RED registration for {BASE_TARGET}, got {len(matches)}"
        )
    red["cases"] = [
        case for case in red["cases"] if case.get("typescript") != BASE_TARGET
    ]
    write_json(red_path, red)

    security_red_path = Path("rust/tdd-red.d/utils-debug-log-security.json")
    if not security_red_path.exists():
        raise SystemExit(
            f"missing expected security RED fragment: {security_red_path}"
        )
    security_red_path.unlink()


def promote_test_mappings() -> None:
    overrides_path = Path("rust/test-port-overrides.json")
    overrides = json.loads(overrides_path.read_text())
    base_entries = [
        entry
        for entry in overrides["entries"]
        if entry.get("typescript") == BASE_TARGET
    ]
    if len(base_entries) != 1:
        raise SystemExit(
            f"expected one override for {BASE_TARGET}, got {len(base_entries)}"
        )
    base_entries[0].update(
        {
            "status": "green",
            "rustTests": [
                "rust/tdd/workflow-utils/tests/debug_log.rs",
                "crates/workflow-utils/tests/debug_log_security.rs",
            ],
            "notes": (
                "All six TypeScript selector and sink assertions execute against "
                "production workflow-utils. Rust additionally rejects substring-confused "
                "and explicitly negated workflow selectors, never invokes the sink when "
                "disabled, and redacts Debug output for diagnostic payloads. TypeScript "
                "intentionally remains an oracle for the legacy unsafe behavior documented "
                "as WF-RUST-102."
            ),
        }
    )
    write_json(overrides_path, overrides)

    fragment_path = Path("rust/test-port-overrides.d/utils-debug-log-security.json")
    write_json(
        fragment_path,
        {
            "schemaVersion": 1,
            "entries": [
                {
                    "typescript": SECURITY_TARGET,
                    "status": "green",
                    "rustTests": [
                        "rust/tdd/workflow-utils/tests/debug_log_security.rs",
                        "crates/workflow-utils/tests/debug_log_security.rs",
                    ],
                    "notes": (
                        "The two TypeScript declarations intentionally characterize the "
                        "legacy substring matcher accepting unrelated and explicitly negated "
                        "selectors and forwarding diagnostic arguments. The Rust translation "
                        "and direct production tests require token-aware rejection, zero sink "
                        "calls, negative-selector precedence, and redacted Debug output. "
                        f"Verified by read-only Rust utility workflow run {GREEN_RUN}."
                    ),
                }
            ],
        },
    )


def promote_security_record() -> None:
    security_path = Path("security.txt")
    security = security_path.read_text()
    marker = "\n#\n# WF-RUST-102:"
    index = security.find(marker)
    if index < 0:
        raise SystemExit("WF-RUST-102 security record not found")
    security = security[:index].rstrip() + f"""

#
# WF-RUST-102: TypeScript workflow debug gating uses whole-string substring
# matching, so unrelated names such as myworkflow:* and explicitly negated
# selectors such as app:*,-workflow:* still forward diagnostic arguments. The
# TypeScript characterization keeps that privacy defect reproducible. Rust
# parses comma/whitespace-delimited selector tokens, gives -* and -workflow:*
# precedence, rejects larger substring lookalikes, returns before touching the
# sink, and redacts Debug output for diagnostic values. The expected-RED proof
# and permanent GREEN run {GREEN_RUN} cover six compatibility, two translated
# security, and three direct production security tests. Full analysis is in
# docs/rust-port/findings/WF-RUST-102.md.
"""
    security_path.write_text(security)

    rules_path = Path("docs/rust-port/SECURITY.md")
    rules = rules_path.read_text()
    heading = "## Debug selector gating and diagnostic redaction"
    if heading not in rules:
        rules_path.write_text(
            rules.rstrip()
            + """

## Debug selector gating and diagnostic redaction

Debug namespace configuration is untrusted text. Match complete comma- or
whitespace-delimited tokens, apply explicit negative selectors before positive
selectors, and return before formatting or forwarding arguments when disabled.
Diagnostic wrapper types must not expose payload values through derived or
custom `Debug` output. WF-RUST-102 applies this rule to workflow utility logs.
"""
        )

    ledger_path = Path("docs/rust-port/TYPESCRIPT_LOGIC_AND_SECURITY_FIXES.md")
    ledger = ledger_path.read_text()
    heading = "## WF-RUST-102: Debug selector substring confusion"
    if heading not in ledger:
        ledger_path.write_text(
            ledger.rstrip()
            + f"""

## WF-RUST-102: Debug selector substring confusion

The TypeScript debug gate treats any `DEBUG` string containing `workflow:` as
an enable signal, including unrelated namespaces and explicit negative tokens.
The committed TypeScript characterization proves that diagnostic arguments are
forwarded under those conditions. Rust uses token-aware matching, negative
selector precedence, a zero-call disabled path, and payload-redacting `Debug`
implementations. The pre-implementation Rust target is observed RED and the
permanent utility lane is GREEN in run `{GREEN_RUN}`. Full evidence is in
`docs/rust-port/findings/WF-RUST-102.md`.
"""
        )


def close_finding_and_matrix() -> None:
    finding_path = Path("docs/rust-port/findings/WF-RUST-102.md")
    finding = finding_path.read_text()
    old_status = "## Status\n\nImplementation complete; permanent GREEN validation pending."
    new_status = "## Status\n\nClosed at the production Rust debug boundary."
    if old_status not in finding:
        raise SystemExit("WF-RUST-102 pending status was not found")
    finding = finding.replace(old_status, new_status, 1)

    old_evidence = (
        "This document does not claim closure until the permanent read-only utility "
        "lane passes on that formatted implementation and a separate guarded "
        "promotion updates the security and parity ledgers."
    )
    new_evidence = (
        f"The permanent read-only utility lane passed on the formatted implementation "
        f"in workflow run `{GREEN_RUN}`. A branch-head-guarded promotion then removed "
        "the expected-RED registrations and updated the security and parity ledgers."
    )
    if old_evidence not in finding:
        raise SystemExit("WF-RUST-102 pending evidence text was not found")
    finding_path.write_text(finding.replace(old_evidence, new_evidence, 1))

    matrix_path = Path("docs/rust-port/PARITY_MATRIX.md")
    matrix = matrix_path.read_text()
    if "WF-RUST-102" not in matrix:
        matrix_path.write_text(
            matrix.rstrip()
            + f"""

## Utility security promotions

| Contract | Status | Evidence |
| --- | --- | --- |
| Debug selector gating and diagnostic redaction | PROVEN | TypeScript legacy characterization plus production and translated Rust tests; read-only workflow run `{GREEN_RUN}`; WF-RUST-102 |
"""
        )


def main() -> None:
    promote_red_registrations()
    promote_test_mappings()
    promote_security_record()
    close_finding_and_matrix()


if __name__ == "__main__":
    main()

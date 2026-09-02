#!/usr/bin/env python3
"""Promote the verified get-port utility suite to GREEN."""

from __future__ import annotations

import json
from pathlib import Path

TARGET = "packages/utils/src/get-port.test.ts"
RUST_TESTS = ["rust/tdd/workflow-utils/tests/get_port.rs"]
NOTES = (
    "All 20 TypeScript listener discovery, deterministic ordering, restart, "
    "concurrency, workflow-health probing, timeout, IPv6, fallback, and Windows "
    "netstat declarations execute against production workflow-utils. Rust "
    "additionally rejects partial or out-of-range Windows ports and control-byte "
    "injection in custom HTTP probe endpoints before network use. Verified by "
    "read-only utility workflow run 33580063231."
)


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def remove_red_registration() -> None:
    path = Path("rust/tdd-red.json")
    document = json.loads(path.read_text())
    matches = [case for case in document["cases"] if case.get("typescript") == TARGET]
    if len(matches) != 1:
        raise SystemExit(
            f"expected exactly one RED registration for {TARGET}, got {len(matches)}"
        )
    document["cases"] = [
        case for case in document["cases"] if case.get("typescript") != TARGET
    ]
    write_json(path, document)


def promote_override() -> None:
    path = Path("rust/test-port-overrides.json")
    document = json.loads(path.read_text())
    matches = [entry for entry in document["entries"] if entry.get("typescript") == TARGET]
    if len(matches) != 1:
        raise SystemExit(f"expected one test-port override for {TARGET}, got {len(matches)}")
    entry = matches[0]
    if entry.get("status") != "red":
        raise SystemExit(f"expected RED override for {TARGET}, got {entry.get('status')}")
    entry.update({"status": "green", "rustTests": RUST_TESTS, "notes": NOTES})
    write_json(path, document)


def update_parity_matrix() -> None:
    path = Path("docs/rust-port/PARITY_MATRIX.md")
    text = path.read_text()
    marker = "## Utility process-port discovery promotion"
    if marker in text:
        return
    path.write_text(
        text.rstrip()
        + """

## Utility process-port discovery promotion

| Contract | Status | Evidence |
| --- | --- | --- |
| Process listener discovery and workflow endpoint probing | PROVEN | 20 translated TypeScript declarations plus strict port parsing and request-target injection regressions; read-only workflow run `33580063231` |
"""
    )


def main() -> None:
    remove_red_registration()
    promote_override()
    update_parity_matrix()


if __name__ == "__main__":
    main()

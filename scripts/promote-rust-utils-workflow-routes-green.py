#!/usr/bin/env python3
"""Promote the verified workflow-routes utility suite to GREEN."""

from __future__ import annotations

import json
from pathlib import Path

TARGET = "packages/utils/src/workflow-routes.test.ts"
RUST_TESTS = ["rust/tdd/workflow-utils/tests/workflow_routes.rs"]
NOTES = (
    "All three TypeScript flow, health, configured-base-path, and retired-step "
    "declarations execute against production workflow-utils. Rust additionally "
    "proves manifest construction, percent-encoded webhook tokens, query and "
    "fragment replacement, trailing-slash normalization, and rejection of "
    "relative or whitespace-confused base URLs. Verified by read-only utility "
    "workflow run 33579321645."
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
    marker = "## Utility workflow route promotion"
    if marker in text:
        return
    path.write_text(
        text.rstrip()
        + """

## Utility workflow route promotion

| Contract | Status | Evidence |
| --- | --- | --- |
| Workflow URL, health, manifest, and webhook construction | PROVEN | 3 translated TypeScript declarations plus route-normalization and token-encoding regressions; read-only workflow run `33579321645` |
"""
    )


def main() -> None:
    remove_red_registration()
    promote_override()
    update_parity_matrix()


if __name__ == "__main__":
    main()

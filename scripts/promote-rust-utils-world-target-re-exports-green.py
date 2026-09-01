#!/usr/bin/env python3
"""Promote the verified world-target and re-export utility suites to GREEN."""

from __future__ import annotations

import json
from pathlib import Path

TARGETS = {
    "packages/utils/src/world-target.test.ts": {
        "rustTests": [
            "rust/tdd/workflow-utils/tests/world_target.rs",
            "crates/workflow-utils/tests/world_target.rs",
        ],
        "notes": (
            "All ten TypeScript environment-resolution declarations execute against "
            "production workflow-utils. Direct production tests additionally pin empty "
            "environment values and exact Vercel target matching so prefix lookalikes are "
            "not accepted. Verified by read-only utility workflow run 33560481078."
        ),
    },
    "packages/utils/src/re-exports.test.ts": {
        "rustTests": [
            "rust/tdd/workflow-utils/tests/re_exports.rs",
            "crates/workflow-utils/tests/re_exports.rs",
        ],
        "notes": (
            "All three TypeScript crate-root re-export declarations execute against the "
            "production workflow-utils parse-name exports. Verified by read-only utility "
            "workflow run 33560481078."
        ),
    },
}


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def remove_red_registrations() -> None:
    path = Path("rust/tdd-red.json")
    document = json.loads(path.read_text())
    for target in TARGETS:
        matches = [case for case in document["cases"] if case.get("typescript") == target]
        if len(matches) != 1:
            raise SystemExit(
                f"expected exactly one RED registration for {target}, got {len(matches)}"
            )
    document["cases"] = [
        case for case in document["cases"] if case.get("typescript") not in TARGETS
    ]
    write_json(path, document)


def promote_overrides() -> None:
    path = Path("rust/test-port-overrides.json")
    document = json.loads(path.read_text())
    by_path = {entry.get("typescript"): entry for entry in document["entries"]}
    for target, replacement in TARGETS.items():
        entry = by_path.get(target)
        if entry is None:
            raise SystemExit(f"missing test-port override for {target}")
        entry.update(
            {
                "status": "green",
                "rustTests": replacement["rustTests"],
                "notes": replacement["notes"],
            }
        )
    write_json(path, document)


def update_parity_matrix() -> None:
    path = Path("docs/rust-port/PARITY_MATRIX.md")
    text = path.read_text()
    marker = "## Utility target and export promotions"
    if marker in text:
        return
    path.write_text(
        text.rstrip()
        + """

## Utility target and export promotions

| Contract | Status | Evidence |
| --- | --- | --- |
| Workflow world-target resolution | PROVEN | 10 translated TypeScript tests plus direct exact-match regressions; read-only workflow run `33560481078` |
| Parse-name crate-root re-exports | PROVEN | 3 translated TypeScript tests plus direct production re-export tests; read-only workflow run `33560481078` |
"""
    )


def main() -> None:
    remove_red_registrations()
    promote_overrides()
    update_parity_matrix()


if __name__ == "__main__":
    main()

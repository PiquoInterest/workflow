#!/usr/bin/env python3
"""Promote verified global-singleton and promise utility suites to GREEN."""

from __future__ import annotations

import json
from pathlib import Path

TARGETS = {
    "packages/utils/src/global-singleton.test.ts": {
        "rustTests": ["rust/tdd/workflow-utils/tests/global_singleton.rs"],
        "notes": (
            "All nine TypeScript sharing, factory-once, mutation, process-registry, "
            "shape-version, name-isolation, and reset declarations execute against "
            "production workflow-utils. Rust additionally proves concurrent callers run "
            "one factory and that a name/version reused with another concrete type fails "
            "closed. Verified by read-only utility workflow run 33578496332."
        ),
    },
    "packages/utils/src/promise.test.ts": {
        "rustTests": ["rust/tdd/workflow-utils/tests/promise.rs"],
        "notes": (
            "All five TypeScript resolver, resolve, reject, call-once, and memoization "
            "declarations execute against production workflow-utils. Rust additionally "
            "proves first-settlement-wins behavior, cloned cross-thread resolution, and "
            "retry after an initializer panic. Verified by read-only utility workflow run "
            "33578496332."
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
        if entry.get("status") != "red":
            raise SystemExit(f"expected RED override for {target}, got {entry.get('status')}")
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
    marker = "## Utility singleton and deferred promotions"
    if marker in text:
        return
    path.write_text(
        text.rstrip()
        + """

## Utility singleton and deferred promotions

| Contract | Status | Evidence |
| --- | --- | --- |
| Process-wide versioned singleton registry | PROVEN | 9 translated TypeScript tests plus concurrent initialization and type-confusion regressions; read-only workflow run `33578496332` |
| Deferred resolver and lazy once value | PROVEN | 5 translated TypeScript tests plus settlement-race, cross-thread, and panic-retry regressions; read-only workflow run `33578496332` |
"""
    )


def main() -> None:
    remove_red_registrations()
    promote_overrides()
    update_parity_matrix()


if __name__ == "__main__":
    main()
